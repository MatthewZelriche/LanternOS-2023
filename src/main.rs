#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(pointer_is_aligned)]

mod concurrency;
mod mmio;
mod peripherals;
mod util;

use core::arch::global_asm;

use crate::peripherals::{mailbox::GetClockRate, MAILBOX};

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

#[no_mangle]
pub extern "C" fn main(dtb_ptr: u64) -> ! {
    // Print current UART clock rate
    let msg = GetClockRate::new(2);
    let msg = MAILBOX.lock().send_message(msg).unwrap();
    kprint!(
        "Clock: {}, Rate: {} HZ",
        msg.data.get_clock_id(),
        msg.data.get_rate()
    );

    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}
