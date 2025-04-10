/// ASSUMPTIONS:
/// We assume that bbt is static, as such we use static lifetimes.
/// we assume the structure is channels[LUNS[Planes[Blocks]]]
use alloc::vec::Vec;

use crate::{bindings::nvm_mmgr_geometry, core::addresses::PhysicalBlockAddress};

pub struct BadBlockTable {
    pub channel_bbts: Vec<ChannelBadBlockTable>,
}
impl BadBlockTable {
    pub fn new(geometry: &nvm_mmgr_geometry) -> Self {
        BadBlockTable {
            channel_bbts: Vec::with_capacity(geometry.n_of_ch as usize),
        }
    }

    pub fn set_bad_block(&mut self, pba: PhysicalBlockAddress) {
        self.channel_bbts[pba.channel as usize].lun_bbts[pba.lun as usize].plane_bbts
            [pba.plane as usize]
            .bbt[pba.block as usize] = BadBlockStatus::Bad;
    }

    pub fn get_block_status(&mut self, pba: PhysicalBlockAddress) -> BadBlockStatus {
        self.channel_bbts[pba.channel as usize].lun_bbts[pba.lun as usize].plane_bbts
            [pba.plane as usize]
            .bbt[pba.block as usize]
    }
}

pub struct ChannelBadBlockTable {
    pub(self) lun_bbts: Vec<LunBadBlockTable>,
}
impl ChannelBadBlockTable {
    fn new(geometry: &nvm_mmgr_geometry) -> Self {
        Self {
            lun_bbts: Vec::with_capacity(geometry.lun_per_ch as usize),
        }
    }
}

pub struct LunBadBlockTable {
    pub(self) plane_bbts: Vec<PlaneBadBlockTable>,
}
impl LunBadBlockTable {
    fn new(geometry: &nvm_mmgr_geometry) -> Self {
        Self {
            plane_bbts: Vec::with_capacity(geometry.n_of_planes as usize),
        }
    }
}

pub struct PlaneBadBlockTable {
    pub(self) bbt: Vec<BadBlockStatus>,
}
impl PlaneBadBlockTable {
    fn new(geometry: &nvm_mmgr_geometry) -> Self {
        Self {
            bbt: Vec::with_capacity(geometry.blk_per_lun as usize),
        }
    }
}

#[derive(Clone, Copy)]
pub enum BadBlockStatus {
    Bad,
    Good,
    Reserved,
}
