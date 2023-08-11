#![no_std]
#![no_main]
#![feature(panic_info_message)]

pub mod memory;
pub mod peripherals;
pub mod util;

extern "C" {
    static __PG_SIZE: u8;
}
pub fn page_size() -> u64 {
    unsafe { (&__PG_SIZE as *const u8) as u64 }
}

use core::{arch::asm, ops::Deref};

use crate::peripherals::{MAILBOX, UART};
use generic_once_cell::Lazy;
use memory::frame_allocator::FrameAlloc;
use raspi_concurrency::mutex::{Mutex, RawMutex};
use raspi_exception::install_exception_handlers;
use raspi_memory::{
    memory_map::{EntryType, MemoryMap},
    page_table::{PageAlloc, PageTable},
};
use raspi_peripherals::get_mmio_offset_from_peripheral_base;

static FRAME_ALLOCATOR: Lazy<RawMutex, Mutex<FrameAlloc>> =
    Lazy::new(|| Mutex::new(FrameAlloc::new()));

fn invalidate_tlb() {
    unsafe {
        asm!("TLBI VMALLE1", "DSB ISH", "ISB");
    }
}

// Safety: At this point, assume the TTBR0 table has been totally wiped out
#[no_mangle]
pub extern "C" fn secondary_core_kmain(core_num: u64) -> ! {
    // TODO: When we jump to the kernel, we need some way to synchronize the cores to tell the kernel's
    // main thread that its able to reclaim bootloader memory
    kprints!(core_num, "Hello from secondary core!");
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_early_init(
    core_num: u64,
    memory_linear_map_start: u64,
    mem_map: *mut MemoryMap,
) -> ! {
    // Copy over the old memory map data before we reclaim the bootloader memory
    let mem_map_old: &MemoryMap = unsafe { &mut *mem_map };
    let map = mem_map_old.clone();
    let peripheral_start_addr = map
        .get_entries()
        .iter()
        .find(|x| x.entry_type == EntryType::Mmio)
        .unwrap()
        .base_addr;

    // Update our MMIO address to use higher half
    UART.lock().update_mmio_base(
        memory_linear_map_start + peripheral_start_addr + get_mmio_offset_from_peripheral_base(),
    );
    MAILBOX.lock().update_mmio_base(
        memory_linear_map_start + peripheral_start_addr + get_mmio_offset_from_peripheral_base(),
    );

    // Fork off the secondary cores
    if core_num != 0 {
        secondary_core_kmain(core_num);
    }
    kprintln!("Performing kernel early init...");

    install_exception_handlers();

    // Initialize a page frame allocator for the kernel
    for entry in map.get_entries() {
        match entry.entry_type {
            EntryType::Free => {
                for addr in (entry.base_addr..entry.end_addr).step_by(page_size() as usize) {
                    // If we fail to add a page to the free list, just silently ignore
                    let _ = FRAME_ALLOCATOR.lock().deallocate_frame(addr as *mut u8);
                }
            }
            _ => (),
        }
    }
    kprintln!(
        "Initialized page frame allocator with {} free frames",
        FRAME_ALLOCATOR.lock().num_free_frames()
    );

    // Wipe the identity-mapped page table
    let ttbr0 = PageTable::new(FRAME_ALLOCATOR.deref()).unwrap();
    aarch64_cpu::registers::TTBR0_EL1.set_baddr(ttbr0.as_raw_ptr() as u64);
    invalidate_tlb();

    kmain(ttbr0);
}

fn kmain(_ttbr0: PageTable<RawMutex, FrameAlloc>) -> ! {
    kprintln!("Kernel initialization complete");

    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}
