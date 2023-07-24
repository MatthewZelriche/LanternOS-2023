pub mod mailbox;
pub mod uart;

use self::mailbox::Mailbox;
use self::uart::Uart;
use crate::concurrency::spinlock::{RawSpinlock, Spinlock};
use generic_once_cell::Lazy;

pub static UART: Lazy<RawSpinlock, Spinlock<Uart>> = Lazy::new(|| Spinlock::new(Uart::new()));
pub static MAILBOX: Lazy<RawSpinlock, Spinlock<Mailbox>> =
    Lazy::new(|| Spinlock::new(Mailbox::new()));
