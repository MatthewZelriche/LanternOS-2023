#![no_std]
#![no_main]
#![feature(int_roundings)]

mod init_mmu;
mod mem_size;
mod memory_map;

use crate::init_mmu::init_mmu;
use core::{
    arch::{asm, global_asm},
    ops::DerefMut,
    panic::PanicInfo,
    slice::{from_raw_parts, from_raw_parts_mut},
};

use align_data::include_aligned;
use elf_parse::{ElfFile, MachineType};
use generic_once_cell::Lazy;
use page_frame_allocator::PageFrameAllocator;
use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};
use raspi_paging::{FrameAlloc, MemType, PageTableRoot, VirtualAddr};
use raspi_peripherals::{
    mailbox::{Mailbox, Message, SetClockRate},
    uart::Uart,
};

use crate::{
    mem_size::MemSize,
    memory_map::{EntryType, MemoryMap, MemoryMapEntry},
};

pub static UART: Lazy<RawSpinlock, Spinlock<Uart>> = Lazy::new(|| Spinlock::new(Uart::new()));
pub static MAILBOX: Lazy<RawSpinlock, Spinlock<Mailbox>> =
    Lazy::new(|| Spinlock::new(Mailbox::new()));

// TODO: Find a way to handle automatically setting this to page size
#[repr(align(0x1000))]
struct AlignPage;
static KERNEL: &[u8] = include_aligned!(AlignPage, "../../../out/lantern-os.elf");

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

extern "C" {
    static __PG_SIZE: u8;
    static __KERNEL_VIRT_START: u8;
}
pub fn page_size() -> u64 {
    unsafe { (&__PG_SIZE as *const u8) as u64 }
}
pub fn kernel_virt_start() -> u64 {
    unsafe { (&__KERNEL_VIRT_START as *const u8) as u64 }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            writeln!(UART.lock(), $($arg)*).unwrap();
        }
    };
}

pub struct PageFrameAllocatorNewtype(PageFrameAllocator);
impl FrameAlloc for PageFrameAllocatorNewtype {
    fn alloc_frame(&mut self) -> *mut u8 {
        self.0.alloc_frame() as *mut u8
    }
}

pub static FRAME_ALLOCATOR: Lazy<RawSpinlock, Spinlock<PageFrameAllocatorNewtype>> =
    Lazy::new(|| {
        Spinlock::new(PageFrameAllocatorNewtype(PageFrameAllocator::new(
            page_size(),
        )))
    });

#[no_mangle]
pub extern "C" fn main(dtb_ptr: *const u8) -> ! {
    // Inform the raspi of our desired clock speed for the UART. Necessary for UART to function.
    // Mailbox requires physical address instead of virtual, but we don't have the MMU up yet
    // so it currently doesn't matter.
    let mut msg = SetClockRate::new(2, Uart::INIT_RATE_DEF);
    MAILBOX.lock().send_message((&mut msg) as *mut Message<_>);

    println!("Raspi bootloader is preparing environment for kernel...");

    // Load the dtb
    // Panic if we can't load it, for now
    let mut map = MemoryMap::new(dtb_ptr).expect("Failed to construct memory map");

    // TODO: Not sure why this is necessary...but if I don't reserve the very first page of memory,
    // attempting to write to that region causes cpu faults.
    // Something to do with QEMU? Or the exception vector?
    map.add_entry(MemoryMapEntry {
        base_addr: 0,
        size: MemSize { bytes: 0x1000 },
        end_addr: 0x1000,
        entry_type: EntryType::Firmware,
    })
    .unwrap();

    let kernel_elf = ElfFile::new(KERNEL).expect("Failed to parse kernel ELF");
    if kernel_elf.hdr.machine != MachineType::AARCH64 {
        panic!("Kernel ELF file is using the wrong architecture!");
    }
    println!("Successfully parsed kernel ELF");

    load_elf(&kernel_elf, &mut map);
    println!("Loaded Kernel ELF into memory");

    println!("");
    println!("Page size:       {}", MemSize { bytes: page_size() });
    println!(
        "Reserved Pages:  {}",
        (map.get_total_mem() - map.get_free_mem()) / page_size()
    );
    println!(
        "Available Pages: {}",
        map.get_free_mem().to_bytes() / page_size()
    );
    println!("Total Memory:    {}", map.get_total_mem());
    println!("Avail Memory:    {}", map.get_free_mem());
    println!("");
    println!("{}", map);

    println!("Initializing page frame allocator...");
    for entry in map.get_entries() {
        match entry.entry_type {
            EntryType::Free => {
                for addr in (entry.base_addr..entry.end_addr).step_by(page_size() as usize) {
                    // If we fail to add a page to the free list, just silently ignore
                    let _ = FRAME_ALLOCATOR.lock().0.free_frame(addr as *mut u64);
                }
            }
            _ => (),
        }
    }
    println!(
        "Successfully initialized page frame allocator with {} free frames.",
        FRAME_ALLOCATOR.lock().0.num_free_frames()
    );

    println!("Enabling MMU...");
    let mut page_table: PageTableRoot;
    let mut ttbr1 = PageTableRoot::new(page_size(), FRAME_ALLOCATOR.lock().deref_mut());
    // Identity map all of physical memory as 1GiB huge pages
    page_table = PageTableRoot::new(page_size(), FRAME_ALLOCATOR.lock().deref_mut());
    let max_addr = map.get_total_mem().to_bytes();
    for page in (0..max_addr).step_by(0x40000000) {
        page_table
            .map_1gib_page(page, FRAME_ALLOCATOR.lock().deref_mut())
            .expect("Failed to Identity map full physical memory");
    }

    // Virtually map kernel memory to higher half with 4KiB pages
    let kernel_region = map
        .get_entries()
        .iter()
        .find(|x| x.entry_type == EntryType::Kernel)
        .expect("Failed to find kernel in memory");

    let kernel_virt_start = kernel_virt_start();
    let mut offset = 0;
    for phys_page in (kernel_region.base_addr..kernel_region.end_addr).step_by(page_size() as usize)
    {
        // TODO: Dehardcode
        ttbr1
            .map_page(
                phys_page,
                raspi_paging::VirtualAddr(kernel_virt_start + offset),
                MemType::NORMAL_CACHEABLE,
                FRAME_ALLOCATOR.lock().deref_mut(),
            )
            .expect("Failed to virtually map kernel");
        offset += page_size();
    }
    // Also map the stack to the higher half
    // TODO: Guard page
    let stack_region = map
        .get_entries()
        .iter()
        .find(|x| x.entry_type == EntryType::Stack)
        .expect("Couldn't find kernel stack in memory");
    let stack_virt_start = kernel_virt_start + offset;
    offset = 0;
    for phys_page in (stack_region.base_addr..stack_region.end_addr).step_by(page_size() as usize) {
        ttbr1
            .map_page(
                phys_page,
                VirtualAddr(stack_virt_start + offset),
                MemType::NORMAL_CACHEABLE,
                FRAME_ALLOCATOR.lock().deref_mut(),
            )
            .expect("Failed to virtually map stack");
        offset += page_size();
    }
    let stack_top = stack_virt_start + offset;

    // Remap MMIO
    let mmio_start = 0xFFFF008000000000;
    let mut offset = 0;
    let mmio_segment = map
        .get_entries()
        .iter()
        .find(|x| x.entry_type == EntryType::Mmio)
        .expect("Failed to find MMIO in memory");
    for phys_page in (mmio_segment.base_addr..mmio_segment.end_addr).step_by(page_size() as usize) {
        ttbr1
            .map_page(
                phys_page,
                VirtualAddr(mmio_start + offset),
                MemType::DEVICE,
                FRAME_ALLOCATOR.lock().deref_mut(),
            )
            .expect("Failed to remap MMIO to higher half!");
        offset += page_size();
    }

    init_mmu(&page_table, &ttbr1);
    println!("Successfully enabled the MMU");

    // Transfer control to the kernel
    println!(
        "Transferring control to kernel entry point {:#x}",
        kernel_elf.hdr.entry
    );
    println!("");

    let fn_void_ptr = kernel_elf.hdr.entry as *const ();
    unsafe {
        asm!("mov sp, {stack_top}", 
        "br {entry_point}", 
        stack_top = in(reg) stack_top, 
        entry_point = in(reg) fn_void_ptr);
    }
    loop {}
}

fn load_elf(kernel_elf: &ElfFile, map: &mut MemoryMap) {
    // TODO: Less hacky way of loading
    // Copy kernel into memory
    // TODO: Add this to memory map
    let mut kernel_memsz: u64 = 0;
    for program in kernel_elf
        .program_headers()
        .expect("Kernel ELF has no program segments")
    {
        if program.program_type == 1 {
            kernel_memsz += program.memsz;
        }
    }
    kernel_memsz = kernel_memsz.next_multiple_of(page_size());
    // Find a contiguous region in physical memory to store the segment
    let region = map
        .get_entries()
        .iter()
        .find(|x| x.size.bytes >= kernel_memsz && x.entry_type == EntryType::Free)
        .expect("Failed to find available memory for kernel")
        .clone();
    let base_addr = kernel_elf
        .program_headers()
        .unwrap()
        .next()
        .unwrap()
        .virt_addr;

    for program in kernel_elf
        .program_headers()
        .expect("Kernel ELF has no program segments")
    {
        // Loadable segment
        if program.program_type == 1 {
            let file_segment = unsafe {
                from_raw_parts(
                    KERNEL.as_ptr().add(program.offset as usize),
                    program.filesz as usize,
                )
            };

            // TODO: This is currently hardcoded because we know in QEMU
            // (at this point) we can rely on the kernel getting loaded at
            // address 0, which is what we've linked the kernel to. Once we enable the MMU,
            // this link position needs to change and we will need to remap the kernel.
            let mem_offset = program.virt_addr - base_addr;
            let mem_segment = unsafe {
                from_raw_parts_mut(
                    (region.base_addr + mem_offset) as *mut u8,
                    program.memsz as usize,
                )
            };

            (&mut mem_segment[0..program.filesz as usize]).copy_from_slice(file_segment);

            if program.memsz > program.filesz {
                (&mut mem_segment[program.filesz as usize..]).fill(0);
            }
        }
    }

    // Add this kernel region to the memory map
    map.add_entry(MemoryMapEntry {
        base_addr: region.base_addr,
        size: MemSize {
            bytes: kernel_memsz,
        },
        end_addr: region.base_addr + kernel_memsz,
        entry_type: memory_map::EntryType::Kernel,
    })
    .expect("Failed to install kernel data into memory map");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}
