use core::{
    alloc::Allocator,
    cell::{OnceCell, RefCell},
    mem::MaybeUninit,
};

use alloc::{collections::VecDeque, vec::Vec};

use crate::bindings::generated::nvm_mmgr_geometry;

#[derive(Debug)]
pub struct GlobalProvisioner<A: Allocator + 'static> {
    pub channels: MaybeUninit<RefCell<Vec<Channel<A>, &'static A>>>,
    pub alloc: OnceCell<&'static A>,
}

pub struct Channel<A: Allocator + 'static> {
    pub luns: Vec<Lun<A>, &'static A>,
}

// TODO: what about planes?
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
#[derive(Copy, Clone)]
pub struct Block {
    pub id: usize,
    pub plane_id: usize,
}

#[derive(Copy, Clone)]
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
    pub const fn new() -> Self {
        Self {
            channels: MaybeUninit::zeroed(),
            alloc: OnceCell::new(),
        }
    }

    pub fn init(
        &self,
        geometry: &nvm_mmgr_geometry,
        alloc: &'static A,
    ) -> Result<(), ProvisionError> {
        self.alloc
            .set(&alloc)
            .map_err(|_| ProvisionError::AlreadyInit)?;

        let mut channels: Vec<Channel<A>, &A> =
            Vec::with_capacity_in(geometry.n_of_ch as usize, alloc);

        for _ in 0..geometry.n_of_ch {
            let mut luns: Vec<Lun<A>, &A> =
                Vec::with_capacity_in(geometry.lun_per_ch as usize, alloc);
            for _ in 0..geometry.lun_per_ch {
                // let lun = Lun {
                //     free: VecDeque::with_capacity_in(geometry.blk_per_lun as usize, alloc),
                //     used: VecDeque::with_capacity_in(geometry.blk_per_lun as usize, alloc),
                //     partially_used: VecDeque::with_capacity_in(geometry.pg_per_blk as usize, alloc),
                // };
                // luns.push(lun);
            }
            // channels.push(Channel { luns });
        }

        *self.get_channel_cell().borrow_mut() = channels;

        Ok(())
    }

    pub fn get_channel_cell(&self) -> &RefCell<Vec<Channel<A>, &'static A>> {
        unsafe { self.channels.assume_init_ref() }
    }
}
