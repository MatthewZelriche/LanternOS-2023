//! A simple Global allocator that works only on a physical frame level
//!
//! This global allocator exists only to provide physical memory frames to PageTables. It cannot allocate
//! layouts greater than 1 frame size. This is to keep the bootloader lightweight and avoid having to
//! implement an entire heap for the bootloader.

use crate::{page_size, MEM_MAP};
use core::ptr::write_bytes;
use raspi_memory::{
    mem_size::MemSize,
    memory_map::{EntryType, MemoryMapEntry},
    page_frame_allocator::PageFrameAllocator,
};

// FrameAlloc is a very simple wrapper around our global frame allocator for use by the bootloader
// This ensures nobody can accidentally call PageFrameAllocator's alloc and dealloc functions directly,
// as they would go unnoticed by the bootloader's MemoryMap.
pub struct FrameAlloc(PageFrameAllocator);
impl FrameAlloc {
    pub fn new() -> Self {
        FrameAlloc(PageFrameAllocator::new(page_size()))
    }
    pub fn num_free_frames(&self) -> u64 {
        self.0.num_free_frames()
    }
}
impl raspi_memory::page_table::PageAlloc for FrameAlloc {
    fn allocate_frame(&mut self) -> Result<*mut u8, ()> {
        let frame = self
            .0
            .alloc_frame()
            .expect("Bootloader ran out of physical frames to allocate!")
            as *mut u8;

        unsafe {
            write_bytes(frame as *mut u8, 0, page_size() as usize);
        }

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
        let _ = self.0.free_frame(frame as *mut u64);
    }
}
