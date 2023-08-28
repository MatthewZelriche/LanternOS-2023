use super::{get_default_mmio_base, mmio_read, mmio_write};
use bitfield::{Bit, BitMut};
use core::{fmt::Write, hint};

pub struct Uart {
    mmio_base: u64,
}

impl Uart {
    pub const INIT_RATE_DEF: u32 = 3000000;

    pub const UART0_BASE_OFFSET: u64 = 0x201000;
    pub const UART0_WRITE_OFFSET: u64 = Uart::UART0_BASE_OFFSET;
    pub const UART0_FR_OFFSET: u64 = Uart::UART0_BASE_OFFSET + 0x18;
    pub const UART0_IBRD_OFFSET: u64 = Uart::UART0_BASE_OFFSET + 0x24;
    pub const UART0_FBRD_OFFSET: u64 = Uart::UART0_BASE_OFFSET + 0x28;
    pub const UART0_LCR_H_OFFSET: u64 = Uart::UART0_BASE_OFFSET + 0x2c;
    pub const UART0_CR_OFFSET: u64 = Uart::UART0_BASE_OFFSET + 0x30;

    pub fn new() -> Self {
        let mut instance = Uart {
            mmio_base: get_default_mmio_base(),
        };
        // Unwrap here since it would be fatal for this initialization to fail
        // before we have a functional way of outputting text
        // TODO: May be able to change this once we get framebuffer working
        instance.reset(Uart::INIT_RATE_DEF).unwrap();

        instance
    }

    pub fn update_mmio_base(&mut self, mmio_base: u64) {
        self.mmio_base = mmio_base;
    }

    pub fn reset(&mut self, clock_rate: u32) -> Result<(), ()> {
        // TODO: Currently untested due to lack of access to real hardware

        // Wait until buffered output is finished
        self.flush_fifo();

        // Shut down UART0
        mmio_write(self.mmio_base + Uart::UART0_CR_OFFSET, 0);

        let baud: u32 = clock_rate / 16;
        let baud_whole: u32 = clock_rate / (16 * baud);
        let baud_frac: u32 = clock_rate % (16 * baud);
        mmio_write(self.mmio_base + Uart::UART0_IBRD_OFFSET, baud_whole);
        mmio_write(self.mmio_base + Uart::UART0_FBRD_OFFSET, baud_frac);

        let mut lcr: u32 = 0;
        lcr.set_bit(4, true);
        lcr.set_bit(5, true);
        lcr.set_bit(6, true);
        mmio_write(self.mmio_base + Uart::UART0_LCR_H_OFFSET, lcr);

        // Bring the UART0 back online
        let mut cr_data: u32 = 0;
        cr_data.set_bit(0, true);
        cr_data.set_bit(8, true);
        cr_data.set_bit(9, true);
        mmio_write(self.mmio_base + Uart::UART0_CR_OFFSET, cr_data);

        Ok(())
    }

    fn flush_fifo(&self) {
        while mmio_read(self.mmio_base + Uart::UART0_FR_OFFSET).bit(5) {
            hint::spin_loop();
        }
    }

    pub fn send_byte(&mut self, byte: u8) {
        while mmio_read(self.mmio_base + Uart::UART0_FR_OFFSET).bit(5) {
            hint::spin_loop();
        }
        mmio_write(self.mmio_base + Uart::UART0_WRITE_OFFSET, byte as u32);
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.send_byte(c as u8);
        }
        Result::Ok(())
    }
}
