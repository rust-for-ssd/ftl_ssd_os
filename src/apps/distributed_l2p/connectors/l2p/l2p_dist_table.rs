use crate::l2p::l2p::LogicalAddr;
use alloc::collections::BTreeMap;
use core::{alloc::Allocator, ffi::CStr};

// Pick l2p in rr fashion
pub struct L2PDistributionTable<A: Allocator + 'static, const CAPACITY: usize> {
    last_picked: u8,
    table: BTreeMap<LogicalAddr, u8, &'static A>,
    pipe_names: [&'static CStr; CAPACITY],
}

impl<A: Allocator + 'static, const CAPACITY: usize> L2PDistributionTable<A, CAPACITY> {
    pub fn new(alloc: &'static A, pipe_names: [&'static CStr; CAPACITY]) -> Self {
        Self {
            last_picked: 0,
            table: BTreeMap::new_in(&alloc),
            pipe_names,
        }
    }

    pub fn get_table_idx(&self, logical_addr: &LogicalAddr) -> Option<u8> {
        self.table.get(logical_addr).copied()
    }

    pub fn set_table_idx(&mut self, logical_addr: LogicalAddr, table_id: u8) -> Option<u8> {
        self.table.insert(logical_addr, table_id)
    }

    pub fn pick_table_idx(&mut self) -> u8 {
        let res = self.last_picked;
        self.last_picked = (self.last_picked + 1) % CAPACITY as u8;
        res
    }

    pub fn get_table_pipe_name(&self, table_idx: u8) -> &CStr {
        self.pipe_names[table_idx as usize]
    }

    pub fn prepare_for_benchmark(&mut self, n_reqs: usize) {
        for i in 0..n_reqs {
            self.table.insert(i as u32, (i % CAPACITY) as u8);
        }
    }
}
