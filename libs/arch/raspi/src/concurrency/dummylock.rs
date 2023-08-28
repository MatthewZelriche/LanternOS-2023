use lock_api::{GuardSend, RawMutex};

pub struct RawDummylock(u64);

unsafe impl RawMutex for RawDummylock {
    const INIT: Self = RawDummylock(0);

    type GuardMarker = GuardSend;

    fn lock(&self) {
        // intentionally empty
    }

    fn try_lock(&self) -> bool {
        // intentionally empty
        true
    }

    unsafe fn unlock(&self) {
        // intentionally empty
    }
}

pub type Dummylock<T> = lock_api::Mutex<RawDummylock, T>;
pub type DummylockGuard<'a, T> = lock_api::MutexGuard<'a, RawDummylock, T>;
