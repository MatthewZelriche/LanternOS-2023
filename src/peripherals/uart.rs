pub struct Uart;

use crate::mmio::{self, mmio_read, mmio_write, UART0_CR, UART0_FBRD, UART0_IBRD, UART0_LCR_H};
use bitfield::{Bit, BitMut};
use core::{fmt::Write, hint};

use super::{mailbox::SetClockRate, MAILBOX};

impl Uart {
    const INIT_RATE_DEF: u32 = 3000000;
    pub fn new() -> Self {
        let mut instance = Uart {};
        // Unwrap here since it would be fatal for this initialization to fail
        // before we have a functional way of outputting text
        // TODO: May be able to change this once we get framebuffer working
        instance.reset(Uart::INIT_RATE_DEF).unwrap();

        instance
    }

    pub fn reset(&mut self, clock_rate: u32) -> Result<(), ()> {
        // TODO: Currently untested due to lack of access to real hardware

        // Wait until buffered output is finished
        self.flush_fifo();

        // Shut down UART0
        mmio_write(UART0_CR, 0);

        // Set UART clock rate
        let msg = SetClockRate::new(2, clock_rate);
        let _ = MAILBOX.lock().send_message(msg)?;

        let baud: u32 = clock_rate / 16;
        let baud_whole: u32 = clock_rate / (16 * baud);
        let baud_frac: u32 = clock_rate % (16 * baud);
        mmio_write(UART0_IBRD, baud_whole);
        mmio_write(UART0_FBRD, baud_frac);

        let mut lcr: u32 = 0;
        lcr.set_bit(4, true);
        lcr.set_bit(5, true);
        lcr.set_bit(6, true);
        mmio_write(UART0_LCR_H, lcr);

        // Bring the UART0 back online
        let mut cr_data: u32 = 0;
        cr_data.set_bit(0, true);
        cr_data.set_bit(8, true);
        cr_data.set_bit(9, true);
        mmio_write(UART0_CR, cr_data);

        Ok(())
    }

    fn flush_fifo(&self) {
        while mmio_read(mmio::UART_FR).bit(5) {
            hint::spin_loop();
        }
    }

    pub fn send_byte(&mut self, byte: u8) {
        while mmio_read(mmio::UART_FR).bit(5) {
            hint::spin_loop();
        }
        mmio_write(mmio::UART0_BASE, byte as u32);
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
