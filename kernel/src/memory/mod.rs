use generic_once_cell::OnceCell;

use raspi_concurrency::spinlock::{RawSpinlock, Spinlock};

use self::frame_allocator::PageFrameAllocator;

pub mod frame_allocator;
pub mod init_mmu;
pub mod map;
pub mod paging;
pub mod util;

pub static FRAME_ALLOCATOR: OnceCell<RawSpinlock, Spinlock<PageFrameAllocator>> = OnceCell::new();

// Size of pages, in bytes
pub const PAGE_SZ: u64 = 0x1000;

// TODO: Dehardcode this for other page sizes
pub fn get_page_addr(addr: u64) -> u64 {
    addr & !(PAGE_SZ - 1)
}
