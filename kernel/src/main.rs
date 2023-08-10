#![no_std]
#![no_main]

pub mod peripherals;
pub mod util;

use crate::peripherals::{MAILBOX, UART};
use generic_once_cell::OnceCell;
use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};
use raspi_memory::memory_map::{EntryType, MemoryMap};
use raspi_peripherals::get_mmio_offset_from_peripheral_base;

static MEM_MAP: OnceCell<RawSpinlock, Spinlock<MemoryMap>> = OnceCell::new();

#[no_mangle]
pub extern "C" fn kernel_early_init(memory_linear_map_start: u64, mem_map: *mut MemoryMap) -> ! {
    // Copy over the old memory map data before we reclaim the bootloader memory
    let mem_map_old: &MemoryMap = unsafe { &mut *mem_map };
    let map_mutex = MEM_MAP.get_or_init(|| Spinlock::new(mem_map_old.clone()));
    let peripheral_start_addr = map_mutex
        .lock()
        .get_entries()
        .iter()
        .find(|x| x.entry_type == EntryType::Mmio)
        .unwrap()
        .base_addr;

    // Update our MMIO address to use higher half
    UART.lock()
        .update_mmio_base(peripheral_start_addr + get_mmio_offset_from_peripheral_base());
    MAILBOX
        .lock()
        .update_mmio_base(peripheral_start_addr + get_mmio_offset_from_peripheral_base());
    kprint!("Performing kernel early init...");

    kmain();
}

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
