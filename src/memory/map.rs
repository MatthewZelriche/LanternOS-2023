use core::fmt::Display;

use fdt_rs::{
    base::DevTree,
    error::DevTreeError,
    prelude::{FallibleIterator, PropReader},
};

use super::util::MemSize;

#[derive(Default, Debug)]
pub enum EntryType {
    #[default]
    Free,
    _DtReserved,
    _Firmware,
    _Kernel,
    _Mmio,
}

#[derive(Default)]
pub struct MemoryMapEntry {
    pub base_addr: u64,
    pub size: MemSize,
    pub end_addr: u64,
    pub entry_type: EntryType,
}

impl Display for MemoryMapEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Type: {:?} | {:#018x} - {:#018x} | {}\n",
            self.entry_type, self.base_addr, self.end_addr, self.size
        )?;

        Ok(())
    }
}

pub struct MemoryMap {
    // Before we can create a physical page frame allocator, we need a memory map
    // of our physical address space. But we need to determine our memory map at runtime...
    // For now, since we don't have access to page allocation this early, we assume no more than
    // 32 entries in the memory map.
    entries: [MemoryMapEntry; 32],
    next_idx: usize,
    addr_end: u64,
}

impl Display for MemoryMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for entry in &self.entries {
            if entry.size.to_bytes() != 0 {
                write!(f, "{}", entry)?;
            }
        }
        Ok(())
    }
}

// This OS assumes low-peripheral mode at all times
impl MemoryMap {
    pub fn new(dtb_ptr: *const u8) -> Result<MemoryMap, DevTreeError> {
        let mut map = MemoryMap {
            entries: Default::default(),
            next_idx: 0,
            addr_end: 0,
        };

        let dtb: DevTree;
        unsafe {
            // Sound because this memory region will be protected by the memory map for the entire
            // lifetime of the os
            dtb = DevTree::from_raw_pointer(dtb_ptr).expect("Failed to read dtb! Err");
        }

        // Determine cell sizes
        let mut address_cells = 0;
        let mut size_cells = 0;
        if let Some(root) = dtb.root()? {
            address_cells = root
                .props()
                .find(|x| Ok(x.name()? == "#address-cells"))?
                .ok_or(DevTreeError::ParseError)?
                .u32(0)?;
            size_cells = root
                .props()
                .find(|x| Ok(x.name()? == "#size-cells"))?
                .ok_or(DevTreeError::ParseError)?
                .u32(0)?;
        }

        // First enumerate our free memory blocks
        let mut max_addr: u64 = 0;
        dtb.nodes()
            .filter(|x| Ok(x.name()?.contains("memory@")))
            .for_each(|x| {
                let reg = x
                    .props()
                    .find(|x| Ok(x.name()? == "reg"))?
                    .ok_or(DevTreeError::ParseError)?;

                let base_addr: u64;
                let size_bytes: u64;
                match address_cells {
                    1 => base_addr = reg.u32(0)?.into(),
                    2 => base_addr = reg.u64(0)?,
                    _ => return Err(DevTreeError::ParseError),
                }

                match size_cells {
                    1 => size_bytes = reg.u32(address_cells as usize)?.into(),
                    2 => size_bytes = reg.u64(address_cells as usize)?,
                    _ => return Err(DevTreeError::ParseError),
                }

                if base_addr + size_bytes > max_addr {
                    max_addr = base_addr + size_bytes;
                }

                map.add_entry(MemoryMapEntry {
                    base_addr,
                    size: MemSize { bytes: size_bytes },
                    end_addr: base_addr + size_bytes,
                    entry_type: EntryType::Free,
                })
            })?;

        map.addr_end = max_addr;
        if map.addr_end == 0 {
            Err(DevTreeError::ParseError)
        } else {
            Ok(map)
        }
    }

    pub fn get_free_mem(&self) -> MemSize {
        let mut bytes = 0;
        for entry in &self.entries {
            bytes += entry.size.to_bytes();
        }

        MemSize { bytes }
    }

    pub fn get_total_mem(&self) -> MemSize {
        MemSize {
            bytes: self.addr_end,
        }
    }

    fn add_entry(&mut self, entry: MemoryMapEntry) -> Result<(), DevTreeError> {
        if self.next_idx > 31 {
            Err(DevTreeError::NotEnoughMemory)
        } else {
            self.entries[self.next_idx] = entry;
            self.next_idx += 1;
            Ok(())
        }
    }
}
