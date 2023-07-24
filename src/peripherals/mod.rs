pub mod uart;

use self::uart::Uart;
use crate::concurrency::spinlock::{RawSpinlock, Spinlock};
use generic_once_cell::Lazy;

pub static UART: Lazy<RawSpinlock, Spinlock<Uart>> = Lazy::new(|| Spinlock::new(Uart::new()));
