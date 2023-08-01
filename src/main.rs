#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(pointer_is_aligned)]
#![feature(int_roundings)]

mod concurrency;
mod memory;
mod mmio;
mod peripherals;
mod util;

use crate::{
    concurrency::spinlock::Spinlock,
    memory::{
        frame_allocator::PageFrameAllocator, init_mmu::init_mmu, map::MemoryMap,
        paging::PageTableRoot, util::MemSize, FRAME_ALLOCATOR, PAGE_SZ,
    },
};
use core::arch::global_asm;

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

#[no_mangle]
pub extern "C" fn main(dtb_ptr: *const u8) {
    kprint!("Booting kernel in privilage mode EL1...");
    kprint!("");
    // Load the dtb
    // Panic if we can't load it, for now
    let map = MemoryMap::new(dtb_ptr).expect("Failed to construct memory map");
    kprint!("Page size:       {}", MemSize { bytes: PAGE_SZ });
    kprint!(
        "Reserved Pages:  {}",
        (map.get_total_mem() - map.get_free_mem()) / PAGE_SZ
    );
    kprint!(
        "Available Pages: {}",
        map.get_free_mem().to_bytes() / PAGE_SZ
    );
    kprint!("Total Memory:    {}", map.get_total_mem());
    kprint!("Avail Memory:    {}", map.get_free_mem());
    kprint!("");
    kprint!("{}", map);

    kprint!("Setting up page frame allocator...");
    // This early init runs with 1 core and IRQ disabled, so we dont have to worry about get().unwrap() to
    // this frame allocator ever failing
    FRAME_ALLOCATOR.get_or_init(|| Spinlock::new(PageFrameAllocator::new(&map)));
    kprint!(
        "Initialized page frame allocator with {} free pages",
        FRAME_ALLOCATOR.get().unwrap().lock().num_free_pages()
    );

    kprint!("Identity mapping all physical memory...");
    let mut page_table = PageTableRoot::new();
    // Identity map all of physical memory to 4KiB pages
    let max_addr = map.get_total_mem().to_bytes();

    for page in (0..max_addr).step_by(0x40000000) {
        page_table
            .map_1gib_page(page)
            .expect("Failed to Identity map full physical memory");
    }

    // Turn on the MMU. From here on we are operating on virtual addresses
    init_mmu(&page_table);
    kprint!("MMU initialized to identity mapping scheme");

    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}
