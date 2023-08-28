use core::{
    alloc::{Allocator, GlobalAlloc},
    ptr::{null_mut, NonNull},
};

use allocators::allocators::linked_list_allocator::LinkedListAlloc;
use generic_once_cell::OnceCell;
use raspi::concurrency::mutex::RawMutex;

pub mod frame_allocator;

pub struct GlobalAllocator(pub OnceCell<RawMutex, LinkedListAlloc<RawMutex>>);

#[global_allocator]
pub static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator(OnceCell::new());

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.0
            .get()
            .expect("Attempted an allocation without an initialized global allocator")
            .allocate(layout)
            .map_or(null_mut(), |x| x.as_ptr() as *mut u8)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.0
            .get()
            .expect("Attempted a deallocation without an initialized global allocator")
            .deallocate(
                NonNull::new(ptr).expect("Passed null ptr to global allocator"),
                layout,
            )
    }
}
