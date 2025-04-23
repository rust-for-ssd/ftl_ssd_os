use crate::{
    l2p::l2p::PhysicalAddr,
    println,
    requester::requester::{CommandType, Request},
};
use alloc::{collections::BTreeMap, vec::Vec};
use core::{alloc::Allocator, ptr::null_mut};

pub type mm_page = [u8; 2];

pub struct MediaManager<A: Allocator + 'static> {
    data_buffer: BTreeMap<PhysicalAddr, mm_page, &'static A>,
}

#[derive(Debug)]
pub enum MM_ERR {
    NoPPAInReq,
    PPANotFound,
    NullDataPtr,
}

impl<A: Allocator + 'static> MediaManager<A> {
    pub fn new(alloc: &'static A) -> Self {
        MediaManager {
            data_buffer: BTreeMap::new_in(alloc),
        }
    }

    pub fn execute_request(&mut self, request: &Request) -> Result<*mut mm_page, MM_ERR> {
        match request.cmd {
            CommandType::READ => {
                println!("READ DATA SUCESSFULLY");
                let Some(ppa) = request.physical_addr else {
                    return Err(MM_ERR::NoPPAInReq);
                };
                let Some(res) = self.data_buffer.get(&ppa) else {
                    return Err(MM_ERR::PPANotFound);
                };
                Ok(res.as_ptr().cast_mut().cast())
            }
            CommandType::WRITE => {
                println!("WROTE DATA SUCESSFULLY");

                self.data_buffer
                    .insert(request.physical_addr.unwrap(), unsafe {
                        *request.data.clone()
                    });
                Ok(null_mut())
            }
            CommandType::ERASE => {
                println!("WROTE DATA SUCESSFULLY");
                let Some(ppa) = request.physical_addr else {
                    return Err(MM_ERR::NoPPAInReq);
                };
                self.data_buffer.remove(&ppa);
                Ok(null_mut())
            }
        }
    }
}
