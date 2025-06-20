use crate::{
    bindings::generated::nvm_mmgr_geometry,
    l2p::l2p::PhysicalAddr,
    requester::requester::{CommandType, Request},
};
use alloc::collections::BTreeMap;
use core::{alloc::Allocator, ptr::null_mut};

pub type mm_page = [u8; 2];

#[derive(Debug)]
pub struct Geometry {
    pub n_pages: u32,
    pub n_of_ch: u8,
    pub n_of_planes: u8,
    pub lun_per_ch: u8,
    pub blk_per_lun: u16,
    pub pg_per_blk: u16,
}

impl Geometry {
    pub fn map_geometry(nvm_geo: &nvm_mmgr_geometry) -> Geometry {
        Geometry {
            n_pages: nvm_geo.tot_pg as u32,
            n_of_ch: nvm_geo.n_of_ch,
            n_of_planes: nvm_geo.n_of_planes,
            lun_per_ch: nvm_geo.lun_per_ch,
            blk_per_lun: nvm_geo.blk_per_lun,
            pg_per_blk: nvm_geo.pg_per_blk,
        }
    }
}

pub struct MediaManager<A: Allocator + 'static> {
    data_buffer: BTreeMap<PhysicalAddr, mm_page, &'static A>,
}

#[derive(Debug)]
pub enum MM_ERR {
    NoPPAInReq,
    PPANotFound,
    NullDataPtr,
}

static dummy: mm_page = [99, 99];

impl<A: Allocator + 'static> MediaManager<A> {
    pub fn new(alloc: &'static A) -> Self {
        let mm = MediaManager {
            data_buffer: BTreeMap::new_in(alloc),
        };
        mm
    }

    pub fn execute_request(&mut self, request: &Request) -> Result<*mut mm_page, MM_ERR> {
        match request.cmd {
            CommandType::READ => {
                let Some(ppa) = request.physical_addr else {
                    return Err(MM_ERR::NoPPAInReq);
                };

                #[cfg(feature = "benchmark")]
                return Ok(&dummy as *const mm_page as *mut mm_page);

                let Some(res) = self.data_buffer.get(&ppa) else {
                    return Err(MM_ERR::PPANotFound);
                };
                Ok(res.as_ptr().cast_mut().cast())
            }
            CommandType::WRITE => {
                let Some(ppa) = request.physical_addr else {
                    return Err(MM_ERR::NoPPAInReq);
                };

                #[cfg(not(feature = "benchmark"))]
                self.data_buffer
                    .insert(ppa, unsafe { *request.data.clone() });

                Ok(null_mut())
            }
            CommandType::ERASE => {
                let Some(ppa) = request.physical_addr else {
                    return Err(MM_ERR::NoPPAInReq);
                };
                self.data_buffer.remove(&ppa);
                Ok(null_mut())
            }
        }
    }
}
