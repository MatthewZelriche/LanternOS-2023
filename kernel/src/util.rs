use crate::peripherals::UART;
use core::arch::asm;
use core::panic::PanicInfo;

extern "C" {
    static __PG_SIZE: u8;
    static __KERNEL_VIRT_START: u8;
}
pub fn page_size() -> u64 {
    unsafe { (&__PG_SIZE as *const u8) as u64 }
}
pub fn kernel_virt_start() -> u64 {
    unsafe { (&__KERNEL_VIRT_START as *const u8) as u64 }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            use raspi::peripherals::timer::uptime;
            use crate::peripherals::UART;
            let mut lock = UART.lock();
            write!(lock, "[{:.5}] ", uptime().as_secs_f64()).unwrap();
            write!(lock, $($arg)*).unwrap();
        }
    };
}

#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            use raspi::peripherals::timer::uptime;
            use crate::peripherals::UART;
            let mut lock = UART.lock();
            write!(lock, "[{:.5}] ", uptime().as_secs_f64()).unwrap();
            writeln!(lock, $($arg)*).unwrap();
        }
    };
}

#[macro_export]
macro_rules! kprints {
    ($core:expr, $($arg:tt)*) => {
        {
            use core::fmt::Write;
            use crate::peripherals::UART;
            use raspi::peripherals::timer::uptime;
            let mut lock = UART.lock();
            write!(lock, "[{:.5} | Core {}] ", uptime().as_secs_f64(), $core).unwrap();
            writeln!(lock, $($arg)*).unwrap();
        }
    };
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Cant use kprint* macros here because it will result in jumbled text
    // Need to handle the lock manually
    use core::fmt::Write;
    let mut lock = UART.lock();

    writeln!(lock, "\nKERNEL PANIC!").unwrap();
    if let Some(location) = info.location() {
        writeln!(lock, "Location: {}", location).unwrap();
    }
    if let Some(message) = info.message() {
        writeln!(lock, "Reason: \n{}", message).unwrap();
    }

    loop {}
}

pub fn clear_tlb() {
    unsafe {
        asm!("TLBI VMALLE1", "DSB ISH", "ISB");
    }
}
