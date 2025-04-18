use core::{
    alloc::Allocator,
    cell::{Cell, OnceCell, RefCell},
    mem::MaybeUninit,
};

use alloc::{collections::VecDeque, vec::Vec};

use crate::{bindings::generated::nvm_mmgr_geometry, shared::addresses::PhysicalBlockAddress};

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

    pub fn provision_block(&mut self) -> Result<PhysicalBlockAddress, ProvisionError> {
        // pick channel RR
        self.last_picked_channel = (self.last_picked_channel + 1) % self.channels.len();
        let channel: &mut Channel<A> = &mut self.channels[self.last_picked_channel];
        // pick lun RR

        channel.last_picked_lun = (channel.last_picked_lun + 1) % channel.luns.len();
        let lun: &mut Lun<A> = &mut channel.luns[channel.last_picked_lun];
        // find free block
        // move from free to used
        let block = lun.provision_block()?;

        // return pba
        Ok(PhysicalBlockAddress {
            channel: self.last_picked_channel as u64,
            lun: channel.last_picked_lun as u64,
            plane: block.plane_id as u64,
            block: block.id as u64,
        })
    }
}

impl<A: Allocator + 'static> Lun<A> {
    fn provision_block(&mut self) -> Result<Block, ProvisionError> {
        let Some(block) = self.free.pop_front() else {
            return Err(ProvisionError::NoFreeBlock);
        };

        self.used.push_back(block.clone());

        return Ok(block);
    }
}
