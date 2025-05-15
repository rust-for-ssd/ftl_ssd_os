use core::alloc::Allocator;

use alloc::{collections::VecDeque, vec::Vec};

use crate::bbt::bbt::{BadBlockStatus, BadBlockTable};
use crate::shared::addresses::{PhysicalBlockAddress, PhysicalPageAddress};

use crate::media_manager::media_manager::Geometry;

#[derive(Debug)]
pub struct Provisioner<A: Allocator + 'static> {
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
#[derive(Debug)]
pub struct Lun<A: Allocator + 'static> {
    pub free: VecDeque<Block, &'static A>,
    pub used: VecDeque<Block, &'static A>,
    pub partially_used: VecDeque<BlockWithPageInfo, &'static A>,
    pages_per_block: u16,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Page {
    pub id: usize,
    pub plane_id: usize,
    pub block_id: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Block {
    pub id: usize,
    pub plane_id: usize,
}

#[derive(Clone, Debug)]
pub struct BlockWithPageInfo {
    pub id: usize,
    pub plane_id: usize,
    pub pages_reserved: u16,
}

#[derive(Debug, PartialEq)]
pub enum ProvisionError {
    AlreadyInit,
    NoFreeBlock,
    NoFreePage,
}

impl<A: Allocator + 'static> Provisioner<A> {
    pub fn new(geometry: &Geometry, alloc: &'static A) -> Self {
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
                    pages_per_block: geometry.pg_per_blk,
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

    pub fn init_free_from_bbt<BBT_A: Allocator + 'static>(
        &mut self,
        geo: &Geometry,
        bbt: &BadBlockTable<BBT_A>,
    ) {
        for channel in 0..geo.n_of_ch {
            for lun in 0..geo.lun_per_ch {
                for plane in 0..geo.n_of_planes {
                    for block in 0..geo.blk_per_lun as u16 / geo.n_of_planes as u16 {
                        let pba = PhysicalBlockAddress {
                            channel: channel.into(),
                            lun: lun.into(),
                            plane: plane.into(),
                            block: block.into(),
                        };
                        if let BadBlockStatus::Good = bbt.get_block_status(&pba) {
                            self.push_free_block(&pba);
                        }
                    }
                }
            }
        }
    }

    pub fn init_all_free(&mut self) {
        for ch in self.channels.iter_mut() {
            for lun in ch.luns.iter_mut() {
                let cap = lun.free.capacity();
                for block_idx in 0..cap {
                    lun.free.push_back(Block {
                        id: block_idx,
                        plane_id: 0,
                    });
                }
            }
        }
    }

    pub fn provision_block(&mut self) -> Result<PhysicalBlockAddress, ProvisionError> {
        // pick channel RR
        for ch_i in 0..self.channels.len() {
            let ch_idx = (self.last_picked_channel + ch_i) % self.channels.len();
            let channel = &mut self.channels[ch_idx];

            // pick lun RR
            for lun_i in 0..channel.luns.len() {
                let lun_idx = (channel.last_picked_lun + lun_i) % channel.luns.len();

                // find free block
                // move from free to used
                if let Ok(block) = channel.luns[lun_idx].provision_block() {
                    self.last_picked_channel = ch_idx;
                    channel.last_picked_lun = lun_idx;
                    return Ok(PhysicalBlockAddress {
                        channel: ch_idx as u64,
                        lun: lun_idx as u64,
                        plane: block.plane_id as u64,
                        block: block.id as u64,
                    });
                };
            }
        }
        Err(ProvisionError::NoFreeBlock)
    }
    pub fn provision_page(&mut self) -> Result<PhysicalPageAddress, ProvisionError> {
        // pick channel RR
        for ch_i in 0..self.channels.len() {
            let ch_idx = (self.last_picked_channel + ch_i) % self.channels.len();
            let channel = &mut self.channels[ch_idx];

            // pick lun RR
            for lun_i in 0..channel.luns.len() {
                let lun_idx = (channel.last_picked_lun + lun_i) % channel.luns.len();

                // find free block
                // move from free to used
                if let Ok(page) = channel.luns[lun_idx].provision_page() {
                    // self.last_picked_channel = ch_idx;
                    // channel.last_picked_lun = lun_idx;
                    return Ok(PhysicalPageAddress {
                        channel: ch_idx as u64,
                        lun: lun_idx as u64,
                        plane: page.plane_id as u64,
                        block: page.block_id as u64,
                        page: page.id as u64,
                    });
                };
            }
        }

        Err(ProvisionError::NoFreePage)
    }
    pub fn push_free_block(&mut self, pba: &PhysicalBlockAddress) {
        self.channels[pba.channel as usize].luns[pba.lun as usize]
            .free
            .push_back(Block {
                id: pba.block as usize,
                plane_id: pba.plane as usize,
            });
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

    fn provision_page(&mut self) -> Result<Page, ProvisionError> {
        if self.partially_used.is_empty() {
            let Some(block) = self.free.pop_front() else {
                #[cfg(feature = "benchmark")]
                return Ok(Page {
                    id: 0,
                    plane_id: 0,
                    block_id: 0,
                });
                return Err(ProvisionError::NoFreePage);
            };
            self.partially_used.push_back(BlockWithPageInfo {
                id: block.id,
                plane_id: block.plane_id,
                pages_reserved: 0,
            });
        }

        let page = {
            let block = self.partially_used.front_mut().unwrap();
            let page_id = block.pages_reserved;
            block.pages_reserved += 1;
            Page {
                id: page_id.into(),
                plane_id: block.plane_id,
                block_id: block.id,
            }
        };

        if page.id == (self.pages_per_block - 1).into() {
            self.partially_used.pop_front();
            self.used.push_back(Block {
                id: page.block_id,
                plane_id: page.plane_id,
            });
        }

        return Ok(page);
    }
}
