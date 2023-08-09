//! A simple Global allocator that works only on a physical frame level
//!
//! This global allocator exists only to provide physical memory frames to PageTables. It cannot allocate
//! layouts greater than 1 frame size. This is to keep the bootloader lightweight and avoid having to
//! implement an entire heap for the bootloader.

use crate::page_size;
use generic_once_cell::Lazy;
use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};
use raspi_memory::page_frame_allocator::PageFrameAllocator;

pub struct FrameAlloc;
impl raspi_memory::page_table::PageAlloc for FrameAlloc {
    fn allocate_frame(&mut self) -> Result<*mut u8, ()> {
        Ok(FRAME_ALLOCATOR.lock().alloc_frame() as *mut u8)
    }

    fn deallocate_frame(&mut self, frame: *mut u8) {
        let _ = FRAME_ALLOCATOR.lock().free_frame(frame as *mut u64);
    }
}

pub static FRAME_ALLOCATOR: Lazy<RawSpinlock, Spinlock<PageFrameAllocator>> =
    Lazy::new(|| Spinlock::new(PageFrameAllocator::new(page_size())));
