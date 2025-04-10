use core::cell::RefCell;

/// ASSUMPTIONS:
/// We assume that bbt is static, as such we use static lifetimes.
/// we assume the structure is channels[LUNS[Planes[Blocks]]]
use alloc::vec::{self, Vec};

use crate::{bindings::nvm_mmgr_geometry, shared::addresses::PhysicalBlockAddress};


pub struct BadBlockTable {
    pub channels: RefCell<Vec<Channel>>,
}
pub struct Channel {
    pub luns: Vec<Lun>,
}
pub struct Lun {
    pub planes: Vec<Plane>,
}
pub struct Plane {
    pub blocks: Vec<BadBlockStatus>,
}

#[derive(Clone, Copy)]
pub enum BadBlockStatus {
    Bad,
    Good,
    Reserved,
}


impl BadBlockTable {
    pub const fn new() -> Self {
        BadBlockTable {
            channels: RefCell::new(Vec::new()),
        }
    }
    pub fn init(&self, geometry: &nvm_mmgr_geometry) -> () {
        
        let mut channels = Vec::with_capacity(geometry.n_of_ch as usize);
        for _ in 0..geometry.n_of_ch {
            let mut luns = Vec::with_capacity(geometry.lun_per_ch as usize);
            for _ in 0..geometry.lun_per_ch {
                let mut planes = Vec::with_capacity(geometry.n_of_planes as usize);
                for _ in 0..geometry.n_of_planes {
                    let mut blocks = Vec::with_capacity(geometry.blk_per_lun as usize);
                    for _ in 0..geometry.blk_per_lun {
                        blocks.push(BadBlockStatus::Good);
                    }
                    planes.push(Plane { blocks });
                }
                luns.push(Lun { planes });
            }
            channels.push(Channel { luns });
        }
        self.channels.replace(channels);
        return; 
    }

    pub fn set_bad_block(&self, pba: &PhysicalBlockAddress) {
        self.channels.borrow_mut()[pba.channel as usize]
            .luns[pba.lun as usize]
            .planes[pba.plane as usize]
            .blocks[pba.block as usize] = BadBlockStatus::Bad;
    }

    pub fn get_block_status(&self, pba: &PhysicalBlockAddress) -> BadBlockStatus {
        self.channels.borrow()[pba.channel as usize]
            .luns[pba.lun as usize]
            .planes[pba.plane as usize]
            .blocks[pba.block as usize]
    }
}

unsafe impl Sync for BadBlockTable {}


// pub struct BadBlockTable {
//     pub channel_bbts: Vec<ChannelBadBlockTable>,
// }
// impl BadBlockTable {
//     pub fn new(geometry: &nvm_mmgr_geometry) -> Self {
//         BadBlockTable {
//             channel_bbts: Vec::with_capacity(geometry.n_of_ch as usize),
//         }
//     }

//     pub fn set_bad_block(&mut self, pba: PhysicalBlockAddress) {
//         self.channel_bbts[pba.channel as usize].lun_bbts[pba.lun as usize].plane_bbts
//             [pba.plane as usize]
//             .bbt[pba.block as usize] = BadBlockStatus::Bad;
//     }

//     pub fn get_block_status(&mut self, pba: PhysicalBlockAddress) -> BadBlockStatus {
//         self.channel_bbts[pba.channel as usize].lun_bbts[pba.lun as usize].plane_bbts
//             [pba.plane as usize]
//             .bbt[pba.block as usize]
//     }
// }

// pub struct ChannelBadBlockTable {
//     pub(self) lun_bbts: Vec<LunBadBlockTable>,
// }
// impl ChannelBadBlockTable {
//     fn new(geometry: &nvm_mmgr_geometry) -> Self {
//         Self {
//             lun_bbts: Vec::with_capacity(geometry.lun_per_ch as usize),
//         }
//     }
// }

// pub struct LunBadBlockTable {
//     pub(self) plane_bbts: Vec<PlaneBadBlockTable>,
// }
// impl LunBadBlockTable {
//     fn new(geometry: &nvm_mmgr_geometry) -> Self {
//         Self {
//             plane_bbts: Vec::with_capacity(geometry.n_of_planes as usize),
//         }
//     }
// }

// pub struct PlaneBadBlockTable {
//     pub(self) bbt: Vec<BadBlockStatus>,
// }
// impl PlaneBadBlockTable {
//     fn new(geometry: &nvm_mmgr_geometry) -> Self {
//         Self {
//             bbt: Vec::with_capacity(geometry.blk_per_lun as usize),
//         }
//     }
// }

// #[derive(Clone, Copy)]
// pub enum BadBlockStatus {
//     Bad,
//     Good,
//     Reserved,
// }
