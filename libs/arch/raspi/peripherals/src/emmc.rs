use core::{hint, time::Duration};

use bitfield::{Bit, BitMut, BitRangeMut};

use crate::{
    get_board, get_default_mmio_base, get_emmc_offset_from_mmio_base,
    mailbox::{GetClockRate, Mailbox, Message},
    mmio_read, mmio_write,
    timer::wait_for,
    uart::Uart,
    Board,
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

    pub fn reset(&mut self) {
        // Write in a reset request
        let mut control1 = mmio_read(self.emmc_base + Emmc::CONTROL1_OFFSET);
        control1.set_bit(24, true);
        mmio_write(self.emmc_base + Emmc::CONTROL1_OFFSET, control1);

        // Wait for reset to complete...
        while mmio_read(self.emmc_base + Emmc::CONTROL1_OFFSET).bit(24) {
            hint::spin_loop();
        }

        // RPI4 requires special initialization
        // https://forums.raspberrypi.com/viewtopic.php?t=308089#p1845707
        // I cannot find ANY official documentation regarding the bits that get set.
        // They are listed as reserved in the BCM2835 documentation, which according to the
        // docs suggests they are defined by Arasan and would theoretically be documented in
        // SD3.0_Host_AHB_eMMC4.4_Usersguide_ver5.9_jan11_10.pdf, but supposedly this document
        // is not publically available....
        // So where rst on the raspberrypi forums got bits from 8-11 from is anybody's guess...
        if get_board() == Board::RPI4 {
            let mut control0 = mmio_read(self.emmc_base + Emmc::CONTROL0_OFFSET);
            control0.set_bit_range(11, 8, 0b1111);
            mmio_write(self.emmc_base + Emmc::CONTROL0_OFFSET, control0);

            // Max delay is specified as 1ms in SD Card Physical Spec
            // Give a little extra room to be safe
            wait_for(Duration::from_micros(1500));
        }
    }
}
