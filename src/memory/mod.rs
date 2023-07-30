pub mod frame_allocator;
pub mod map;
pub mod util;

// Size of pages, in bytes
pub const PAGE_SZ: u64 = 0x1000;

// TODO: Dehardcode this for other page sizes
pub fn get_page_addr(addr: u64) -> u64 {
    addr & !(PAGE_SZ - 1)
}
