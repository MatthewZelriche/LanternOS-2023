#![no_std]
#![no_main]

use core::{arch::global_asm, hint::black_box, panic::PanicInfo};

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

#[no_mangle]
pub extern "C" fn main(dead_beef: u64) -> ! {
    // Debug statements to verify main function is executing with qemu 'info registers'
    let mut _db = dead_beef;
    black_box(_db = _db - 0xbeef);
    black_box(_db);

    // Never return from this divering fn
    loop {}
}

// Just spin the cpu...cant do anything else right now.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
