#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(pointer_is_aligned)]
#![feature(int_roundings)]

mod memory;

use core::arch::global_asm;
use core::panic::PanicInfo;

use raspi_concurrency::spinlock::Spinlock;
use raspi_peripherals::mmio::Mmio;
use raspi_peripherals::MMIO;

use crate::memory::frame_allocator::PageFrameAllocator;
use crate::memory::map::MemoryMap;
use crate::memory::paging::{PageTableRoot, VirtualAddr};
use crate::memory::util::MemSize;
use crate::memory::{init_mmu::init_mmu, FRAME_ALLOCATOR, PAGE_SZ};

// Loads our entry point, _start, written entirely in assembly
global_asm!(include_str!("start.S"));

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            use raspi_peripherals::UART;
            writeln!(UART.lock(), $($arg)*).unwrap();
        }
    };
}

#[no_mangle]
pub extern "C" fn main(dtb_ptr: *const u8) {
    kprint!("Booting kernel in privilage mode EL1...");
    kprint!("");
    // Load the dtb
    // Panic if we can't load it, for now
    let map = MemoryMap::new(dtb_ptr).expect("Failed to construct memory map");
    kprint!("Page size:       {}", MemSize { bytes: PAGE_SZ });
    kprint!(
        "Reserved Pages:  {}",
        (map.get_total_mem() - map.get_free_mem()) / PAGE_SZ
    );
    kprint!(
        "Available Pages: {}",
        map.get_free_mem().to_bytes() / PAGE_SZ
    );
    kprint!("Total Memory:    {}", map.get_total_mem());
    kprint!("Avail Memory:    {}", map.get_free_mem());
    kprint!("");
    kprint!("{}", map);

    kprint!("Setting up page frame allocator...");
    // This early init runs with 1 core and IRQ disabled, so we dont have to worry about get().unwrap() to
    // this frame allocator ever failing
    FRAME_ALLOCATOR.get_or_init(|| Spinlock::new(PageFrameAllocator::new(&map)));
    kprint!(
        "Initialized page frame allocator with {} free pages",
        FRAME_ALLOCATOR.get().unwrap().lock().num_free_pages()
    );

    kprint!("Identity mapping all physical memory...");
    let mut page_table = PageTableRoot::new();
    // Identity map all of physical memory to 1GiB pages
    let max_addr = map.get_total_mem().to_bytes();
    for page in (0..max_addr).step_by(0x40000000) {
        page_table
            .map_1gib_page(page)
            .expect("Failed to Identity map full physical memory");
    }

    // Linearly map all MMIO to a second virtual location so we can specify it as
    // DEVICE memory
    // Uses 2MiB pages
    // TODO: Change the linear offset we are using
    let new_mmio_base: u64 = 0x1000000000;
    for page in (Mmio::MMIO_PHYS_BASE..Mmio::PERIPHERALS_PHYS_END).step_by(0x200000) {
        let virt_offset = page - Mmio::MMIO_PHYS_BASE;
        page_table
            .map_2mib_page(page, VirtualAddr(new_mmio_base + virt_offset))
            .expect("Failed to Identity map device memory");
    }

    // Turn on the MMU. From here on we are operating on virtual addresses
    // Safety: From here on, it's crucially important to no longer access the MMIO region
    // from the identity mapping, as the identity mapping is mapped to Normal cachable memory,
    // resulting in aliasing
    init_mmu(&page_table);

    // Set the MMIO base to our chosen virtual address
    MMIO.lock().set_base(new_mmio_base);
    kprint!("MMU initialized to identity mapping scheme");
    kprint!("Switched MMIO to secondary virtual address");

    // Never return from this diverging fn
    panic!("Reached end of kmain!")
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprint!("{}", _info);
    loop {}
}
