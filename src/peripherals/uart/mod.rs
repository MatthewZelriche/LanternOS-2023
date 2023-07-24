pub struct Uart;

use crate::mmio::{self, mmio_read, mmio_write};
use bitfield::Bit;
use core::{
    fmt::{Result, Write},
    hint,
};

impl Uart {
    pub fn new() -> Self {
        let instance = Uart {};
        instance.flush_fifo();

        // TODO: proper init

        instance
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
        mmio_write(mmio::UART0_BASE, byte as u64);
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
