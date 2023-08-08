//! A simple Global allocator that works only on a physical frame level
//!
//! This global allocator exists only to provide physical memory frames to PageTables. It cannot allocate
//! layouts greater than 1 frame size. This is to keep the bootloader lightweight and avoid having to
//! implement an entire heap for the bootloader.

use crate::page_size;
use core::alloc::GlobalAlloc;
use generic_once_cell::Lazy;
use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};
use raspi_memory::page_frame_allocator::PageFrameAllocator;

pub struct PageFrameAllocatorNewtype(pub Lazy<RawSpinlock, Spinlock<PageFrameAllocator>>);
unsafe impl GlobalAlloc for PageFrameAllocatorNewtype {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.align() > page_size() as usize || layout.size() > page_size() as usize {
            return core::ptr::null_mut();
        }

        self.0.lock().alloc_frame() as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if layout.align() > page_size() as usize || layout.size() > page_size() as usize {
            panic!("Cannot dealloc this memory block");
        }

        self.0
            .lock()
            .free_frame(ptr as *mut u64)
            .expect("Failed to free frame");
    }
}
#[global_allocator]
pub static FRAME_ALLOCATOR: PageFrameAllocatorNewtype =
    PageFrameAllocatorNewtype(Lazy::new(|| {
        Spinlock::new(PageFrameAllocator::new(page_size()))
    }));
