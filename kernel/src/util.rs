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
            write!(UART.lock(), $($arg)*).unwrap();
        }
    };
}

#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            writeln!(UART.lock(), $($arg)*).unwrap();
        }
    };
}

#[macro_export]
macro_rules! kprints {
    ($core:expr, $($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut lock = UART.lock();
            write!(lock, "[Core {}] ", $core).unwrap();
            writeln!(lock, $($arg)*).unwrap();
        }
    };
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("\nKERNEL PANIC!");
    if let Some(location) = info.location() {
        kprintln!("Location: {}", location);
    }
    if let Some(message) = info.message() {
        kprintln!("Reason: \n\n{}", message);
    }

    loop {}
}

pub fn clear_tlb() {
    unsafe {
        asm!("TLBI VMALLE1", "DSB ISH", "ISB");
    }
}
