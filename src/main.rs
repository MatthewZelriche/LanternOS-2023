#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}

// Just spin the cpu...cant do anything else right now.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
