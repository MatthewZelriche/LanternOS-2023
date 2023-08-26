use core::fmt::Display;

use arrayvec::ArrayVec;

use crate::mem_size::MemSize;

#[derive(Default, Clone, Copy, PartialEq)]
pub enum EntryType {
    #[default]
    Free,
    Stack,
    DtReserved,
    Firmware,
    Bootloader,
    BLReserved,
    Kernel,
    Mmio,
}

impl EntryType {
    fn to_string(&self) -> &str {
        match self {
            EntryType::Free => "Free",
            EntryType::Stack => "Stack",
            EntryType::DtReserved => "DeviceTree",
            EntryType::Firmware => "Firmware",
            EntryType::Bootloader => "Bootloader",
            EntryType::BLReserved => "BLReserved",
            EntryType::Mmio => "MMIO",
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
        let range = self.base_addr..=self.end_addr;
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

#[derive(Clone)]
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
    pub fn new() -> MemoryMap {
        MemoryMap {
            entries: ArrayVec::new(),
            addr_end: 0,
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

    pub fn set_total_mem(&mut self, total_mem: u64) {
        self.addr_end = total_mem;
    }

    pub fn add_entry(&mut self, mut entry: MemoryMapEntry) -> Result<(), ()> {
        // Merge adjacent entries of same type
        if let Some(old_entry) = self
            .entries
            .iter()
            .find(|x| x.end_addr == entry.base_addr && x.entry_type == entry.entry_type)
        {
            entry.base_addr = old_entry.base_addr;
            entry.size = MemSize {
                bytes: entry.end_addr - entry.base_addr,
            };
        }
        if let Some(old_entry) = self
            .entries
            .iter()
            .find(|x| x.base_addr == entry.end_addr && x.entry_type == entry.entry_type)
        {
            entry.end_addr = old_entry.end_addr;
            entry.size = MemSize {
                bytes: entry.end_addr - entry.base_addr,
            };
        }

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
            .map_err(|_| ())?;

        // Add the reserved entry and return
        let res = self.entries.try_push(entry).map_err(|_| ());

        // Sort the map from 0 to max addr
        self.entries
            .sort_unstable_by(|a, b| a.base_addr.cmp(&b.base_addr));

        res
    }
}
