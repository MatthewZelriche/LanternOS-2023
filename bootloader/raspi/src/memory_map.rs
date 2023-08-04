use core::fmt::Display;

use arrayvec::ArrayVec;
use fdt_rs::{
    base::DevTree,
    error::DevTreeError,
    prelude::{FallibleIterator, PropReader},
};

extern "C" {
    static __stack_end: u8;
    static __bootloader_start: u8;
    static __bootloader_end: u8;
    static __stack: u8;
}

use crate::{mem_size::MemSize, page_size};
use raspi_peripherals::{mailbox::GetGpuMemory, mmio::Mmio, MAILBOX};

pub fn get_page_addr(addr: u64) -> u64 {
    addr & !(page_size() - 1)
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum EntryType {
    #[default]
    Free,
    DtReserved,
    Firmware,
    Bootloader,
    Stack,
    Kernel,
    Mmio,
}

impl EntryType {
    fn to_string(&self) -> &str {
        match self {
            EntryType::Free => "Free",
            EntryType::DtReserved => "DeviceTree",
            EntryType::Firmware => "Firmware",
            EntryType::Bootloader => "Bootloader",
            EntryType::Mmio => "MMIO",
            EntryType::Stack => "Stack",
            EntryType::Kernel => "Kernel",
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct MemoryMapEntry {
    pub base_addr: u64,
    pub size: MemSize,
    pub end_addr: u64,
    pub entry_type: EntryType,
}

impl MemoryMapEntry {
    fn contains(&self, other: &Self) -> bool {
        let range = self.base_addr..self.end_addr;
        range.contains(&other.base_addr) || range.contains(&other.end_addr)
    }
    fn fully_contains(&self, other: &Self) -> bool {
        let range = self.base_addr..self.end_addr;
        range.contains(&other.base_addr) && range.contains(&other.end_addr)
    }

    fn reduce(&mut self, other: &Self) -> Option<MemoryMapEntry> {
        let mut new_block = None;
        if self.contains(other) {
            if other.base_addr <= self.base_addr {
                self.base_addr = other.end_addr;
                self.size = MemSize {
                    bytes: self.end_addr - self.base_addr,
                };
            } else if other.end_addr >= self.end_addr {
                self.end_addr = other.base_addr;
                self.size = MemSize {
                    bytes: self.end_addr - self.base_addr,
                };
            } else {
                let old_end = self.end_addr;
                // Truncate original
                self.end_addr = other.base_addr;
                self.size = MemSize {
                    bytes: self.end_addr - self.base_addr,
                };

                // Add new free after reserved
                let base = other.end_addr;
                let end = old_end;
                new_block = Some(MemoryMapEntry {
                    base_addr: other.end_addr,
                    size: MemSize { bytes: end - base },
                    end_addr: end,
                    entry_type: EntryType::Free,
                });
            }
        }

        new_block
    }
}

impl Display for MemoryMapEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Type: {:10} | {:#018x} - {:#018x} | {}\n",
            self.entry_type.to_string(),
            self.base_addr,
            self.end_addr,
            self.size
        )?;

        Ok(())
    }
}

pub struct MemoryMap {
    // Before we can create a physical page frame allocator, we need a memory map
    // of our physical address space. But we need to determine our memory map at runtime...
    // For now, since we don't have access to page allocation this early, we assume no more than
    // 32 entries in the memory map.
    entries: ArrayVec<MemoryMapEntry, 32>,
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
            entries: ArrayVec::new(),
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

        // Now we can start assigning reserved blocks...

        // Find and reserve pages for the DTB
        let dtb_page_start = get_page_addr(dtb_ptr as u64);
        let dtb_page_end = (dtb_page_start + dtb.totalsize() as u64).next_multiple_of(page_size());
        let dtb_size_bytes = dtb_page_end - dtb_page_start;
        map.add_entry(MemoryMapEntry {
            base_addr: dtb_page_start,
            size: MemSize {
                bytes: dtb_size_bytes,
            },
            end_addr: dtb_page_end,
            entry_type: EntryType::DtReserved,
        })?;

        // Reserve GPU firmware
        let msg = GetGpuMemory::new();
        let msg = MAILBOX
            .lock()
            .send_message(msg)
            .map_err(|_| DevTreeError::ParseError)?;
        let start: u64 = msg.data.get_base().into();
        let size: u64 = msg.data.get_size().into();
        let end: u64 = start + size;
        map.add_entry(MemoryMapEntry {
            base_addr: start,
            size: MemSize { bytes: size.into() },
            end_addr: end,
            entry_type: EntryType::Firmware,
        })?;

        // Reserve the region for MMIO
        let size = Mmio::PERIPHERALS_PHYS_END - Mmio::PERIPHERALS_PHYS_BASE;
        map.add_entry(MemoryMapEntry {
            base_addr: Mmio::PERIPHERALS_PHYS_BASE,
            size: MemSize { bytes: size },
            end_addr: Mmio::PERIPHERALS_PHYS_END,
            entry_type: EntryType::Mmio,
        })?;

        // Reserve the stack region
        let stack_start;
        let stack_end;
        unsafe {
            stack_start = (&__stack_end as *const u8) as u64;
            stack_end = (&__stack as *const u8) as u64;
        }
        let stack_start_page_addr = get_page_addr(stack_start);
        let stack_end_page_addr = get_page_addr(stack_end); // Stack end is exclusive
        let stack_size = stack_end_page_addr - stack_start_page_addr;
        map.add_entry(MemoryMapEntry {
            base_addr: stack_start_page_addr,
            size: MemSize { bytes: stack_size },
            end_addr: stack_end_page_addr,
            entry_type: EntryType::Stack,
        })?;

        // Reserve the bootloader region
        let bl_start;
        let bl_end;
        unsafe {
            bl_start = (&__bootloader_start as *const u8) as u64;
            bl_end = &__bootloader_end as *const u8 as u64;
        }
        let bl_start_page_addr = get_page_addr(bl_start);
        let bl_end_page_addr = bl_end.next_multiple_of(page_size());
        let bl_size = bl_end_page_addr - bl_start_page_addr;
        map.add_entry(MemoryMapEntry {
            base_addr: bl_start_page_addr,
            size: MemSize { bytes: bl_size },
            end_addr: bl_end_page_addr,
            entry_type: EntryType::Bootloader,
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
            match entry.entry_type {
                EntryType::Free => bytes += entry.size.to_bytes(),
                _ => (),
            }
        }

        MemSize { bytes }
    }

    pub fn get_entries(&self) -> &ArrayVec<MemoryMapEntry, 32> {
        &self.entries
    }

    pub fn get_total_mem(&self) -> MemSize {
        MemSize {
            bytes: self.addr_end,
        }
    }

    pub fn add_entry(&mut self, entry: MemoryMapEntry) -> Result<(), DevTreeError> {
        // Remove free entries if they are completely consumed by a reserved entry
        self.entries.retain(|x| !entry.fully_contains(x));

        // Reduce our free space
        let mut new_entries: ArrayVec<MemoryMapEntry, 4> = ArrayVec::new();
        for existing in self.entries.as_mut_slice() {
            if let Some(additional_entry) = existing.reduce(&entry) {
                new_entries.push(additional_entry);
            }
        }

        // Add any newly created entries to accomodate the reserved entry
        self.entries
            .try_extend_from_slice(&new_entries)
            .map_err(|_| DevTreeError::NotEnoughMemory)?;

        // Add the reserved entry and return
        let res = self
            .entries
            .try_push(entry)
            .map_err(|_| DevTreeError::NotEnoughMemory);

        // Sort the map from 0 to max addr
        self.entries
            .sort_unstable_by(|a, b| a.base_addr.cmp(&b.base_addr));

        res
    }
}
