#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(allocator_api)]
#![feature(int_roundings)]

pub mod memory;
pub mod peripherals;
pub mod util;

extern crate alloc;

extern "C" {
    static __PG_SIZE: u8;
    static __KERNEL_VIRT_END: u8;
}
pub fn page_size() -> u64 {
    unsafe { (&__PG_SIZE as *const u8) as u64 }
}

pub fn kernel_virt_end() -> u64 {
    unsafe { (&__KERNEL_VIRT_END as *const u8) as u64 }
}

use crate::{
    memory::GLOBAL_ALLOCATOR,
    peripherals::{EMMC2, MAILBOX, UART},
    util::clear_tlb,
};
use aarch64_cpu::registers;
use allocators::allocators::linked_list_allocator::LinkedListAlloc;
use generic_once_cell::Lazy;
use memory::frame_allocator::FrameAlloc;
use raspi_concurrency::{
    barrier::Barrier,
    mutex::{Mutex, RawMutex},
};
use raspi_exception::install_exception_handlers;
use raspi_memory::{
    memory_map::{EntryType, MemoryMap},
    page_table::{Lvl0TableDescriptor, MemoryType, PageAlloc, PageTable, VirtualAddr},
};
use raspi_peripherals::{
    emmc::{EMMCController, SdResult},
    get_mmio_offset_from_peripheral_base,
    timer::uptime,
};

static FRAME_ALLOCATOR: Lazy<RawMutex, Mutex<FrameAlloc>> =
    Lazy::new(|| Mutex::new(FrameAlloc::new()));

static BARRIER: Lazy<RawMutex, Barrier> = Lazy::new(|| Barrier::new(4));

// Safety: At this point, assume the TTBR0 table has been totally wiped out
#[no_mangle]
pub extern "C" fn secondary_core_kmain(core_num: u64) -> ! {
    // Safe to unwrap here because we know the barrier won't be "consumed" until after
    // the barrier synchronizes
    BARRIER.wait();
    kprints!(core_num, "Hello from secondary core!");
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_early_init(
    core_num: u64,
    memory_linear_map_start: u64,
    kernel_end: u64,
    mem_map: *mut MemoryMap,
) -> ! {
    // Fork off the secondary cores
    if core_num != 0 {
        secondary_core_kmain(core_num);
    }

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
    let _ = EMMC2.set(Mutex::new(unsafe { EMMCController::new(0xFE300000) }));

    kprintln!("Performing kernel early init...");

    let addr = install_exception_handlers();
    kprintln!("Registered exception handlers at {:#x}", addr);

    // Initialize a page frame allocator for the kernel
    for entry in map.get_entries() {
        match entry.entry_type {
            // Also free Bootloader memory as its no longer needed
            EntryType::Bootloader | EntryType::Free => {
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

    // TODO: Might want to consider lazy loading of memory into the kernel heap,
    // via some kind of kmmap call. This would also allow resizable kernel heap.
    let mut ttbr1 = unsafe {
        PageTable::from_raw_ptr(
            registers::TTBR1_EL1.get_baddr() as *const Lvl0TableDescriptor,
            &FRAME_ALLOCATOR,
        )
    };

    // Initialize kernel heap:
    let kernel_heap_start = kernel_end;
    let kernel_heap_end = kernel_heap_start + 0x200000;
    for virt_page in (kernel_heap_start..kernel_heap_end).step_by(page_size() as usize) {
        let phys_page = FRAME_ALLOCATOR
            .lock()
            .allocate_frame()
            .expect("Failed to allocate memory for kernel heap") as u64;
        ttbr1
            .map_page(
                phys_page,
                VirtualAddr(virt_page),
                MemoryType::NORMAL_CACHEABLE,
            )
            .expect("Failed to map memory for kernel heap");
    }
    GLOBAL_ALLOCATOR
        .0
        .set(unsafe {
            LinkedListAlloc::<RawMutex>::new(
                kernel_heap_start as *mut u8,
                kernel_heap_end as *mut u8,
            )
        })
        .expect("Failed to initialize heap allocator!");
    kprintln!(
        "Initialized kernel heap at address range {:#x} - {:#x}",
        kernel_heap_start,
        kernel_heap_end
    );

    let init_res = EMMC2.get().unwrap().lock().emmc_init_card();
    if init_res != SdResult::EMMC_OK {
        panic!("Failed to initialize SD Card with error: {:?}", init_res);
    } else {
        kprintln!("Initialized EMMC2 driver. Storage medium ready to receive block requests.");
    }

    // Wipe the identity-mapped page table
    let ttbr0 = PageTable::new(&FRAME_ALLOCATOR).unwrap();
    registers::TTBR0_EL1.set_baddr(ttbr0.as_raw_ptr() as u64);
    clear_tlb();

    // TODO: Synchronize with secondary threads
    kprintln!("Kernel initialization complete");
    BARRIER.wait();
    kmain(ttbr0);
}

fn kmain(_ttbr0: PageTable<RawMutex, FrameAlloc>) -> ! {
    // Never return from this diverging fn
    loop {}
}
