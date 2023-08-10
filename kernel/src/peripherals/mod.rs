use generic_once_cell::Lazy;
use raspi_concurrency::mutex::{Mutex, RawMutex};
use raspi_peripherals::{mailbox::Mailbox, uart::Uart};

pub static UART: Lazy<RawMutex, Mutex<Uart>> = Lazy::new(|| Mutex::new(Uart::new()));
pub static MAILBOX: Lazy<RawMutex, Mutex<Mailbox>> = Lazy::new(|| Mutex::new(Mailbox::new()));
