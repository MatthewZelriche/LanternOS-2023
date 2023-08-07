#![feature(pointer_is_aligned)]
#![feature(core_intrinsics)]
#![no_std]

pub const PERIPHERALS_PHYS_BASE: u64 = 0xFC000000;
pub const PERIPHERALS_PHYS_END: u64 = 0x100000000;
//pub const MMIO_PHYS_BASE: u64 = 0x3F000000; // Raspi3
pub const MMIO_PHYS_BASE: u64 = 0xFE000000; // Raspi4

fn mmio_read(reg: u64) -> u32 {
    unsafe { core::intrinsics::volatile_load(reg as *const u32) }
}

fn mmio_write(reg: u64, val: u32) {
    unsafe { core::intrinsics::volatile_store(reg as *mut u32, val) }
}

#[allow(dead_code)]
pub mod mailbox;
pub mod uart;
