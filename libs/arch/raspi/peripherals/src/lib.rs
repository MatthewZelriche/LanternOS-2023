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
        ConstantsRaspi3::MMIO_PHYS_BASE - ConstantsRaspi3::PERIPHERALS_PHYS_BASE;
    pub const EMMC_OFFSET_FROM_MMIO_BASE: u64 = 0x300000;
}

pub struct ConstantsRaspi4;
impl ConstantsRaspi4 {
    pub const PERIPHERALS_PHYS_BASE: u64 = 0xFC000000; // Raspi4
    pub const PERIPHERALS_PHYS_END: u64 = 0x100000000; // Raspi4
    pub const MMIO_PHYS_BASE: u64 = 0xFE000000; // Raspi4
    pub const MMIO_OFFSET: u64 =
        ConstantsRaspi4::MMIO_PHYS_BASE - ConstantsRaspi4::PERIPHERALS_PHYS_BASE;
    // Sourced from: https://github.com/librerpi/rpi-open-firmware/blob/master/docs/rpi4-gpio-mux.dot
    // Seems very poorly documented
    pub const EMMC_OFFSET_FROM_MMIO_BASE: u64 = 0x340000;
}

pub enum Board {
    RPI3,
    RPI4,
    UNSUPPORTED,
}

pub fn get_board() -> Board {
    let val = aarch64_cpu::registers::MIDR_EL1.get();
    let partno: u64 = val.bit_range(15, 4);

    match partno {
        0xD03 => Board::RPI3,
        0xD08 => Board::RPI4,
        _ => Board::UNSUPPORTED,
    }
}

pub fn get_default_mmio_base() -> u64 {
    match get_board() {
        Board::RPI3 => ConstantsRaspi3::MMIO_PHYS_BASE,
        Board::RPI4 => ConstantsRaspi4::MMIO_PHYS_BASE,
        Board::UNSUPPORTED => panic!("Unsupported board type"),
    }
}

pub fn get_board_peripheral_range() -> (u64, u64) {
    match get_board() {
        Board::RPI3 => (
            ConstantsRaspi3::PERIPHERALS_PHYS_BASE,
            ConstantsRaspi3::PERIPHERALS_PHYS_END,
        ),
        Board::RPI4 => (
            ConstantsRaspi4::PERIPHERALS_PHYS_BASE,
            ConstantsRaspi4::PERIPHERALS_PHYS_END,
        ),
        Board::UNSUPPORTED => panic!("Unsupported board type"),
    }
}

pub fn get_mmio_offset_from_peripheral_base() -> u64 {
    match get_board() {
        Board::RPI3 => ConstantsRaspi3::MMIO_OFFSET,
        Board::RPI4 => ConstantsRaspi4::MMIO_OFFSET,
        Board::UNSUPPORTED => panic!("Unsupported board type"),
    }
}

fn get_emmc_offset_from_mmio_base() -> u64 {
    match get_board() {
        Board::RPI3 => ConstantsRaspi3::EMMC_OFFSET_FROM_MMIO_BASE,
        Board::RPI4 => ConstantsRaspi4::EMMC_OFFSET_FROM_MMIO_BASE,
        Board::UNSUPPORTED => panic!("Unsupported board type"),
    }
}

fn mmio_read(reg: u64) -> u32 {
    unsafe { core::intrinsics::volatile_load(reg as *const u32) }
}

fn mmio_write(reg: u64, val: u32) {
    unsafe { core::intrinsics::volatile_store(reg as *mut u32, val) }
}

pub mod emmc;
pub mod mailbox;
pub mod timer;
pub mod uart;
