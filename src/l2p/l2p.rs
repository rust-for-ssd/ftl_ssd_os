use alloc::{collections::BTreeMap, vec::Vec};
use core::alloc::Allocator;

use crate::shared::macros::println;

pub type LogicalAddr = u32;
pub type PhysicalAddr = u32;

#[derive(Debug)]
pub struct L2pMapper<const CAPACITY: usize, A: Allocator + 'static> {
    // entries: BTreeMap<LogicalAddr, PhysicalAddr, &'static A>,
    entries: Vec<Option<PhysicalAddr>, &'static A>,
    alloc: &'static A,
}

impl<const CAPACITY: usize, A: Allocator + 'static> L2pMapper<CAPACITY, A> {
    pub fn new(alloc: &'static A) -> Self {
        let mut entries = Vec::with_capacity_in(CAPACITY, alloc);
        for _ in 0..CAPACITY {
            entries.push(None);
        }
        L2pMapper { entries, alloc }
    }

    pub fn map(&mut self, logical_addr: LogicalAddr, physical_addr: PhysicalAddr) {
        self.entries[logical_addr as usize] = Some(physical_addr);
    }

    pub fn lookup(&self, logical_addr: LogicalAddr) -> Option<PhysicalAddr> {
        self.entries[logical_addr as usize].clone()
    }

    pub fn unmap(&mut self, logical_addr: LogicalAddr) -> Option<PhysicalAddr> {
        let res = self.lookup(logical_addr);
        self.entries[logical_addr as usize] = None;
        return res;
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

    pub fn prepare_for_benchmark(&mut self, n_requests: usize) {
        for i in 0..n_requests {
            self.map(i as u32, i as u32);
        }
    }
}
