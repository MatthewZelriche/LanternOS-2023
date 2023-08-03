#![no_std]

pub struct PageFrameAllocator<'a> {
    freelist: Option<&'a Node<'a>>,
    num_free: u64,
    page_size: u64,
}

struct Node<'a> {
    next: Option<&'a Node<'a>>,
}

impl PageFrameAllocator<'_> {
    pub fn new(page_size: u64) -> Self {
        PageFrameAllocator {
            freelist: None,
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
        assert!(frame_addr as u64 % self.page_size == 0);
        if frame_addr.is_null() {
            return Err(());
        }

        let new_node = Node {
            next: self.freelist,
        };

        // We store the freelist nodes "in-place". Each free page is by definition not being used
        // for anything else, so we can utilize it without having to allocate any additional memory to
        // store our free list.
        // This is sound because:
        // The start of a page is guarunteed to be properly aligned, since the smallest supported page
        // size is 4 KiB.
        // The pointer is the start of a free page, so we know that we aren't damaging any existing data
        // and so this pointer is always valid for writes.
        // We convert the pointer to a reference immediately after writing a Node to the pointer, so we
        // know the pointer can be converted to a &Node
        let addr_option_ptr = frame_addr as *mut Node;
        unsafe {
            core::ptr::write(addr_option_ptr, new_node);
            self.freelist = Some(addr_option_ptr.as_ref().unwrap());
        }

        self.num_free += 1;
        Ok(())
    }

    pub fn alloc_page(&mut self) -> *mut u64 {
        match self.freelist {
            Some(head) => {
                let addr = head as *const Node as *mut u64;
                self.freelist = head.next;
                self.num_free -= 1;
                addr
            }
            None => panic!("Page Frame Allocator ran out of physical memory"),
        }
    }
}
