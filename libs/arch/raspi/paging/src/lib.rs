#![no_std]
#![feature(pointer_is_aligned)]
use core::{mem::size_of, slice::from_raw_parts_mut};

use bitfield::{bitfield, BitRange};

pub trait FrameAlloc {
    fn alloc_frame(&mut self) -> *mut u8;
}

pub struct PageTableRoot<'a> {
    pub lvl0_table: &'a mut [Lvl0TableDescriptor],
    page_size: u64,
}

impl PageTableRoot<'_> {
    pub fn new<T: FrameAlloc>(page_size: u64, alloc: &mut T) -> Self {
        // Allocate a single page for the Level 0 table
        let page = PageTableRoot::new_table_mem(page_size, alloc) as *mut Lvl0TableDescriptor;
        unsafe {
            PageTableRoot {
                lvl0_table: from_raw_parts_mut(page, 512),
                page_size,
            }
        }
    }

    pub fn map_1gib_page<T: FrameAlloc>(&mut self, phys: u64, alloc: &mut T) -> Result<(), ()> {
        assert!((phys as *mut u64).is_aligned_to(0x40000000));

        let virt_addr = VirtualAddr(phys);

        let lvl0_table = &mut self.lvl0_table;
        let mut lvl0_descriptor = lvl0_table[virt_addr.lvl0_idx() as usize];
        if !lvl0_descriptor.valid() {
            // We need to allocate a Lvl1 table to store in this descriptor, then we need
            // to initialize this descriptor
            let table_addr = PageTableRoot::new_table_mem(self.page_size, alloc) as u64;
            lvl0_descriptor.set_valid(true);
            lvl0_descriptor.set_is_table(true);
            lvl0_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));

            // Now store the initialized descriptor back into the table
            lvl0_table[virt_addr.lvl0_idx() as usize] = lvl0_descriptor;
        }

        let lvl1_table_ptr = (lvl0_descriptor.next_table_addr() << 12) as *mut Lvl1BlockDescriptor;
        let lvl1_table = unsafe { from_raw_parts_mut(lvl1_table_ptr, 512) };
        let mut lvl1_block_descriptor = lvl1_table[virt_addr.lvl1_idx() as usize];
        if lvl1_block_descriptor.valid() {
            return Err(());
        } else {
            lvl1_block_descriptor.set_valid(true);
            lvl1_block_descriptor.set_is_table(false);
            lvl1_block_descriptor.set_access_flag(true);
            lvl1_block_descriptor.set_output_addr(phys.bit_range(47, 30));

            // Don't forget to store the page descriptor back into the table
            lvl1_table[virt_addr.lvl1_idx() as usize] = lvl1_block_descriptor;
            Ok(())
        }
    }

    pub fn map_2mib_page<T: FrameAlloc>(
        &mut self,
        phys: u64,
        virt_addr: VirtualAddr,
        alloc: &mut T,
    ) -> Result<(), ()> {
        assert!((virt_addr.0 as *mut u64).is_aligned_to(0x200000));

        let lvl0_table = &mut self.lvl0_table;
        let mut lvl0_descriptor = lvl0_table[virt_addr.lvl0_idx() as usize];
        if !lvl0_descriptor.valid() {
            // We need to allocate a Lvl1 table to store in this descriptor, then we need
            // to initialize this descriptor
            let table_addr = PageTableRoot::new_table_mem(self.page_size, alloc) as u64;
            lvl0_descriptor.set_valid(true);
            lvl0_descriptor.set_is_table(true);
            lvl0_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));

            // Now store the initialized descriptor back into the table
            lvl0_table[virt_addr.lvl0_idx() as usize] = lvl0_descriptor;
        }

        let lvl1_table_ptr = (lvl0_descriptor.next_table_addr() << 12) as *mut Lvl1TableDescriptor;
        let lvl1_table = unsafe { from_raw_parts_mut(lvl1_table_ptr, 512) };
        let mut lvl1_descriptor = lvl1_table[virt_addr.lvl1_idx() as usize];
        if !lvl1_descriptor.valid() {
            // We need to allocate a new Lvl2 table to store in this descriptor
            let table_addr = PageTableRoot::new_table_mem(self.page_size, alloc) as u64;
            lvl1_descriptor.set_valid(true);
            lvl1_descriptor.set_is_table(true);
            lvl1_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));

            // Now store the initialized descriptor back into the table
            lvl1_table[virt_addr.lvl1_idx() as usize] = lvl1_descriptor;
        }

        let lvl2_table_ptr = (lvl1_descriptor.next_table_addr() << 12) as *mut Lvl2BlockDescriptor;
        let lvl2_table = unsafe { from_raw_parts_mut(lvl2_table_ptr, 512) };
        let mut lvl2_block_descriptor = lvl2_table[virt_addr.lvl2_idx() as usize];
        if lvl2_block_descriptor.valid() {
            Err(())
        } else {
            lvl2_block_descriptor.set_valid(true);
            lvl2_block_descriptor.set_is_table(false);
            lvl2_block_descriptor.set_access_flag(true);
            lvl2_block_descriptor.set_output_addr(phys.bit_range(47, 21));

            // Don't forget to store the page descriptor back into the table
            lvl2_table[virt_addr.lvl2_idx() as usize] = lvl2_block_descriptor;
            Ok(())
        }
    }

    // Identity Map a single page for 4kib granularity
    pub fn map_page<T: FrameAlloc>(
        &mut self,
        phys: u64,
        virt_addr: VirtualAddr,
        alloc: &mut T,
    ) -> Result<(), ()> {
        assert!((phys as *mut u64).is_aligned_to(0x1000));

        let lvl0_table = &mut self.lvl0_table;
        let mut lvl0_descriptor = lvl0_table[virt_addr.lvl0_idx() as usize];
        if !lvl0_descriptor.valid() {
            // We need to allocate a Lvl1 table to store in this descriptor, then we need
            // to initialize this descriptor
            let table_addr = PageTableRoot::new_table_mem(self.page_size, alloc) as u64;
            lvl0_descriptor.set_valid(true);
            lvl0_descriptor.set_is_table(true);
            lvl0_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));

            // Now store the initialized descriptor back into the table
            lvl0_table[virt_addr.lvl0_idx() as usize] = lvl0_descriptor;
        }

        let lvl1_table_ptr = (lvl0_descriptor.next_table_addr() << 12) as *mut Lvl1TableDescriptor;
        let lvl1_table = unsafe { from_raw_parts_mut(lvl1_table_ptr, 512) };
        let mut lvl1_descriptor = lvl1_table[virt_addr.lvl1_idx() as usize];
        if !lvl1_descriptor.valid() {
            // We need to allocate a new Lvl2 table to store in this descriptor
            let table_addr = PageTableRoot::new_table_mem(self.page_size, alloc) as u64;
            lvl1_descriptor.set_valid(true);
            lvl1_descriptor.set_is_table(true);
            lvl1_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));

            // Now store the initialized descriptor back into the table
            lvl1_table[virt_addr.lvl1_idx() as usize] = lvl1_descriptor;
        }

        let lvl2_table_ptr = (lvl1_descriptor.next_table_addr() << 12) as *mut Lvl2TableDescriptor;
        let lvl2_table = unsafe { from_raw_parts_mut(lvl2_table_ptr, 512) };
        let mut lvl2_descriptor = lvl2_table[virt_addr.lvl2_idx() as usize];
        if !lvl2_descriptor.valid() {
            // We need to allocate a new Lvl3 table to store in this descriptor
            let table_addr = PageTableRoot::new_table_mem(self.page_size, alloc) as u64;
            lvl2_descriptor.set_valid(true);
            lvl2_descriptor.set_is_table(true);
            lvl2_descriptor.set_next_table_addr(table_addr.bit_range(47, 12));

            // Now store the initialized descriptor back into the table
            lvl2_table[virt_addr.lvl2_idx() as usize] = lvl2_descriptor;
        }

        let page_table_ptr = (lvl2_descriptor.next_table_addr() << 12) as *mut PageDescriptor;
        let page_table = unsafe { from_raw_parts_mut(page_table_ptr, 512) };
        let mut page_descriptor = page_table[virt_addr.lvl3_idx() as usize];

        page_descriptor.set_valid(true);
        page_descriptor.set_is_page(true);
        page_descriptor.set_access_flag(true);
        page_descriptor.set_output_addr(phys.bit_range(47, 12));

        // Don't forget to store the page descriptor back into the table
        page_table[virt_addr.lvl3_idx() as usize] = page_descriptor;

        Ok(())
    }

    fn new_table_mem<T: FrameAlloc>(sz: u64, allocator: &mut T) -> *mut u64 {
        //let addr = FRAME_ALLOCATOR.lock().alloc_page();
        let addr = allocator.alloc_frame();
        unsafe {
            core::ptr::write_bytes(addr, 0, (sz as usize) / size_of::<u64>());
        }
        addr as *mut u64
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
