#![feature(pointer_is_aligned)]
#![feature(core_intrinsics)]
#![no_std]

use generic_once_cell::Lazy;
use mailbox::Mailbox;
use mmio::Mmio;
use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};
use uart::Uart;

#[allow(dead_code)]
pub mod mailbox;
pub mod mmio;
pub mod uart;

pub static MMIO: Lazy<RawSpinlock, Spinlock<Mmio>> = Lazy::new(|| Spinlock::new(Mmio::new()));
pub static UART: Lazy<RawSpinlock, Spinlock<Uart>> = Lazy::new(|| Spinlock::new(Uart::new()));
pub static MAILBOX: Lazy<RawSpinlock, Spinlock<Mailbox>> =
    Lazy::new(|| Spinlock::new(Mailbox::new()));
