#![no_std]
#![no_main]

pub mod peripherals;
pub mod util;

use crate::peripherals::{MAILBOX, UART};
use raspi_peripherals::get_mmio_offset_from_peripheral_base;

#[no_mangle]
pub extern "C" fn kernel_early_init(mmio_start_addr: u64) -> ! {
    // Update our MMIO address to use higher half
    UART.lock()
        .update_mmio_base(mmio_start_addr + get_mmio_offset_from_peripheral_base());
    MAILBOX
        .lock()
        .update_mmio_base(mmio_start_addr + get_mmio_offset_from_peripheral_base());
    kprint!("Performing kernel early init...");

    kmain();
}

/*
 * By the time we reach this fn, the bootloader has already configured the UART clock rate and baud rate.
 * There's no need to send a mailbox message here; it would be redundant
 */
fn kmain() -> ! {
    kprint!("Kernel initialization complete");
    /*
    // Unmap our identity mapping
    // TODO: Dehardcode max mem size
    let mut ttbr0 = unsafe {
        PageTable::from_raw(
            aarch64_cpu::registers::TTBR0_EL1.get_baddr() as *mut Lvl0TableDescriptor,
            page_size(),
        )
    };
    for page in (0..0x100000000u64).step_by(0x40000000) {
        ttbr0.unmap_1gib_page(VirtualAddr(page));
    }
    util::clear_tlb();
    */

    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}
