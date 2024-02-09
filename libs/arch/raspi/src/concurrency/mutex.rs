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
                "ldr x9, =1",
                "LDXR x10, [{lock_ptr}]",
                "cmp x10, #1",                  // Test if the mutex is currently locked
                "beq 3f",                       // If locked, wait until we can try again later
                "stxr w11, x9, [{lock_ptr}]",   // Otherwise, attempt a lock
                "cmp w11, #0",                  // Did the lock succeed?
                "beq 4f",                       // Then jump to the end
                "",
                "3:",                           // Couldn't attain a lock...
                "wfe",                          // Sleep CPU until we can try again
                "b 2b",
                "4:",                           // Successful lock, prevent re-ordering & exit
                "DMB SY",
                lock_ptr = in(reg) (&self.0) as *const u64,
            );
        }
    }

    fn try_lock(&self) -> bool {
        let mut res: u32 = 1;
        unsafe {
            asm!(
                "ldr x9, =1",
                "LDXR x10, [{lock_ptr}]",
                "cmp x10, #1",                  // Test if the mutex is currently locked
                "beq 2f",                       // If locked, wait until we can try again later
                "stxr w15, x9, [{lock_ptr}]",   // Otherwise, attempt a lock
                "mov {res:w}, w15",             // Did we succeed at getting the lock?
                "DMB SY",
                "2:",
                lock_ptr = in(reg) (&self.0) as *const u64,
                res = inout(reg) res,
            );
        }

        res == 0
    }

    unsafe fn unlock(&self) {
        unsafe {
            asm!(
            "DMB SY",
            "ldr x9, =0",
            "str x9, [{lock_ptr}]",             // Unlock the mutex.
            "sev",                              // Inform others that they can attempt to
                                                // lock the mutex again
            lock_ptr = in(reg) (&self.0) as *const u64,
            );
        }
    }
}

pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutex, T>;
