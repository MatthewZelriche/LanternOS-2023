use core::ptr;

/// A physical page frame allocator implemented using a simple linked freelist.
///
/// The page frame allocator keeps a list of free pages of physical memory,
/// ensuring O(1) allocation and deallocation of individual page frames. The nodes of the
/// linked list are stored "in-place" inside the free pages themselves, so that no additional memory
/// allocations are necessary for bookkeeping.
///
/// # Safety
/// This page frame allocator uses raw pointers that point to nodes stored in free pages. This means the
/// linked list is stored across all of free memory. It is the user's responsibility to ensure that pages
/// stored in the freelist are never written to under any circumstances. It is also a requirement
/// that the struct always be accessed under some form of mutual exclusion, such as a mutex.
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
    /// Initializes a new, empty page frame allocator
    ///
    /// The page frame allocator starts with an empty freelist. After initialization, the user may
    /// add frames to the freelist with calls to ```free_frame```.
    pub fn new(page_size: u64) -> Self {
        PageFrameAllocator {
            freelist: ptr::null_mut(),
            num_free: 0,
            page_size,
        }
    }

    /// Retrieves a count of the frames currently stored in this Allocator's freelist.
    pub fn num_free_frames(&self) -> u64 {
        self.num_free
    }

    /// Deallocates a frame, returning it to the allocator's freelist.
    ///
    /// ```frame_addr``` must be the physical address to the start of a physical frame.
    /// Returns an Err if ```frame_addr``` is not aligned to the page boundary, or if it is null.
    /// This does mean that the first frame of physical memory cannot be assigned to an allocator.
    ///
    /// # Safety
    /// After freeing a frame with this method, it is the user's responsibility to ensure that the memory
    /// is not written to again until the frame is re-allocated with ```alloc_frame```
    pub fn free_frame(&mut self, frame_addr: *mut u64) -> Result<(), ()> {
        // Can't use null address (0x0) as a valid frame, even though ARM bare metal would
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

    /// Allocates a frame of physical memory from the freelist
    ///
    /// Returns the physical address of the start of a frame of memory.
    ///
    /// # Panics
    /// Panics if the freelist contains zero frames.
    pub fn alloc_frame(&mut self) -> *mut u64 {
        match self.freelist {
            ptr if ptr.is_null() => panic!("Page frame allocator out of usable frames!"),
            _ => {
                let frame = self.freelist as *mut u64;
                // Dereferencing this node is safe because the only way a frame of memory can exist
                // inside our freelist is if it has been added there by a call to free_frame, guarunteeing
                // that a valid node is there to be deferenced
                let next_node = unsafe { &(*(self.freelist)) }.next;
                self.freelist = next_node;
                self.num_free -= 1;
                frame
            }
        }
    }
}
