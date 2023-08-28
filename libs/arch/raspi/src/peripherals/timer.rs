use core::{hint, time::Duration};

use bitfield::BitRange;
use tock_registers::interfaces::Readable;

pub fn timer_freq() -> u32 {
    aarch64_cpu::registers::CNTFRQ_EL0.get().bit_range(31, 0)
}

pub fn timer_cycle_count() -> u64 {
    aarch64_cpu::registers::CNTPCT_EL0.get()
}

pub fn uptime() -> Duration {
    let uptime_ns: u64 = ((timer_cycle_count() as f64 / timer_freq() as f64) * 1e+9) as u64;
    Duration::from_nanos(uptime_ns)
}

pub fn duration_to_cycles(duration: Duration) -> u64 {
    (duration.as_secs_f64() * timer_freq() as f64) as u64
}

pub fn wait_for(time: Duration) {
    let end_cycles = timer_cycle_count() + duration_to_cycles(time);

    while timer_cycle_count() < end_cycles {
        hint::spin_loop();
    }
}
