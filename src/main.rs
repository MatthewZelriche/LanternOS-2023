#![no_std]
#![no_main]
#![feature(core_intrinsics)]

mod mmio;

use bitfield::Bit;
use core::{arch::global_asm, hint, panic::PanicInfo};
use mmio::{mmio_read, mmio_write};

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

#[no_mangle]
pub extern "C" fn main() -> ! {
    kprint("Hello, world!\n");

    // Never return from this diverging fn
    panic!()
}

fn kputc(c: char) {
    while mmio_read(mmio::UART_FR).bit(5) {
        hint::spin_loop();
    }
    mmio_write(mmio::UART0_BASE, c as u64);
}

fn kprint(s: &str) {
    for c in s.chars() {
        kputc(c);
    }
}

// Just spin the cpu...cant do anything else right now.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprint("Kernel panic!");
    loop {}
}
