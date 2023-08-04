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

use crate::{mem_size::MemSize, memory_map::MemoryMap};

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
    println!("");

    // Load the dtb
    // Panic if we can't load it, for now
    let map = MemoryMap::new(dtb_ptr).expect("Failed to construct memory map");
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

    let kernel_elf = ElfFile::new(KERNEL).expect("Failed to parse kernel ELF");
    if kernel_elf.hdr.machine != MachineType::AARCH64 {
        panic!("Kernel ELF file is using the wrong architecture!");
    }
    println!("Successfully parsed kernel ELF");

    // Copy kernel into memory
    // TODO: Add this to memory map
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
            let mem_segment =
                unsafe { from_raw_parts_mut(program.virt_addr as *mut u8, program.memsz as usize) };

            (&mut mem_segment[0..program.filesz as usize]).copy_from_slice(file_segment);

            if program.memsz > program.filesz {
                (&mut mem_segment[program.filesz as usize..]).fill(0);
            }
        }
    }
    println!("Loaded Kernel ELF into memory");

    // Transfer control to the kernel
    println!("Transferring control to kernel entry point...");
    println!("");
    type EntryPoint = extern "C" fn() -> !;
    let fn_void_ptr = kernel_elf.hdr.entry as *const ();
    let entry_point: EntryPoint = unsafe { transmute(fn_void_ptr) };
    entry_point();
    }
    println!("Successfully parsed kernel ELF");

    panic!("Failed to load kernel!");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}
