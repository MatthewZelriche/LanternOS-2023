use lock_api::GuardSend;

pub struct RawMutex(u64);

unsafe impl lock_api::RawMutex for RawMutex {
    const INIT: Self = RawMutex(0);

    type GuardMarker = GuardSend;

    fn lock(&self) {
        todo!()
    }

    fn try_lock(&self) -> bool {
        todo!()
    }

    unsafe fn unlock(&self) {
        todo!()
    }
}

pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutex, T>;
