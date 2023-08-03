#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            use raspi_peripherals::UART;
            writeln!(UART.lock(), $($arg)*).unwrap();
        }
    };
}

extern "C" {
    static __PG_SIZE: u8;
}

pub fn page_size() -> u64 {
    unsafe { (&__PG_SIZE as *const u8) as u64 }
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    kprint!("Hello from kernel main");
    kprint!("");

    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprint!("{}", _info);
    loop {}
}
