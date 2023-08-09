use lock_api::{GuardSend, RawMutex};

pub struct RawSpinlock(u64);

unsafe impl RawMutex for RawSpinlock {
    const INIT: Self = RawSpinlock(0);

    type GuardMarker = GuardSend;

    fn lock(&self) {
        // TODO
    }

    fn try_lock(&self) -> bool {
        // TODO
        true
    }

    unsafe fn unlock(&self) {
        // TODO
    }
}

pub type Spinlock<T> = lock_api::Mutex<RawSpinlock, T>;
pub type SpinlockGuard<'a, T> = lock_api::MutexGuard<'a, RawSpinlock, T>;
