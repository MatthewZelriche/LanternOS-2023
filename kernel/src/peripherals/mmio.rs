// From: https://datasheets.raspberrypi.com/bcm2711/bcm2711-peripherals.pdf
// Used for reserving this range in the memory map

#[derive(Default)]
pub struct Mmio {
    mmio_base: u64,

    pub gpio_base: u64,
    pub uart0_base: u64,
    pub uart_fr: u64,
    pub uart0_ibrd: u64,
    pub uart0_fbrd: u64,
    pub uart0_lcr_h: u64,
    pub uart0_cr: u64,
    pub mbox_base: u64,
    pub mbox_wr: u64,
    pub mbox_status: u64,
}

impl Mmio {
    pub const PERIPHERALS_PHYS_BASE: u64 = 0xFC000000;
    pub const PERIPHERALS_PHYS_END: u64 = 0x100000000;
    //pub const MMIO_PHYS_BASE: u64 = 0x3F000000; // Raspi3
    pub const MMIO_PHYS_BASE: u64 = 0xFE000000; // Raspi4

    pub fn new() -> Self {
        let mut mmio = Mmio::default();

        mmio.set_base(Mmio::MMIO_PHYS_BASE);

        mmio
    }

    pub fn set_base(&mut self, addr: u64) {
        self.mmio_base = addr;

        self.gpio_base = self.mmio_base + 0x200000;

        self.uart0_base = self.gpio_base + 0x1000;
        self.uart_fr = self.uart0_base + 0x18;
        self.uart0_ibrd = self.uart0_base + 0x24;
        self.uart0_fbrd = self.uart0_base + 0x28;
        self.uart0_lcr_h = self.uart0_base + 0x2c;
        self.uart0_cr = self.uart0_base + 0x30;

        self.mbox_base = self.mmio_base + 0xB880;
        self.mbox_status = self.mbox_base + 0x18;
        self.mbox_wr = self.mbox_base + 0x20;
    }
}

pub fn mmio_read(reg: u64) -> u32 {
    unsafe { core::intrinsics::volatile_load(reg as *const u32) }
}

pub fn mmio_write(reg: u64, val: u32) {
    unsafe { core::intrinsics::volatile_store(reg as *mut u32, val) }
}
