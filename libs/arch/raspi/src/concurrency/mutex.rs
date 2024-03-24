use core::arch::asm;

use lock_api::GuardSend;

#[derive(Debug)]
pub struct RawMutex(u64);

unsafe impl lock_api::RawMutex for RawMutex {
    const INIT: Self = RawMutex(0);
    type GuardMarker = GuardSend;

    fn lock(&self) {
        unsafe {
            asm!(
                "2:",
                "ldaxr {temp}, [{lock_ptr}]",   // Read lock variable, setting exclusive monitor
                "cmp {temp}, #1",   // Is this mutex already locked? Then try again...
                "beq 2b",
                "ldr {temp}, =1",   // Mutex was free during ldaxr, now we try to update the lock exclusively
                "stlxr {res:w}, {temp}, [{lock_ptr}]",
                "cmp {res:w}, #0",  // If res is 0, someone else mucked with this memory address since we called ldaxr!
                                    // The store failed and we will have to try again...
                "bne 2b",
                temp = in(reg) 123,
                lock_ptr = in(reg) (&(self.0)) as *const u64,
                res = in(reg) 0,
            );
        }
    }

    fn try_lock(&self) -> bool {
        let mut lock_succeed = 1; // Zero means true in this context
        unsafe {
            asm!(
                "ldaxr {temp}, [{lock_ptr}]",   // Read lock variable, setting exclusive monitor
                "cmp {temp}, #1",   // Is this mutex already locked? Bail out and return false
                "beq 2f",
                "ldr {temp}, =1",   // Try to update the lock exclusively...
                "stlxr {res:w}, {temp}, [{lock_ptr}]",  // Set lock_succeed to the value of res
                                                        // If res is nonzero, we failed to update the lock...
                "2:",
                temp = in(reg) 123,
                lock_ptr = in(reg) (&(self.0)) as *const u64,
                res = inout(reg) lock_succeed,
            );
        }
        lock_succeed == 0
    }

    unsafe fn unlock(&self) {
        unsafe {
            asm!(
                "2:",
                "ldaxr {temp}, [{lock_ptr}]", // Set the exclusive monitor
                                              // Note that we don't actually care what the value of lock_ptr is
                                              // This is because its already required by lock_api that an unlock
                                              // always be paired with a corresponding lock. So this should never
                                              // be called if the lock is free. Either way, unlock should always
                                              // set the mutex to free regardless. (Going from free -> free is harmless)
                "ldr {temp}, =0",
                "stlxr {res:w}, {temp}, [{lock_ptr}]",  // Attempt to free the mutex
                "cmp {res:w}, #0",  // We may have tried to free the mutex at the moment someone else was trying to lock it
                                    // This will muck up both attempts to modify this address and both will fail.
                                    // This means in order to properly unlock this mutex we have to try again, until the exclusive monitor
                                    // informs us that we have successfully reset this address to 0.
                "bne 2b",
                temp = in(reg) 123,
                lock_ptr = in(reg) (&(self.0)) as *const u64,
                res = in(reg) 0,
            );
        }
    }
}

pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutex, T>;
