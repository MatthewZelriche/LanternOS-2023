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

use crate::memory::{map::MemoryMap, util::MemSize, PAGE_SZ};
use core::arch::global_asm;

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

#[no_mangle]
pub extern "C" fn main(dtb_ptr: *const u8) -> ! {
    kprint!("Booting kernel...");
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

    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}
