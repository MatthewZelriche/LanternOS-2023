#![no_std]
#![no_main]

pub mod peripherals;
pub mod util;

use crate::{
    peripherals::{MAILBOX, UART},
    util::page_size,
};
use raspi_paging::PageTableRoot;
use raspi_peripherals::get_mmio_offset_from_peripheral_base;

/*
 * By the time we reach this fn, the bootloader has already configured the UART clock rate and baud rate.
 * There's no need to send a mailbox message here; it would be redundant
 */
#[no_mangle]
pub extern "C" fn main() -> ! {
    // Update our MMIO address to use higher half
    // TODO: Dehardcode this
    UART.lock()
        .update_mmio_base(0xFFFF008000000000 + get_mmio_offset_from_peripheral_base());
    MAILBOX
        .lock()
        .update_mmio_base(0xFFFF008000000000 + get_mmio_offset_from_peripheral_base());

    // Unmap our identity mapping
    // TODO: Dehardcode max mem size
    let mut ttbr0 =
        PageTableRoot::from_ptr(aarch64_cpu::registers::TTBR0_EL1.get_baddr(), page_size());
    for page in (0..0x100000000u64).step_by(0x40000000) {
        ttbr0.unmap_1gib_page(page);
    }
    util::clear_tlb();

    kprint!("Hello from kernel main");

    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}
