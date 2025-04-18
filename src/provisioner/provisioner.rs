use core::{
    alloc::Allocator,
    cell::{Cell, OnceCell, RefCell},
    mem::MaybeUninit,
};

use alloc::{collections::VecDeque, vec::Vec};

use crate::bindings::generated::nvm_mmgr_geometry;

#[derive(Debug)]
pub struct GlobalProvisioner<A: Allocator + 'static> {
    pub channels: Vec<Channel<A>, &'static A>,
    last_picked_channel: usize,
    alloc: &'static A,
}

#[derive(Debug)]
pub struct Channel<A: Allocator + 'static> {
    pub luns: Vec<Lun<A>, &'static A>,
    last_picked_lun: usize,
}

// ASSUMPTION: we assume the free list only contains block which are valid for writing,
//  i.e. they are not reserved or bad
// TODO: what about planes?
#[derive(Debug)]
pub struct Lun<A: Allocator + 'static> {
    pub free: VecDeque<Block, &'static A>,
    pub used: VecDeque<Block, &'static A>,
    pub partially_used: VecDeque<BlockWithPageInfo, &'static A>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Page {
    InUse,
    Free,
}
#[derive(Copy, Clone, Debug)]
pub struct Block {
    pub id: usize,
    pub plane_id: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct BlockWithPageInfo {
    pub id: usize,
    pub plane_id: usize,
    // pub pages: [Page; config::PAGES_PER_BLOCK],
}

#[derive(Debug, PartialEq)]
pub enum ProvisionError {
    AlreadyInit,
    NoFreeBlock,
    NoFreePage,
    // BlockErr(&'s str),
    // FreeList(&'s str),
}

impl<A: Allocator + 'static> GlobalProvisioner<A> {
    pub fn new(geometry: &nvm_mmgr_geometry, alloc: &'static A) -> Self {
        let mut channels: Vec<Channel<A>, &A> =
            Vec::with_capacity_in(geometry.n_of_ch as usize, alloc);

        for _ in 0..geometry.n_of_ch {
            let mut luns: Vec<Lun<A>, &A> =
                Vec::with_capacity_in(geometry.lun_per_ch as usize, alloc);
            for _ in 0..geometry.lun_per_ch {
                let lun = Lun {
                    free: VecDeque::with_capacity_in(geometry.blk_per_lun as usize, alloc),
                    used: VecDeque::with_capacity_in(geometry.blk_per_lun as usize, alloc),
                    partially_used: VecDeque::with_capacity_in(geometry.pg_per_blk as usize, alloc),
                };
                luns.push(lun);
            }
            channels.push(Channel {
                luns,
                last_picked_lun: 0,
            });
        }

        Self {
            channels,
            last_picked_channel: 0,
            alloc,
        }
    }

    // fn get_channel_rr(&self) -> &mut Channel<A> {
    //    self.get_channel_cell()
    //     self.last_picked_channel
    // }

    // pub fn provision_block(&self) -> Result<PhysicalBlockAddress, ProvisionError> {
    //     // pick channel RR
    //     // pick lun RR
    //     // find free block
    //     // move from free to used
    //     // return pba
    //     // let channel =
    // }
}
