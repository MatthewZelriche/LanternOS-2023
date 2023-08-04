#![no_std]
#![no_main]
#![feature(int_roundings)]

mod mem_size;
mod memory_map;

use core::{
    arch::global_asm,
    mem::transmute,
    panic::PanicInfo,
    slice::{from_raw_parts, from_raw_parts_mut},
};

use align_data::include_aligned;
use elf_parse::{ElfFile, MachineType};
use generic_once_cell::Lazy;
use page_frame_allocator::PageFrameAllocator;
use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};

use crate::{
    mem_size::MemSize,
    memory_map::{EntryType, MemoryMap, MemoryMapEntry},
};

// TODO: Find a way to handle automatically setting this to page size
#[repr(align(0x1000))]
struct AlignPage;
static KERNEL: &[u8] = include_aligned!(AlignPage, "../../../out/lantern-os.elf");

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

extern "C" {
    static __PG_SIZE: u8;
}
pub fn page_size() -> u64 {
    unsafe { (&__PG_SIZE as *const u8) as u64 }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            use raspi_peripherals::UART;
            writeln!(UART.lock(), $($arg)*).unwrap();
        }
    };
}

pub static FRAME_ALLOCATOR: Lazy<RawSpinlock, Spinlock<PageFrameAllocator>> =
    Lazy::new(|| Spinlock::new(PageFrameAllocator::new(page_size())));

#[no_mangle]
pub extern "C" fn main(dtb_ptr: *const u8) {
    println!("Raspi bootloader is preparing environment for kernel...");

    // Load the dtb
    // Panic if we can't load it, for now
    let mut map = MemoryMap::new(dtb_ptr).expect("Failed to construct memory map");

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
                    let _ = FRAME_ALLOCATOR.lock().free_page(addr as *mut u64);
                }
            }
            _ => (),
        }
    }
    println!(
        "Successfully initialized page frame allocator with {} free pages.",
        FRAME_ALLOCATOR.lock().num_free_pages()
    );
    // Transfer control to the kernel
    println!("Transferring control to kernel entry point...");
    println!("");
    type EntryPoint = extern "C" fn() -> !;
    let fn_void_ptr = kernel_elf.hdr.entry as *const ();
    let entry_point: EntryPoint = unsafe { transmute(fn_void_ptr) };
    entry_point();



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
        .find(|x| x.size.bytes >= kernel_memsz)
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
