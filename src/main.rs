#![no_std]
#![no_main]
#![feature(core_intrinsics)]

mod concurrency;
mod mmio;
mod peripherals;
mod util;

use core::arch::global_asm;

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

#[no_mangle]
pub extern "C" fn main() -> ! {
    kprint!("Hello world! My number is: {}", 7);
    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}
