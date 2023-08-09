#![no_std]
#![no_main]

pub mod peripherals;
pub mod util;

use core::alloc::GlobalAlloc;

use crate::peripherals::{MAILBOX, UART};

use raspi_memory::page_frame_allocator::PageFrameAllocator;
use raspi_peripherals::get_mmio_offset_from_peripheral_base;

#[repr(C)]
pub struct InitData {
    frame_allocator: PageFrameAllocator,
    mmio_start_addr: u64,
}

struct DummyAlloc;

unsafe impl GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        todo!()
    }
}

#[global_allocator]
static DUMMYALLOC: DummyAlloc = DummyAlloc {};

#[no_mangle]
pub extern "C" fn kernel_early_init(init_data: InitData) -> ! {
    // Update our MMIO address to use higher half
    UART.lock()
        .update_mmio_base(init_data.mmio_start_addr + get_mmio_offset_from_peripheral_base());
    MAILBOX
        .lock()
        .update_mmio_base(init_data.mmio_start_addr + get_mmio_offset_from_peripheral_base());
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
