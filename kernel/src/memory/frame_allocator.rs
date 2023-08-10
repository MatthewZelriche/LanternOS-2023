use crate::page_size;
use core::ptr::write_bytes;
use raspi_memory::page_frame_allocator::PageFrameAllocator;

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
        Ok(frame)
    }

    fn deallocate_frame(&mut self, frame: *mut u8) {
        let _ = self.0.free_frame(frame as *mut u64);
    }
}
