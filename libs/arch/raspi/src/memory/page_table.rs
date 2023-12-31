use bitfield::{bitfield, BitRange};
use core::slice::from_raw_parts_mut;
use lock_api::{Mutex, RawMutex};

const GIB: u64 = 0x40000000;
const MIB: u64 = 0x100000;
const KIB: u64 = 0x400;

/// Defines different types of memory for the MMU
///
/// The values of the constants represent the indices into the Aarch64 MAIR register.
#[derive(PartialEq, Clone, Copy)]
pub struct MemoryType(u8);
impl MemoryType {
    /// Device memory represents memory that can produce side effects, such as MMIO registers
    pub const DEVICE: MemoryType = MemoryType(0);
    /// Normal Cachable memory represents all memory that does not produce side effects, most commonly
    /// regular cachable DRAM.
    pub const NORMAL_CACHEABLE: MemoryType = MemoryType(1);
}

pub trait PageAlloc {
    /// Returns a new, zero-initialized frame of memory.
    fn allocate_frame(&mut self) -> Result<*mut u8, ()>;
    /// Frees a frame of memory. Memory must have been allocated with ```allocate_frame```
    fn deallocate_frame(&mut self, frame: *mut u8);
}

/// Represents a single Aarch64 page table, supporting 4KiB granularity.
pub struct PageTable<'a, S: RawMutex, T: PageAlloc> {
    allocator: &'a Mutex<S, T>,
    lvl0_table: &'a mut [Lvl0TableDescriptor],
}

impl<'a, S: RawMutex, T: PageAlloc> PageTable<'a, S, T> {
    /// Constructs a new, empty page table.
    ///
    /// All page tables allocate memory for the lvl0 table, even if they are empty and contain no mappings.
    pub fn new(allocator: &'a Mutex<S, T>) -> Result<Self, ()> {
        // Allocate a single page for the Level 0 table
        let page = allocator.lock().allocate_frame()? as *mut Lvl0TableDescriptor;
        unsafe {
            Ok(PageTable {
                allocator,
                lvl0_table: from_raw_parts_mut(page, 512),
            })
        }
    }

    /// Provides access to the underlying raw pointer, for example to store the pointer in a
    /// register.
    pub fn as_raw_ptr(&self) -> *const Lvl0TableDescriptor {
        self.lvl0_table.as_ptr()
    }

    pub unsafe fn from_raw_ptr(
        ptr: *const Lvl0TableDescriptor,
        allocator: &'a Mutex<S, T>,
    ) -> Self {
        PageTable {
            allocator,
            lvl0_table: from_raw_parts_mut(ptr.cast_mut(), 512),
        }
    }

    /// Performs a page table walk, translating a Virtual address into a Physical address.
    ///
    /// Returns Err if the page table walk fails for any reason, for example if the requested virtual
    /// address is not mapped.
    pub fn virt_to_phys(&self, virt_addr: VirtualAddr) -> Result<u64, ()> {
        let lvl0_table = &self.lvl0_table;
        let lvl0_descriptor = lvl0_table[virt_addr.lvl0_idx() as usize];
        if !lvl0_descriptor.valid() {
            return Err(());
        }

        let lvl1_table_ptr = (lvl0_descriptor.next_table_addr() << 12) as *mut Lvl1TableDescriptor;
        let lvl1_table = unsafe { from_raw_parts_mut(lvl1_table_ptr, 512) };
        let lvl1_descriptor = lvl1_table[virt_addr.lvl1_idx() as usize];
        if !lvl1_descriptor.valid() {
            return Err(());
        } else if !lvl1_descriptor.is_table() {
            let lower: u64 = virt_addr.0.bit_range(11, 0);
            let lvl1_block_descriptor = Lvl1BlockDescriptor(lvl1_descriptor.0);
            return Ok((lvl1_block_descriptor.output_addr() << 12) | lower);
        }

        let lvl2_table_ptr = (lvl1_descriptor.next_table_addr() << 12) as *mut Lvl2TableDescriptor;
        let lvl2_table = unsafe { from_raw_parts_mut(lvl2_table_ptr, 512) };
        let lvl2_descriptor = lvl2_table[virt_addr.lvl2_idx() as usize];
        if !lvl2_descriptor.valid() {
            return Err(());
        } else if !lvl2_descriptor.is_table() {
            let lower: u64 = virt_addr.0.bit_range(11, 0);
            let lvl2_block_descriptor = Lvl2BlockDescriptor(lvl2_descriptor.0);
            return Ok((lvl2_block_descriptor.output_addr() << 12) | lower);
        }

        let page_table_ptr = (lvl2_descriptor.next_table_addr() << 12) as *mut PageDescriptor;
        let page_table = unsafe { from_raw_parts_mut(page_table_ptr, 512) };
        let page_descriptor = page_table[virt_addr.lvl3_idx() as usize];

        let lower: u64 = virt_addr.0.bit_range(11, 0);
        Ok((page_descriptor.output_addr() << 12) | lower)
    }

    /// Unmaps a single 1GiB huge page starting at ```virt_addr```
    ///
    /// Returns true if the page was successfully unmapped, or false otherwise. ```virt_addr``` must be
    /// aligned on a 1GiB boundary.
    pub fn unmap_1gib_page(&mut self, virt_addr: VirtualAddr) -> bool {
        if virt_addr.0 % GIB != 0 {
            return false;
        }

        let lvl0_table = &mut self.lvl0_table;
        let lvl0_descriptor = lvl0_table[virt_addr.lvl0_idx() as usize];
        if !lvl0_descriptor.valid() {
            return false;
        }

        let lvl1_table_ptr = (lvl0_descriptor.next_table_addr() << 12) as *mut Lvl1BlockDescriptor;
        let lvl1_table = unsafe { from_raw_parts_mut(lvl1_table_ptr, 512) };
        let lvl1_descriptor = &mut lvl1_table[virt_addr.lvl1_idx() as usize];
        if !lvl1_descriptor.valid() || lvl1_descriptor.is_table() {
            return false;
        }

        lvl1_descriptor.0 = 0;
        true
    }

    /// Maps a single 1GiB huge page of physical memory starting at ```phys_addr``` to ```virt_addr```.
    ///
    /// When a new table is needed, ```alloc``` will allocate a single frame of memory to store the new
    /// table.
    ///
    /// Returns Err if the page table failed to map the page. ```virt_addr``` and ```phys_addr``` must both
    /// be aligned on a 1GiB boundary.
    pub fn map_1gib_page(
        &mut self,
        phys_addr: u64,
        virt_addr: VirtualAddr,
        memory_type: MemoryType,
    ) -> Result<(), ()> {
        if phys_addr % GIB != 0 || virt_addr.0 % GIB != 0 {
            return Err(());
        }

        let lvl0_table = &mut self.lvl0_table;
        let lvl0_descriptor = &mut lvl0_table[virt_addr.lvl0_idx() as usize];
        if !lvl0_descriptor.valid() {
            // We need to allocate a Lvl1 table to store in this descriptor, then we need
            // to initialize this descriptor
            let table_addr = self.allocator.lock().allocate_frame()? as u64;
            lvl0_descriptor.set_valid(true);
            lvl0_descriptor.set_is_table(true);
            lvl0_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));
        }

        let lvl1_table_ptr = (lvl0_descriptor.next_table_addr() << 12) as *mut Lvl1BlockDescriptor;
        let lvl1_table = unsafe { from_raw_parts_mut(lvl1_table_ptr, 512) };
        let lvl1_block_descriptor = &mut lvl1_table[virt_addr.lvl1_idx() as usize];
        if lvl1_block_descriptor.valid() {
            return Err(());
        } else {
            lvl1_block_descriptor.set_valid(true);
            lvl1_block_descriptor.set_is_table(false);
            lvl1_block_descriptor.set_access_flag(true);
            lvl1_block_descriptor.set_attrib_idx(memory_type.0.into());
            lvl1_block_descriptor.set_output_addr(phys_addr.bit_range(47, 30));
            Ok(())
        }
    }

    /// Maps a single 2MiB huge page of physical memory starting at ```phys_addr``` to ```virt_addr```.
    ///
    /// When a new table is needed, ```alloc``` will allocate a single frame of memory to store the new
    /// table.
    ///
    /// Returns Err if the page table failed to map the page. ```virt_addr``` and ```phys_addr``` must both
    /// be aligned on a 2MiB boundary.
    pub fn map_2mib_page(
        &mut self,
        phys_addr: u64,
        virt_addr: VirtualAddr,
        memory_type: MemoryType,
    ) -> Result<(), ()> {
        if phys_addr % (2 * MIB) != 0 || virt_addr.0 % (2 * MIB) != 0 {
            return Err(());
        }

        let lvl0_table = &mut self.lvl0_table;
        let lvl0_descriptor = &mut lvl0_table[virt_addr.lvl0_idx() as usize];
        if !lvl0_descriptor.valid() {
            // We need to allocate a Lvl1 table to store in this descriptor, then we need
            // to initialize this descriptor
            let table_addr = self.allocator.lock().allocate_frame()? as u64;
            lvl0_descriptor.set_valid(true);
            lvl0_descriptor.set_is_table(true);
            lvl0_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));
        }

        let lvl1_table_ptr = (lvl0_descriptor.next_table_addr() << 12) as *mut Lvl1TableDescriptor;
        let lvl1_table = unsafe { from_raw_parts_mut(lvl1_table_ptr, 512) };
        let lvl1_descriptor = &mut lvl1_table[virt_addr.lvl1_idx() as usize];
        if !lvl1_descriptor.valid() {
            // We need to allocate a new Lvl2 table to store in this descriptor
            let table_addr = self.allocator.lock().allocate_frame()? as u64;
            lvl1_descriptor.set_valid(true);
            lvl1_descriptor.set_is_table(true);
            lvl1_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));
        }

        let lvl2_table_ptr = (lvl1_descriptor.next_table_addr() << 12) as *mut Lvl2BlockDescriptor;
        let lvl2_table = unsafe { from_raw_parts_mut(lvl2_table_ptr, 512) };
        let lvl2_block_descriptor = &mut lvl2_table[virt_addr.lvl2_idx() as usize];
        if lvl2_block_descriptor.valid() {
            Err(())
        } else {
            lvl2_block_descriptor.set_valid(true);
            lvl2_block_descriptor.set_is_table(false);
            lvl2_block_descriptor.set_access_flag(true);
            lvl2_block_descriptor.set_attrib_idx(memory_type.0.into());
            lvl2_block_descriptor.set_output_addr(phys_addr.bit_range(47, 21));
            Ok(())
        }
    }

    /// Maps a single 4KiB page of physical memory starting at ```phys_addr``` to ```virt_addr```.
    ///
    /// When a new table is needed, ```alloc``` will allocate a single frame of memory to store the new
    /// table.
    ///
    /// Returns Err if the page table failed to map the page. ```virt_addr``` and ```phys_addr``` must both
    /// be aligned on a 4KiB boundary.
    pub fn map_page(
        &mut self,
        phys_addr: u64,
        virt_addr: VirtualAddr,
        mem_type: MemoryType,
    ) -> Result<(), ()> {
        if phys_addr % (4 * KIB) != 0 || virt_addr.0 % (4 * KIB) != 0 {
            return Err(());
        }

        let lvl0_table = &mut self.lvl0_table;
        let lvl0_descriptor = &mut lvl0_table[virt_addr.lvl0_idx() as usize];
        if !lvl0_descriptor.valid() {
            // We need to allocate a Lvl1 table to store in this descriptor, then we need
            // to initialize this descriptor
            let table_addr = self.allocator.lock().allocate_frame()? as u64;
            lvl0_descriptor.set_valid(true);
            lvl0_descriptor.set_is_table(true);
            lvl0_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));
        }

        let lvl1_table_ptr = (lvl0_descriptor.next_table_addr() << 12) as *mut Lvl1TableDescriptor;
        let lvl1_table = unsafe { from_raw_parts_mut(lvl1_table_ptr, 512) };
        let lvl1_descriptor = &mut lvl1_table[virt_addr.lvl1_idx() as usize];
        if !lvl1_descriptor.valid() {
            // We need to allocate a new Lvl2 table to store in this descriptor
            let table_addr = self.allocator.lock().allocate_frame()? as u64;
            lvl1_descriptor.set_valid(true);
            lvl1_descriptor.set_is_table(true);
            lvl1_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));
        }

        let lvl2_table_ptr = (lvl1_descriptor.next_table_addr() << 12) as *mut Lvl2TableDescriptor;
        let lvl2_table = unsafe { from_raw_parts_mut(lvl2_table_ptr, 512) };
        let lvl2_descriptor = &mut lvl2_table[virt_addr.lvl2_idx() as usize];
        if !lvl2_descriptor.valid() {
            // We need to allocate a new Lvl3 table to store in this descriptor
            let table_addr = self.allocator.lock().allocate_frame()? as u64;
            lvl2_descriptor.set_valid(true);
            lvl2_descriptor.set_is_table(true);
            lvl2_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));
        }

        let page_table_ptr = (lvl2_descriptor.next_table_addr() << 12) as *mut PageDescriptor;
        let page_table = unsafe { from_raw_parts_mut(page_table_ptr, 512) };
        let page_descriptor = &mut page_table[virt_addr.lvl3_idx() as usize];

        page_descriptor.set_valid(true);
        page_descriptor.set_is_page(true);
        page_descriptor.set_access_flag(true);
        page_descriptor.set_output_addr(phys_addr.bit_range(47, 12));
        page_descriptor.set_attrix_idx(mem_type.0.into());

        Ok(())
    }
}

impl<S: RawMutex, T: PageAlloc> Drop for PageTable<'_, S, T> {
    /// Walks the entire allocated page table, freeing each frame
    fn drop(&mut self) {
        for lvl0_descriptor in &mut *self.lvl0_table {
            if lvl0_descriptor.valid() && lvl0_descriptor.is_table() {
                let lvl1_table_ptr =
                    (lvl0_descriptor.next_table_addr() << 12) as *mut Lvl1TableDescriptor;
                let lvl1_table = unsafe { from_raw_parts_mut(lvl1_table_ptr, 512) };

                for lvl1_descriptor in &mut *lvl1_table {
                    if lvl1_descriptor.valid() && lvl1_descriptor.is_table() {
                        let lvl2_table_ptr =
                            (lvl1_descriptor.next_table_addr() << 12) as *mut Lvl2TableDescriptor;
                        let lvl2_table = unsafe { from_raw_parts_mut(lvl2_table_ptr, 512) };

                        for lvl2_descriptor in &mut *lvl2_table {
                            let page_table_ptr =
                                (lvl2_descriptor.next_table_addr() << 12) as *mut PageDescriptor;

                            self.allocator
                                .lock()
                                .deallocate_frame(page_table_ptr as *mut u8);
                        }
                        self.allocator
                            .lock()
                            .deallocate_frame(lvl2_table_ptr as *mut u8);
                    }
                }
                self.allocator
                    .lock()
                    .deallocate_frame(lvl1_table_ptr as *mut u8);
            }
        }
        self.allocator
            .lock()
            .deallocate_frame(self.lvl0_table.as_ptr() as *mut u8);
    }
}

bitfield! {
    pub struct VirtualAddr(u64);
    impl Debug;
    phys_offset, _: 11, 0;
    lvl3_idx, _: 20, 12;
    lvl2_idx, _: 29, 21;
    lvl1_idx, _: 38, 30;
    lvl0_idx, _: 47, 39;
    reserved, _: 63, 48;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Lvl0TableDescriptor(u64);
    impl Debug;
    valid, set_valid: 0;
    is_table, set_is_table: 1;
    ignored1, set_ignored_1: 11, 2;
    next_table_addr, set_next_table_addr: 47, 12;
    reserved, _: 50, 48;
    ignored2, set_ignored_2: 58, 51;
    pxn_table, set_pxn_table: 59;
    uxn_table, set_uxn_table: 60;
    ap_table, set_ap_table: 62, 61;
    ns_table, set_ns_table: 63;
}

type Lvl1TableDescriptor = Lvl0TableDescriptor;

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Lvl1BlockDescriptor(u64);
    impl Debug;
    valid, set_valid: 0;
    is_table, set_is_table: 1;
    attrib_idx, set_attrib_idx: 4, 2;
    ns, set_ns: 5;
    ap, set_ap: 7, 6;
    sharability, set_sharability: 9, 8;
    access_flag, set_access_flag: 10;
    not_global, set_not_global: 11;
    reserved, _: 15, 12;
    nt, set_nt: 16;
    reserved2, _: 29, 17;
    output_addr, set_output_addr: 47, 30;
    reserved3, _: 49, 48;
    gp, set_gp: 50;
    dbm, set_dbm: 51;
    contiguous, set_contiguous: 52;
    pxn, set_pxn: 53;
    uxn, set_uxn: 54;
    software_use, set_software_use: 58, 55;
    pbha, set_pbha: 62, 59;
    ignored, _: 63;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Lvl2BlockDescriptor(u64);
    impl Debug;
    valid, set_valid: 0;
    is_table, set_is_table: 1;
    attrib_idx, set_attrib_idx: 4, 2;
    ns, set_ns: 5;
    ap, set_ap: 7, 6;
    sharability, set_sharability: 9, 8;
    access_flag, set_access_flag: 10;
    not_global, set_not_global: 11;
    reserved, _: 15, 12;
    nt, set_nt: 16;
    reserved2, _: 20, 17;
    output_addr, set_output_addr: 47, 21;
    reserved3, _: 49, 48;
    gp, set_gp: 50;
    dbm, set_dbm: 51;
    contiguous, set_contiguous: 52;
    pxn, set_pxn: 53;
    uxn, set_uxn: 54;
    software_use, set_software_use: 58, 55;
    pbha, set_pbha: 62, 59;
    ignored, _: 63;
}

type Lvl2TableDescriptor = Lvl0TableDescriptor;

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PageDescriptor(u64);
    impl Debug;
    valid, set_valid: 0;
    is_page, set_is_page: 1;
    attrib_idx, set_attrix_idx: 4, 2;
    ns, set_ns: 5;
    ap, set_ap: 7, 6;
    sharability, set_sharability: 9, 8;
    access_flag, set_access_flag: 10;
    not_global, set_not_global: 11;
    output_addr, set_output_addr: 47, 12;
    reserved, _: 49, 48;
    gp, set_gp: 50;
    dbm, set_dbm: 51;
    contiguous, set_contiguous: 52;
    pxn, set_pxn: 53;
    uxn, set_uxn: 54;
    software_use, set_software_use: 58, 55;
    pbha, set_pbha: 62, 59;
    ignored, _: 63;
}
