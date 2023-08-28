use super::mutex::Mutex;

pub struct Barrier {
    num_threads: u8,
    num_arrived: Mutex<u8>,
}

impl Barrier {
    pub fn new(num_threads: u8) -> Self {
        Barrier {
            num_threads,
            num_arrived: Mutex::new(0),
        }
    }

    pub fn wait(&self) {
        {
            let lock = self.num_arrived.lock();
            if *lock == self.num_threads {
                panic!("Attempt to reuse a single use barrier!");
            }
        }

        {
            *self.num_arrived.lock() += 1;
        }

        loop {
            let arrived = self.num_arrived.lock();
            if *arrived == self.num_threads {
                break;
            }
        }
    }
}
