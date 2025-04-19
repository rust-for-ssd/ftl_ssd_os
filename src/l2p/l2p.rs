use alloc::vec::Vec;
use core::alloc::Allocator;

// Type aliases for clarity
pub type LogicalAddr = u64;
pub type PhysicalAddr = u64;

#[derive(Debug)]
pub struct L2pMapEntry {
    logical_addr: LogicalAddr,
    physical_addr: PhysicalAddr,
    valid: bool,
}

#[derive(Debug)]
pub struct L2pMapper<A: Allocator + 'static> {
    // Simple vector-based mapping
    entries: Vec<L2pMapEntry, &'static A>,
    alloc: &'static A,
}

impl<A: Allocator + 'static> L2pMapper<A> {
    pub fn new(alloc: &'static A) -> Self {
        L2pMapper {
            entries: Vec::new_in(alloc),
            alloc,
        }
    }

    pub fn with_capacity(capacity: usize, alloc: &'static A) -> Self {
        L2pMapper {
            entries: Vec::with_capacity_in(capacity, alloc),
            alloc,
        }
    }

    // Map a logical address to a physical address
    pub fn map(&mut self, logical_addr: LogicalAddr, physical_addr: PhysicalAddr) {
        // First check if this logical address already exists
        for entry in self.entries.iter_mut() {
            if entry.valid && entry.logical_addr == logical_addr {
                // Update existing entry
                entry.physical_addr = physical_addr;
                return;
            }
        }

        // Check for any invalid entries that can be reused
        for entry in self.entries.iter_mut() {
            if !entry.valid {
                *entry = L2pMapEntry {
                    logical_addr,
                    physical_addr,
                    valid: true,
                };
                return;
            }
        }

        // If no existing entries found, create a new one
        self.entries.push(L2pMapEntry {
            logical_addr,
            physical_addr,
            valid: true,
        });
    }

    pub fn lookup(&self, logical_addr: LogicalAddr) -> Option<PhysicalAddr> {
        for entry in self.entries.iter() {
            if entry.valid && entry.logical_addr == logical_addr {
                return Some(entry.physical_addr);
            }
        }
        None
    }

    pub fn unmap(&mut self, logical_addr: LogicalAddr) -> Option<PhysicalAddr> {
        for entry in self.entries.iter_mut() {
            if entry.valid && entry.logical_addr == logical_addr {
                entry.valid = false;
                return Some(entry.physical_addr);
            }
        }
        None
    }

    pub fn is_mapped(&self, logical_addr: LogicalAddr) -> bool {
        for entry in self.entries.iter() {
            if entry.valid && entry.logical_addr == logical_addr {
                return true;
            }
        }
        false
    }

    pub fn len(&self) -> usize {
        self.entries.iter().filter(|entry| entry.valid).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.valid = false;
        }
    }
}