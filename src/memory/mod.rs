pub mod map;
pub mod util;

// Size of pages, in bytes
const PAGE_SZ: u64 = 0x1000;

pub fn get_page_addr(addr: u64) -> u64 {
    addr & !(PAGE_SZ)
}
