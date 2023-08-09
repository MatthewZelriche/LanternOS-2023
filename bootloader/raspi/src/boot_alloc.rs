//! A simple Global allocator that works only on a physical frame level
//!
//! This global allocator exists only to provide physical memory frames to PageTables. It cannot allocate
//! layouts greater than 1 frame size. This is to keep the bootloader lightweight and avoid having to
//! implement an entire heap for the bootloader.

use super::memory_map::EntryType;
use crate::{mem_size::MemSize, memory_map::MemoryMapEntry, page_size, MEM_MAP};
use generic_once_cell::Lazy;
use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};
use raspi_memory::page_frame_allocator::PageFrameAllocator;

// FrameAlloc is a very simple wrapper around our global frame allocator for use by the bootloader
// This ensures nobody can accidentally call PageFrameAllocator's alloc and dealloc functions directly,
// as they would go unnoticed by the bootloader's MemoryMap.
pub struct FrameAlloc;
impl FrameAlloc {
    pub fn num_free_frames(&self) -> u64 {
        FRAME_ALLOCATOR.lock().num_free_frames()
    }
}
impl raspi_memory::page_table::PageAlloc for FrameAlloc {
    fn allocate_frame(&mut self) -> Result<*mut u8, ()> {
        let frame = FRAME_ALLOCATOR
            .lock()
            .alloc_frame()
            .expect("Bootloader ran out of physical frames to allocate!")
            as *mut u8;
        match MEM_MAP.get().unwrap().lock().add_entry(MemoryMapEntry {
            base_addr: frame as u64,
            size: MemSize { bytes: page_size() },
            end_addr: frame as u64 + page_size(),
            entry_type: EntryType::BLReserved,
        }) {
            Ok(_) => Ok(frame),
            Err(_) => Err(()),
        }
    }

    fn deallocate_frame(&mut self, frame: *mut u8) {
        let _ = FRAME_ALLOCATOR.lock().free_frame(frame as *mut u64);
    }
}

// Safety: Do not use this directly! Create a new FrameAlloc and use that instead, so we can keep track
// of whats been allocated in the memory map
static FRAME_ALLOCATOR: Lazy<RawSpinlock, Spinlock<PageFrameAllocator>> =
    Lazy::new(|| Spinlock::new(PageFrameAllocator::new(page_size())));
