use generic_once_cell::Lazy;
use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};
use raspi_peripherals::{mailbox::Mailbox, uart::Uart};

pub static UART: Lazy<RawSpinlock, Spinlock<Uart>> = Lazy::new(|| Spinlock::new(Uart::new()));
pub static MAILBOX: Lazy<RawSpinlock, Spinlock<Mailbox>> =
    Lazy::new(|| Spinlock::new(Mailbox::new()));
