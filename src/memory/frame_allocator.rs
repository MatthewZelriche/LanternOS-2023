use super::{map::MemoryMap, PAGE_SZ};

pub struct PageFrameAllocator<'a> {
    freelist: Option<&'a Node<'a>>,
    num_free: u64,
}

struct Node<'a> {
    next: Option<&'a Node<'a>>,
}

impl PageFrameAllocator<'_> {
    pub fn new(map: &MemoryMap) -> Self {
        let mut allocator = PageFrameAllocator {
            freelist: None,
            num_free: 0,
        };

        // Initialize all free pages in the map into the freelist
        for entry in map.get_entries() {
            match entry.entry_type {
                super::map::EntryType::Free => {
                    for addr in (entry.base_addr..entry.end_addr).step_by(PAGE_SZ as usize) {
                        // If we fail to add a page to the free list, just silently ignore
                        match allocator.append_free_page(addr as *mut u64) {
                            Ok(_) => allocator.num_free += 1,
                            Err(_) => (),
                        }
                    }
                }
                _ => (),
            }
        }

        allocator
    }

    pub fn num_free_pages(&self) -> u64 {
        self.num_free
    }

    fn append_free_page(&mut self, frame_addr: *mut u64) -> Result<(), ()> {
        // Can't use null address (0x0) as a valid page, even though ARM bare metal would
        // allow us to do so.
        if frame_addr.is_null() {
            return Err(());
        }

        // Appends to the front of the linked list
        let new_node = Some(Node {
            next: self.freelist,
        });

        // We store the freelist nodes "in-place". Each free page is by definition not being used
        // for anything else, so we can utilize it without having to allocate any additional memory to
        // store our free list.
        // This is sound because:
        // The start of a page is guarunteed to be properly aligned, since the smallest supported page
        // size is 4 KiB.
        // The pointer is the start of a free page, so we know that we aren't damaging any existing data
        // and so this pointer is always valid for writes.
        // We convert the pointer to a reference immediately after writing an Option to the pointer, so we
        // know the pointer can be converted to a &Option
        let addr_option_ptr = frame_addr as *mut Option<Node>;
        unsafe {
            core::ptr::write(addr_option_ptr, new_node);
            self.freelist = addr_option_ptr.as_ref().unwrap().as_ref();
        }

        Ok(())
    }
}
