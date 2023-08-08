#![no_std]

use core::ptr;

pub struct PageFrameAllocator {
    freelist: *mut Node,
    num_free: u64,
    page_size: u64,
}
unsafe impl Send for PageFrameAllocator {}
unsafe impl Sync for PageFrameAllocator {}

struct Node {
    next: *mut Node,
}

impl PageFrameAllocator {
    pub fn new(page_size: u64) -> Self {
        PageFrameAllocator {
            freelist: ptr::null_mut(),
            num_free: 0,
            page_size,
        }
    }

    pub fn num_free_pages(&self) -> u64 {
        self.num_free
    }

    pub fn free_page(&mut self, frame_addr: *mut u64) -> Result<(), ()> {
        // Can't use null address (0x0) as a valid page, even though ARM bare metal would
        // allow us to do so.
        if frame_addr.is_null() | (frame_addr as u64 % self.page_size != 0) {
            return Err(());
        }

        let new_node = Node {
            next: self.freelist,
        };

        // This is safe because we know dst is valid for writes, since by definition frame_adddr
        // is free memory and contains nothing of value that could be overwritten. The function also
        // verifies the address is aligned on a page boundary, ensuring any possible alignment requirement
        // is upheld before writing
        unsafe {
            core::ptr::write(frame_addr as *mut Node, new_node);
            self.freelist = frame_addr as *mut Node;
        }

        self.num_free += 1;
        Ok(())
    }

    pub fn alloc_page(&mut self) -> *mut u64 {
        match self.freelist {
            ptr if ptr.is_null() => panic!("Page allocator out of usable frames!"),
            _ => {
                let page = self.freelist as *mut u64;
                // Dereferencing this node is safe because the only way a page of memory can exist
                // inside our freelist is if it has been added there by a call to free_page, guarunteeing
                // that a valid node is there to be deferenced
                let next_node = unsafe { &(*(self.freelist)) }.next;
                self.freelist = next_node;
                page
            }
        }
    }
}
