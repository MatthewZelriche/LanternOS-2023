#![no_std]
#![no_main]

use core::panic::PanicInfo;

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

/*
 * By the time we reach this fn, the bootloader has already configured the UART clock rate and baud rate.
 * There's no need to send a mailbox message here; it would be redundant
 */
#[no_mangle]
pub extern "C" fn main() -> ! {
    kprint!("Hello from kernel main");

    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprint!("{}", _info);
    loop {}
}
