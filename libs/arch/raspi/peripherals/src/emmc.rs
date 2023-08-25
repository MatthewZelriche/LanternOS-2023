use core::hint;

use bitfield::{Bit, BitMut};

use crate::{
    get_default_mmio_base, get_emmc_offset_from_mmio_base, mmio_read, mmio_write, uart::Uart,
};

pub struct Emmc {
    mmio_base: u64,
    emmc_base: u64,
}

impl Emmc {
    pub const ARG1_OFFSET: u64 = 0x8;
    pub const CMDTM_OFFSET: u64 = 0xc;
    pub const CONTROL0_OFFSET: u64 = 0x28;
    pub const CONTROL1_OFFSET: u64 = 0x2c;

    pub fn new() -> Self {
        let mut this = Emmc {
            mmio_base: get_default_mmio_base(),
            emmc_base: get_default_mmio_base() + get_emmc_offset_from_mmio_base(),
        };

        this.reset();
        this
    }

    pub fn update_mmio_base(&mut self, mmio_base: u64) {
        self.mmio_base = mmio_base;
        self.emmc_base = self.mmio_base + get_emmc_offset_from_mmio_base();
    }

    pub fn reset(&mut self) {}
}
