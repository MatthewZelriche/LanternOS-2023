use generic_once_cell::{Lazy, OnceCell};
use raspi::{
    concurrency::mutex::{Mutex, RawMutex},
    peripherals::{emmc::EMMCController, mailbox::Mailbox, uart::Uart},
};

pub static UART: Lazy<RawMutex, Mutex<Uart>> = Lazy::new(|| Mutex::new(Uart::new()));
pub static MAILBOX: Lazy<RawMutex, Mutex<Mailbox>> = Lazy::new(|| Mutex::new(Mailbox::new()));
pub static EMMC2: OnceCell<RawMutex, Mutex<EMMCController>> = OnceCell::new();
