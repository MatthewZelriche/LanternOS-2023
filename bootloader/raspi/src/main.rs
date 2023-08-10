#![no_std]
#![no_main]
#![feature(int_roundings)]

mod boot_alloc;
mod init_mmu;

use crate::boot_alloc::FrameAlloc;
use crate::init_mmu::init_mmu;
use align_data::include_aligned;
use fdt_rs::base::DevTree;
use fdt_rs::error::DevTreeError;
use fdt_rs::prelude::{FallibleIterator, PropReader};
use raspi_memory::mem_size::MemSize;
use raspi_memory::memory_map::{MemoryMap, MemoryMapEntry, EntryType};
use raspi_peripherals::get_board_peripheral_range;
use raspi_peripherals::mailbox::GetGpuMemory;
use core::ops::Deref;
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
    slice::{from_raw_parts, from_raw_parts_mut},
};
use elf_parse::{ElfFile, MachineType};
use generic_once_cell::{Lazy, OnceCell};
use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};
use raspi_memory::page_table::{MemoryType, PageTable, VirtualAddr, PageAlloc};
use raspi_peripherals::{
    mailbox::{Mailbox, Message, SetClockRate},
    uart::Uart,
};

// Peripheral singletons
pub static UART: Lazy<RawSpinlock, Spinlock<Uart>> = Lazy::new(|| Spinlock::new(Uart::new()));
pub static MAILBOX: Lazy<RawSpinlock, Spinlock<Mailbox>> =
    Lazy::new(|| Spinlock::new(Mailbox::new()));

// TODO: Find a way to handle automatically setting this to page size
// To avoid having to implement an entire FAT library for the bootloader, we embed the entire
// ELF file directly into the bootloader
#[repr(align(0x1000))]
struct AlignPage;
static KERNEL: &[u8] = include_aligned!(AlignPage, "../../../out/lantern-os.elf");

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("el_transition.S"));
global_asm!(include_str!("start_secondary.S"));
global_asm!(include_str!("start.S"));

extern "C" {
    static __PG_SIZE: u8;
    static __KERNEL_VIRT_START: u8;
    static __stack_end: u8;
    static __bootloader_start: u8;
    static __bootloader_end: u8;
    static __stack: u8;

    fn init_secondary_core(mailbox_addr: u64, fun_ptr: u64);
    fn core_1_start();
    fn core_2_start();
    fn core_3_start();
}
pub fn page_size() -> u64 {
    unsafe { (&__PG_SIZE as *const u8) as u64 }
}
pub fn kernel_virt_start() -> u64 {
    unsafe { (&__KERNEL_VIRT_START as *const u8) as u64 }
}
pub fn get_page_addr(addr: u64) -> u64 {
    addr & !(page_size() - 1)
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

static MEM_MAP: OnceCell<RawSpinlock, Spinlock<MemoryMap>> = OnceCell::new();

#[no_mangle]
pub extern "C" fn secondary_core_main(core_num: u64, ttbr0_ptr: u64, ttbr1_ptr: u64) -> ! {
    loop{}
}

#[no_mangle]
pub extern "C" fn main(dtb_ptr: *const u8) -> ! {
    // Inform the raspi of our desired clock speed for the UART. Necessary for UART to function.
    // Mailbox requires physical address instead of virtual, but we don't have the MMU up yet
    // so it currently doesn't matter.
    let mut msg = SetClockRate::new(2, Uart::INIT_RATE_DEF);
    MAILBOX.lock().send_message((&mut msg) as *mut Message<_>);

    println!("Raspi bootloader is preparing environment for kernel...");

    let map_mutex = MEM_MAP.get_or_init(|| {
        Spinlock::new(MemoryMap::new())
    });
    reserve_memory_regions(dtb_ptr, map_mutex).expect("Failed to create memory map");

    // TODO: Not sure why this is necessary...but if I don't reserve the very first page of memory,
    // attempting to write to that region causes cpu faults.
    // Something to do with QEMU? Or the exception vector?
    map_mutex
        .lock()
        .add_entry(MemoryMapEntry {
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

    load_elf(&kernel_elf, map_mutex);
    println!("Loaded Kernel ELF into memory");

    println!("Initializing page frame allocator...");
    // We are definitely singlethreaded in the bootloader, but raspi-paging expects a mutex to
    // a page frame allocator to take advantage of interior mutability
    let frame_allocator: Spinlock<FrameAlloc> = Spinlock::new(FrameAlloc::new());
    for entry in map_mutex.lock().get_entries() {
        match entry.entry_type {
            EntryType::Free => {
                for addr in (entry.base_addr..entry.end_addr).step_by(page_size() as usize) {
                    // If we fail to add a page to the free list, just silently ignore
                    let _ = frame_allocator.lock().deallocate_frame(addr as *mut u8);
                }
            }
            _ => (),
        }
    }
    let start_free_frames = frame_allocator.lock().num_free_frames();
    println!(
        "Successfully initialized page frame allocator with {} free frames.",
        start_free_frames
    );

    println!("Enabling MMU...");
    let mut ttbr1 = PageTable::new(&frame_allocator).expect("Failed to construct page table");
    // Identity map all of physical memory as 1GiB huge pages
    let mut page_table = PageTable::new(&frame_allocator).expect("Failed to construct page table");
    let max_addr = map_mutex.lock().get_total_mem().to_bytes();
    for page in (0..max_addr).step_by(0x40000000) {
        page_table
            .map_1gib_page(page, VirtualAddr(page), MemoryType::DEVICE)
            .expect("Failed to Identity map full physical memory");
    }

    // Virtually map kernel memory to higher half with 4KiB pages
    let lock = map_mutex.lock();
    let kernel_region = lock
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
                VirtualAddr(kernel_virt_start + offset),
                MemoryType::NORMAL_CACHEABLE,
            )
            .expect("Failed to virtually map kernel");
        offset += page_size();
    }

    // Allocate one frame for each CPU's kernel stack
    let mut kernel_stacks_phys_address: [u64; 4] = [0, 0, 0, 0];
    let mut kernel_stacks_virt_top: [u64; 4] = [0, 0, 0, 0];
    // TODO: Guard pages
    for i in 0..4 {
        kernel_stacks_phys_address[i] = frame_allocator.lock().allocate_frame().expect("Failed to allocate frame for kernel stack") as u64;
        ttbr1
            .map_page(
                kernel_stacks_phys_address[i],
                VirtualAddr(kernel_virt_start + offset),
                MemoryType::NORMAL_CACHEABLE,
            )
            .expect("Failed to virtually map stack");
        offset += page_size();
        kernel_stacks_virt_top[i] = kernel_virt_start + offset;
        println!("Phys addr: {:#x}, virt addr: {:#x}", kernel_stacks_phys_address[i], kernel_stacks_virt_top[i]);
    }

    // Linear mapping of all physical RAM into the higher half
    let memory_linear_map_start = kernel_stacks_virt_top[3].next_multiple_of(0x40000000);
    let max_addr = map_mutex.lock().get_total_mem().to_bytes();
    for page in (0..max_addr).step_by(0x40000000) {
        ttbr1
            .map_1gib_page(page, VirtualAddr(memory_linear_map_start + page), MemoryType::DEVICE)
            .expect("Failed to Identity map full physical memory");
    }
    println!("Mapped physical memory into higher half starting at address: {:#x}", memory_linear_map_start);

    println!("Printing memory map: ");
    println!("");
    println!("Page size:       {}", MemSize { bytes: page_size() });
    println!(
        "Reserved Pages:  {}",
        (map_mutex.lock().get_total_mem() - map_mutex.lock().get_free_mem()) / page_size()
    );
    println!(
        "Available Pages: {}",
        map_mutex.lock().get_free_mem().to_bytes() / page_size()
    );
    println!("Total Memory:    {}", map_mutex.lock().get_total_mem());
    println!("Avail Memory:    {}", map_mutex.lock().get_free_mem());
    println!("");
    println!("{}", map_mutex.lock());
    let mut bl_reserved_count = 0;
    for entry in map_mutex.lock().get_entries().iter().filter(|x| x.entry_type == EntryType::BLReserved) {
        bl_reserved_count += entry.size.bytes;
    }
    println!("Bootloader allocated {} pages of memory in total", bl_reserved_count / page_size());
    let final_allocated_pages = start_free_frames - frame_allocator.lock().num_free_frames();
    // Sanity check to ensure our memory map was updated correctly
    // Safety: Unsafe to allocate ANY frames past this point
    assert!((bl_reserved_count / page_size()) == final_allocated_pages);

    // Enable MMU for the primary core
    init_mmu(&page_table, &ttbr1);
    println!("Successfully enabled the MMU");

    // TODO: Spin up secondary cores
    println!("Initializng secondary cores...");
    const CPU_MAILBOX_REGS: [u64; 3] = [0xE0, 0xE8, 0xF0];
    const ARG_ADDRESSES: [u64; 3] = [0xFA0, 0xFC0, 0xFE0];
    for (i, register) in CPU_MAILBOX_REGS.iter().enumerate() {
        unsafe {
            // Write in the arguments
            core::ptr::write_volatile(ARG_ADDRESSES[i] as *mut u64, kernel_stacks_phys_address[i + 1]);
            core::ptr::write_volatile((ARG_ADDRESSES[i] + 8) as *mut u64, page_table.as_raw_ptr() as u64);
            core::ptr::write_volatile((ARG_ADDRESSES[i] + 16) as *mut u64, ttbr1.as_raw_ptr() as u64);

            match register {
                0xE0 => init_secondary_core(*register, core_1_start as u64),
                0xE8 => init_secondary_core(*register, core_2_start as u64),
                0xF0 => init_secondary_core(*register, core_3_start as u64),
                _ => panic!(),
            };
        }
    }
    println!("Bootloader has successfully initialized secondary cores");

    // Transfer control to the kernel
    println!(
        "Transferring control to kernel entry point {:#x}",
        kernel_elf.hdr.entry
    );
    // TODO: Have to drop stuff because rust cant figure out we done when we move to kmain

    let fn_void_ptr = kernel_elf.hdr.entry as *const ();
    // Safety: We are about to leave the bootloader entirely and enter kernel init.
    // Normally, grabbing a pointer to a OnceCell blocked by a mutex would be wildly unsafe,
    // but we know that no bootloader code will ever execute again after we jump to the kernel.
    // We can create a new OnceCell in kernel space by copying the memory map at this pointer, soundly.
    let mem_map_addr = MEM_MAP.get().unwrap().lock().deref() as *const MemoryMap;
    unsafe {
        asm!("mov sp, {stack}", 
        "mov x0, {memory_linear_map_start}", 
        "mov x1, {memory_map_addr}",
        "br {entry}", 
        stack = in(reg) kernel_stacks_virt_top[0], 
        memory_linear_map_start = in(reg) memory_linear_map_start,
        memory_map_addr = in(reg) mem_map_addr,
        entry = in(reg) fn_void_ptr);
    }
    loop {}
}

fn reserve_memory_regions(dtb_ptr: *const u8, map: &Spinlock<MemoryMap>) -> Result<(), DevTreeError> {
    let dtb: DevTree;
    unsafe {
        // Sound because this memory region will be protected by the memory map for the entire
        // lifetime of the os
        dtb = DevTree::from_raw_pointer(dtb_ptr).expect("Failed to read dtb! Err");
    }

    // Determine cell sizes
    let mut address_cells = 0;
    let mut size_cells = 0;
    if let Some(root) = dtb.root()? {
        address_cells = root
            .props()
            .find(|x| Ok(x.name()? == "#address-cells"))?
            .ok_or(DevTreeError::ParseError)?
            .u32(0)?;
        size_cells = root
            .props()
            .find(|x| Ok(x.name()? == "#size-cells"))?
            .ok_or(DevTreeError::ParseError)?
            .u32(0)?;
    }

    // First enumerate our free memory blocks
    let mut max_addr: u64 = 0;
    dtb.nodes()
        .filter(|x| Ok(x.name()?.contains("memory@")))
        .for_each(|x| {
            let reg = x
                .props()
                .find(|x| Ok(x.name()? == "reg"))?
                .ok_or(DevTreeError::ParseError)?;

            let base_addr: u64;
            let size_bytes: u64;
            match address_cells {
                1 => base_addr = reg.u32(0)?.into(),
                2 => base_addr = reg.u64(0)?,
                _ => return Err(DevTreeError::ParseError),
            }

            match size_cells {
                1 => size_bytes = reg.u32(address_cells as usize)?.into(),
                2 => size_bytes = reg.u64(address_cells as usize)?,
                _ => return Err(DevTreeError::ParseError),
            }

            if base_addr + size_bytes > max_addr {
                max_addr = base_addr + size_bytes;
            }

            map.lock().add_entry(MemoryMapEntry {
                base_addr,
                size: MemSize { bytes: size_bytes },
                end_addr: base_addr + size_bytes,
                entry_type: EntryType::Free,
            }).map_err(|_| DevTreeError::NotEnoughMemory)
        })?;

    // Now we can start assigning reserved blocks...

    // Find and reserve pages for the DTB
    let dtb_page_start = get_page_addr(dtb_ptr as u64);
    let dtb_page_end = (dtb_page_start + dtb.totalsize() as u64).next_multiple_of(page_size());
    let dtb_size_bytes = dtb_page_end - dtb_page_start;
    map.lock().add_entry(MemoryMapEntry {
        base_addr: dtb_page_start,
        size: MemSize {
            bytes: dtb_size_bytes,
        },
        end_addr: dtb_page_end,
        entry_type: EntryType::DtReserved,
    }).map_err(|_| DevTreeError::NotEnoughMemory)?;

    // Reserve GPU firmware
    let mut msg = GetGpuMemory::new();
    MAILBOX.lock().send_message((&mut msg) as *mut Message<_>);
    if msg.code != Mailbox::RESP_SUCCESS {
        return Err(DevTreeError::ParseError);
    }
    let start: u64 = msg.data.get_base().into();
    let size: u64 = msg.data.get_size().into();
    let end: u64 = start + size;
    map.lock().add_entry(MemoryMapEntry {
        base_addr: start,
        size: MemSize { bytes: size.into() },
        end_addr: end,
        entry_type: EntryType::Firmware,
    }).map_err(|_| DevTreeError::NotEnoughMemory)?;

    // Reserve the region for MMIO
    let (peripherals_phys_base, peripherals_phys_end) = get_board_peripheral_range();
    let size = peripherals_phys_end - peripherals_phys_base;
    map.lock().add_entry(MemoryMapEntry {
        base_addr: peripherals_phys_base,
        size: MemSize { bytes: size },
        end_addr: peripherals_phys_end,
        entry_type: EntryType::Mmio,
    }).map_err(|_| DevTreeError::NotEnoughMemory)?;

    // Reserve the stack region
    let stack_start;
    let stack_end;
    unsafe {
        stack_start = (&__stack_end as *const u8) as u64;
        stack_end = (&__stack as *const u8) as u64;
    }
    let stack_start_page_addr = get_page_addr(stack_start);
    let stack_end_page_addr = get_page_addr(stack_end); // Stack end is exclusive
    let stack_size = stack_end_page_addr - stack_start_page_addr;
    map.lock().add_entry(MemoryMapEntry {
        base_addr: stack_start_page_addr,
        size: MemSize { bytes: stack_size },
        end_addr: stack_end_page_addr,
        entry_type: EntryType::Bootloader,
    }).map_err(|_| DevTreeError::NotEnoughMemory)?;

    // Reserve the bootloader region
    let bl_start;
    let bl_end;
    unsafe {
        bl_start = (&__bootloader_start as *const u8) as u64;
        bl_end = &__bootloader_end as *const u8 as u64;
    }
    let bl_start_page_addr = get_page_addr(bl_start);
    let bl_end_page_addr = bl_end.next_multiple_of(page_size());
    let bl_size = bl_end_page_addr - bl_start_page_addr;
    map.lock().add_entry(MemoryMapEntry {
        base_addr: bl_start_page_addr,
        size: MemSize { bytes: bl_size },
        end_addr: bl_end_page_addr,
        entry_type: EntryType::Bootloader,
    }).map_err(|_| DevTreeError::NotEnoughMemory)?;

    map.lock().set_total_mem(max_addr);
    Ok(())
}

fn load_elf(kernel_elf: &ElfFile, map: &Spinlock<MemoryMap>) {
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
        .lock()
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
    map.lock()
        .add_entry(MemoryMapEntry {
            base_addr: region.base_addr,
            size: MemSize {
                bytes: kernel_memsz,
            },
            end_addr: region.base_addr + kernel_memsz,
            entry_type: EntryType::Kernel,
        })
        .expect("Failed to install kernel data into memory map");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}
