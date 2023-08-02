#[allow(dead_code)]
pub mod mailbox;
pub mod mmio;
pub mod uart;

use self::uart::Uart;
use self::{mailbox::Mailbox, mmio::Mmio};
use crate::concurrency::spinlock::{RawSpinlock, Spinlock};
use generic_once_cell::Lazy;

pub static MMIO: Lazy<RawSpinlock, Spinlock<Mmio>> = Lazy::new(|| Spinlock::new(Mmio::new()));
pub static UART: Lazy<RawSpinlock, Spinlock<Uart>> = Lazy::new(|| Spinlock::new(Uart::new()));
pub static MAILBOX: Lazy<RawSpinlock, Spinlock<Mailbox>> =
    Lazy::new(|| Spinlock::new(Mailbox::new()));
