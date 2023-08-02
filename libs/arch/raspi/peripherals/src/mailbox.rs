use core::{hint, intrinsics::size_of};

use bitfield::{Bit, BitRangeMut};

use crate::{
    mmio::{mmio_read, mmio_write},
    MMIO,
};

const RESP_SUCCESS: u32 = 0x80000000;
const _RESP_FAIL: u32 = 0x80000001;
const MBOX_FULL_BIT: usize = 31;
const MBOX_EMPTY_BIT: usize = 30;

pub struct Mailbox {}

impl Mailbox {
    pub fn new() -> Self {
        Mailbox {}
    }

    pub fn send_message<T>(&self, mut msg: Message<T>) -> Result<Message<T>, ()> {
        let mmio_lock = MMIO.lock();

        // Ptr we receive must be aligned to 16 bytes
        let ptr = &mut msg as *mut Message<T>;
        assert!(ptr.is_aligned_to(16));

        // Last 4 bits must be set to channel num
        // TODO: The write register is 32 bits. But what if our address is 64 bits?
        let mut register_data = ptr as u32;
        register_data.set_bit_range(3, 0, 8);

        // Blocking request...
        while mmio_read(mmio_lock.mbox_status).bit(MBOX_FULL_BIT) {
            hint::spin_loop();
        }

        mmio_write(mmio_lock.mbox_wr, register_data as u32);

        // Wait until we've received a response...
        // TODO: Note that when we become multithreaded this might cause problems, as
        // we might get the "wrong" message back
        while mmio_read(mmio_lock.mbox_status).bit(MBOX_EMPTY_BIT) {
            hint::spin_loop();
        }

        if msg.code != RESP_SUCCESS {
            Err(())
        } else {
            Ok(msg)
        }
    }
}

#[repr(C, align(16))]
pub struct Message<T> {
    buf_size: u32,
    code: u32,
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
