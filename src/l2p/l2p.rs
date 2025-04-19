use alloc::collections::BTreeMap;
use core::alloc::Allocator;

pub type LogicalAddr = u64;
pub type PhysicalAddr = u64;

#[derive(Debug)]
pub struct L2pMapper<A: Allocator + 'static> {
    entries: BTreeMap<LogicalAddr, PhysicalAddr, &'static A>,
    alloc: &'static A,
}

impl<A: Allocator + 'static> L2pMapper<A> {
    pub fn new(alloc: &'static A) -> Self {
        let entries = BTreeMap::new_in(alloc);
        L2pMapper {
            entries,
            alloc,
        }
    }

    // Map a logical address to a physical address
    pub fn map(&mut self, logical_addr: LogicalAddr, physical_addr: PhysicalAddr) {
        self.entries.insert(logical_addr, physical_addr);
    }

    pub fn lookup(&self, logical_addr: LogicalAddr) -> Option<PhysicalAddr> {
        self.entries.get(&logical_addr).copied()
    }

    pub fn unmap(&mut self, logical_addr: LogicalAddr) -> Option<PhysicalAddr> {
        self.entries.remove(&logical_addr)
    }

    pub fn is_mapped(&self, logical_addr: LogicalAddr) -> bool {
        self.entries.contains_key(&logical_addr)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}