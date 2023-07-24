use core::panic::PanicInfo;

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            use crate::peripherals::UART;
            writeln!(UART.lock(), $($arg)*).unwrap();
        }
    };
}

// Just spin the cpu...cant do anything else right now.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprint!("{}", _info);
    loop {}
}
