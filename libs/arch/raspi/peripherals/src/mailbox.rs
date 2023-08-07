use core::{hint, intrinsics::size_of};

use bitfield::{Bit, BitRangeMut};

use crate::{
    mmio::{mmio_read, mmio_write},
    MMIO,
};

pub const RESP_SUCCESS: u32 = 0x80000000;
pub const RESP_FAIL: u32 = 0x80000001;
const MBOX_FULL_BIT: usize = 31;
const MBOX_EMPTY_BIT: usize = 30;

pub struct Mailbox {}

impl Mailbox {
    pub fn new() -> Self {
        Mailbox {}
    }

    /// Sends a message to the Raspberry Pi Mailbox, blocking until the message is processed
    ///
    /// msg_ptr must contain the 32 bit physical address of the message struct. It may be modified in-place
    /// in order for the Mailbox to provide result data. msg_ptr must also be aligned to a 16 byte
    /// boundary.
    ///
    /// # Panics
    /// Panics if the provided pointer is greater than 32 bits or is not aligned to a 16 bit boundary.
    ///
    pub fn send_message<T>(&self, msg_ptr: *mut Message<T>) {
        // The mailbox register is apparently strictly 32-bits size, so how can we send a physical address
        // above 4GiB? For now, just assert that the given address will fit
        assert!((msg_ptr as u64) < 0x100000000);
        assert!(msg_ptr.is_aligned_to(16));
        let mmio_lock = MMIO.lock();

        // Last 4 bits must be set to channel num
        let mut register_data = msg_ptr as u32;
        register_data.set_bit_range(3, 0, 8);

        // Blocking request...
        while mmio_read(mmio_lock.mbox_status).bit(MBOX_FULL_BIT) {
            hint::spin_loop();
        }

        mmio_write(mmio_lock.mbox_wr, register_data);

        // Wait until we've received a response...
        // TODO: Note that when we become multithreaded this might cause problems, as
        // we might get the "wrong" message back
        while mmio_read(mmio_lock.mbox_status).bit(MBOX_EMPTY_BIT) {
            hint::spin_loop();
        }
    }
}

#[repr(C, align(16))]
pub struct Message<T> {
    buf_size: u32,
    pub code: u32,
    tag: u32,
    pub data: T,
    null: u32,
}

#[repr(C, packed)]
pub struct GetArmMemory {
    buf_size: u32,
    response_size: u32,
    base: u32,
    size: u32,
}
impl GetArmMemory {
    pub fn new() -> Message<GetArmMemory> {
        Message {
            buf_size: size_of::<Message<GetArmMemory>>().try_into().unwrap(),
            code: 0,
            tag: 0x00010005,
            data: GetArmMemory {
                buf_size: 8,
                response_size: 0,
                base: 0,
                size: 0,
            },
            null: 0,
        }
    }
    pub fn get_base(&self) -> u32 {
        self.base
    }
    pub fn get_size(&self) -> u32 {
        self.size
    }
}

#[repr(C, packed)]
pub struct GetGpuMemory {
    buf_size: u32,
    response_size: u32,
    base: u32,
    size: u32,
}
impl GetGpuMemory {
    pub fn new() -> Message<GetGpuMemory> {
        Message {
            buf_size: size_of::<Message<GetGpuMemory>>().try_into().unwrap(),
            code: 0,
            tag: 0x00010006,
            data: GetGpuMemory {
                buf_size: 8,
                response_size: 0,
                base: 0,
                size: 0,
            },
            null: 0,
        }
    }
    pub fn get_base(&self) -> u32 {
        self.base
    }
    pub fn get_size(&self) -> u32 {
        self.size
    }
}

#[repr(C, packed)]
pub struct GetClockRate {
    buf_size: u32,
    response_size: u32,
    clock_id: u32,
    rate: u32,
}
impl GetClockRate {
    pub fn new(clock_id: u32) -> Message<GetClockRate> {
        Message {
            buf_size: size_of::<Message<GetClockRate>>().try_into().unwrap(),
            code: 0,
            tag: 0x00030002,
            data: GetClockRate {
                buf_size: 8,
                response_size: 0,
                clock_id,
                rate: 0,
            },
            null: 0,
        }
    }
    pub fn get_clock_id(&self) -> u32 {
        self.clock_id
    }
    pub fn get_rate(&self) -> u32 {
        self.rate
    }
}

#[repr(C, packed)]
pub struct GetClockRateMeasured {
    buf_size: u32,
    response_size: u32,
    clock_id: u32,
    rate: u32,
}
impl GetClockRateMeasured {
    pub fn new(clock_id: u32) -> Message<GetClockRateMeasured> {
        Message {
            buf_size: size_of::<Message<GetClockRateMeasured>>()
                .try_into()
                .unwrap(),
            code: 0,
            tag: 0x00030002,
            data: GetClockRateMeasured {
                buf_size: 8,
                response_size: 0,
                clock_id,
                rate: 0,
            },
            null: 0,
        }
    }
    pub fn get_clock_id(&self) -> u32 {
        self.clock_id
    }
    pub fn get_rate(&self) -> u32 {
        self.rate
    }
}

#[repr(C, packed)]
pub struct SetClockRate {
    buf_size: u32,
    response_size: u32,
    clock_id: u32,
    rate: u32,
    skip_turbo: u32,
}
impl SetClockRate {
    pub fn new(clock_id: u32, rate: u32) -> Message<SetClockRate> {
        Message {
            buf_size: size_of::<Message<SetClockRate>>().try_into().unwrap(),
            code: 0,
            tag: 0x00038002,
            data: SetClockRate {
                buf_size: 12,
                response_size: 0,
                clock_id,
                rate,
                skip_turbo: 0,
            },
            null: 0,
        }
    }

    pub fn get_rate(&self) -> u32 {
        self.rate
    }
}
