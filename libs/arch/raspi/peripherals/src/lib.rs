#![feature(pointer_is_aligned)]
#![feature(core_intrinsics)]
#![no_std]

use bitfield::BitRange;
use tock_registers::interfaces::Readable;

pub struct ConstantsRaspi3;
impl ConstantsRaspi3 {
    pub const MMIO_PHYS_BASE: u64 = 0x3F000000; // Raspi3
    pub const PERIPHERALS_PHYS_BASE: u64 = 0x3F000000; // Raspi3
    pub const PERIPHERALS_PHYS_END: u64 = 0x40000000; //Raspi3
    pub const MMIO_OFFSET: u64 =
        ConstantsRaspi3::MMIO_PHYS_BASE - ConstantsRaspi3::PERIPHERALS_PHYS_BASE; // Raspi3
}

pub struct ConstantsRaspi4;
impl ConstantsRaspi4 {
    pub const PERIPHERALS_PHYS_BASE: u64 = 0xFC000000; // Raspi4
    pub const PERIPHERALS_PHYS_END: u64 = 0x100000000; // Raspi4
    pub const MMIO_PHYS_BASE: u64 = 0xFE000000; // Raspi4
    pub const MMIO_OFFSET: u64 =
        ConstantsRaspi4::MMIO_PHYS_BASE - ConstantsRaspi4::PERIPHERALS_PHYS_BASE; // Raspi4
}

pub fn get_default_mmio_base() -> u64 {
    let val = aarch64_cpu::registers::MIDR_EL1.get();
    let partno: u64 = val.bit_range(15, 4);

    match partno {
        0xD03 => ConstantsRaspi3::MMIO_PHYS_BASE,
        0xD08 => ConstantsRaspi4::MMIO_PHYS_BASE,
        _ => panic!("Unsupported board type"),
    }
}

pub fn get_board_peripheral_range() -> (u64, u64) {
    let val = aarch64_cpu::registers::MIDR_EL1.get();
    let partno: u64 = val.bit_range(15, 4);

    match partno {
        0xD03 => (
            ConstantsRaspi3::PERIPHERALS_PHYS_BASE,
            ConstantsRaspi3::PERIPHERALS_PHYS_END,
        ),
        0xD08 => (
            ConstantsRaspi4::PERIPHERALS_PHYS_BASE,
            ConstantsRaspi4::PERIPHERALS_PHYS_END,
        ),
        _ => panic!("Unsupported board type"),
    }
}

pub fn get_mmio_offset_from_peripheral_base() -> u64 {
    let val = aarch64_cpu::registers::MIDR_EL1.get();
    let partno: u64 = val.bit_range(15, 4);

    match partno {
        0xD03 => ConstantsRaspi3::MMIO_OFFSET,
        0xD08 => ConstantsRaspi4::MMIO_OFFSET,
        _ => panic!("Unsupported board type"),
    }
}

fn mmio_read(reg: u64) -> u32 {
    unsafe { core::intrinsics::volatile_load(reg as *const u32) }
}

fn mmio_write(reg: u64, val: u32) {
    unsafe { core::intrinsics::volatile_store(reg as *mut u32, val) }
}

#[allow(dead_code)]
pub mod mailbox;
pub mod uart;
