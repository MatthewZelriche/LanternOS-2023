// From: https://datasheets.raspberrypi.com/bcm2711/bcm2711-peripherals.pdf
// Used for reserving this range in the memory map
pub const PERIPHERALS_BASE: u64 = 0xFC000000;
pub const PERIPHERALS_END: u64 = 0x100000000;

//pub const MMIO_BASE: u32 = 0x3F000000; // Raspi3

pub const GPIO_BASE: u32 = MMIO_BASE + 0x200000;
pub const _GPPUD: u32 = GPIO_BASE + 0x94;
pub const _GPPUDCLK0: u32 = GPIO_BASE + 0x98;

pub const UART0_BASE: u32 = GPIO_BASE + 0x1000;
pub const UART_FR: u32 = UART0_BASE + 0x18;
pub const UART0_IBRD: u32 = UART0_BASE + 0x24;
pub const UART0_FBRD: u32 = UART0_BASE + 0x28;
pub const UART0_LCR_H: u32 = UART0_BASE + 0x2C;
pub const UART0_CR: u32 = UART0_BASE + 0x30;
pub const _UART0_IMSC: u32 = UART0_BASE + 0x38;
pub const _UART0_ICR: u32 = UART0_BASE + 0x44;

pub const MBOX_BASE: u32 = MMIO_BASE + 0xB880;
pub const _MBOX_RD: u32 = MBOX_BASE;
pub const MBOX_WR: u32 = MBOX_BASE + 0x20;
pub const MBOX_STATUS: u32 = MBOX_BASE + 0x18;

pub fn mmio_read(reg: u32) -> u32 {
    unsafe { core::intrinsics::volatile_load(reg as *const u32) }
}

pub fn mmio_write(reg: u32, val: u32) {
    unsafe { core::intrinsics::volatile_store(reg as *mut u32, val) }
}
