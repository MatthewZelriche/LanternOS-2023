use crate::peripherals::UART;
use core::arch::asm;
use core::panic::PanicInfo;

extern "C" {
    static __PG_SIZE: u8;
}

pub fn page_size() -> u64 {
    unsafe { (&__PG_SIZE as *const u8) as u64 }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            writeln!(UART.lock(), $($arg)*).unwrap();
        }
    };
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprint!("{}", _info);
    loop {}
}

pub fn clear_tlb() {
    unsafe {
        asm!("TLBI VMALLE1", "DSB ISH", "ISB");
    }
}
