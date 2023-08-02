#![no_std]
#![no_main]

use core::panic::PanicInfo;

pub extern "C" fn main() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
