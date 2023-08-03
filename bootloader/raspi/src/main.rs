#![no_std]
#![no_main]
#![feature(int_roundings)]

mod mem_size;
mod memory_map;

use core::{arch::global_asm, panic::PanicInfo};

use align_data::include_aligned;
use elf_parse::ElfFile;
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

    println!("Setting up page frame allocator...");
    // Initialize all free pages in the map into the freelist
    for entry in map.get_entries() {
        match entry.entry_type {
            memory_map::EntryType::Free => {
                for addr in (entry.base_addr..entry.end_addr).step_by(page_size() as usize) {
                    // If we fail to add a page to the free list, just silently ignore
                    let _ = FRAME_ALLOCATOR.lock().free_page(addr as *mut u64);
                }
            }
            _ => (),
        }
    }
    println!(
        "Initialized page frame allocator with {} free pages",
        FRAME_ALLOCATOR.lock().num_free_pages()
    );

    println!("Begin parsing kernel ELF...");
    ElfFile::new(KERNEL).expect("Failed to parse kernel ELF");

    panic!("Failed to load kernel!");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}
