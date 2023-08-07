#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use generic_once_cell::Lazy;
use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};
use raspi_paging::PageTableRoot;
use raspi_peripherals::{mailbox::Mailbox, uart::Uart};

// We set mmio base to a constant, but immediately update it to the correct
// virtual address in kmain()
pub static UART: Lazy<RawSpinlock, Spinlock<Uart>> =
    Lazy::new(|| Spinlock::new(Uart::new(raspi_peripherals::MMIO_PHYS_BASE)));
pub static MAILBOX: Lazy<RawSpinlock, Spinlock<Mailbox>> =
    Lazy::new(|| Spinlock::new(Mailbox::new(raspi_peripherals::MMIO_PHYS_BASE)));

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
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

fn clear_tlb() {
    unsafe {
        asm!("TLBI VMALLE1", "DSB ISH", "ISB");
    }
}

/*
 * By the time we reach this fn, the bootloader has already configured the UART clock rate and baud rate.
 * There's no need to send a mailbox message here; it would be redundant
 */
#[no_mangle]
pub extern "C" fn main() -> ! {
    // Update our MMIO address to use higher half
    // TODO: Dehardcode this
    UART.lock().update_mmio_base(0xFFFF008002000000);
    MAILBOX.lock().update_mmio_base(0xFFFF008002000000);

    // Unmap our identity mapping
    // TODO: Dehardcode max mem size
    let mut ttbr0 =
        PageTableRoot::from_ptr(aarch64_cpu::registers::TTBR0_EL1.get_baddr(), page_size());
    for page in (0..0x100000000u64).step_by(0x40000000) {
        ttbr0.unmap_1gib_page(page);
    }
    clear_tlb();

    kprint!("Hello from kernel main");

    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprint!("{}", _info);
    loop {}
}
