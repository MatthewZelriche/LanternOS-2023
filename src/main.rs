#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo};

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}

// Just spin the cpu...cant do anything else right now.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
