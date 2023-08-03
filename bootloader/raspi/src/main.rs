#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo};

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

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

#[no_mangle]
pub extern "C" fn main(dtb_ptr: *const u8) {
    println!("Raspi bootloader is preparing environment for kernel...");
    panic!("Test!");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}