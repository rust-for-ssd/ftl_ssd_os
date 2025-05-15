use core::alloc::Allocator;

/// ASSUMPTIONS:
/// We assume that bbt is static, as such we use static lifetimes.
/// we assume the structure is channels[LUNS[Planes[Blocks]]]
use alloc::vec::Vec;

use crate::media_manager::media_manager::Geometry;
use crate::shared::addresses::PhysicalBlockAddress;

pub struct BadBlockTable<A: Allocator + 'static> {
    pub channels: Vec<Channel<A>, &'static A>,
}
pub struct Channel<A: Allocator + 'static> {
    pub luns: Vec<Lun<A>, &'static A>,
}
pub struct Lun<A: Allocator + 'static> {
    pub planes: Vec<Plane<A>, &'static A>,
}
pub struct Plane<A: Allocator + 'static> {
    pub blocks: Vec<BadBlockStatus, &'static A>,
}

#[derive(Clone, Copy)]
pub enum BadBlockStatus {
    Bad,
    Good,
    Reserved,
}

impl<A: Allocator> BadBlockTable<A> {
    pub fn new(geometry: &Geometry, alloc: &'static A) -> Self {
        let mut channels: Vec<Channel<A>, &A> =
            Vec::with_capacity_in(geometry.n_of_ch as usize, alloc);
        for _ in 0..geometry.n_of_ch {
            let mut luns: Vec<Lun<A>, &A> =
                Vec::with_capacity_in(geometry.lun_per_ch as usize, alloc);
            for _ in 0..geometry.lun_per_ch {
                let mut planes: Vec<Plane<A>, &A> =
                    Vec::with_capacity_in(geometry.n_of_planes as usize, alloc);
                for _ in 0..geometry.n_of_planes {
                    let mut blocks: Vec<BadBlockStatus, &A> =
                        Vec::with_capacity_in(geometry.blk_per_lun as usize, alloc);
                    for _ in 0..geometry.blk_per_lun {
                        blocks.push(BadBlockStatus::Good);
                    }
                    planes.push(Plane { blocks });
                }
                luns.push(Lun { planes });
            }
            channels.push(Channel { luns });
        }
        return Self { channels };
    }

    pub fn set_bad_block(&mut self, pba: &PhysicalBlockAddress) {
        self.channels[pba.channel as usize].luns[pba.lun as usize].planes[pba.plane as usize]
            .blocks[pba.block as usize] = BadBlockStatus::Bad;
    }

    pub fn get_block_status(&self, pba: &PhysicalBlockAddress) -> BadBlockStatus {
        self.channels[pba.channel as usize].luns[pba.lun as usize].planes[pba.plane as usize].blocks
            [pba.block as usize]
    }
}
