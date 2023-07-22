pub const MMIO_BASE: u64 = 0xFE000000;
pub const GPIO_BASE: u64 = MMIO_BASE + 0x200000;
pub const GPPUD: u64 = GPIO_BASE + 0x94;
pub const GPPUDCLK0: u64 = GPIO_BASE + 0x98;
pub const UART0_BASE: u64 = GPIO_BASE + 0x1000;
pub const UART_FR: u64 = UART0_BASE + 0x18;
pub const UART0_CR: u64 = UART0_BASE + 0x30;
pub const UART0_ICR: u64 = UART0_BASE + 0x44;

pub fn mmio_read(reg: u64) -> u64 {
    unsafe { core::intrinsics::volatile_load(reg as *const u64) }
}

pub fn mmio_write(reg: u64, val: u64) {
    unsafe { core::intrinsics::volatile_store(reg as *mut u64, val) }
}
