use core::alloc::Allocator;
use alloc::vec::Vec;
use crate::println;
use super::super::apps::connector_per_component::connectors::requester::{Request, CommandType};

pub struct MediaManager<A: Allocator + 'static> {
    data_buffer: Vec<u8, &'static A>,
}

impl<A: Allocator + 'static> MediaManager<A> {
    pub fn new(alloc: &'static A) -> Self {
        MediaManager {
            data_buffer: Vec::new_in(alloc),
        }
    }
    
    pub fn execute_request(&mut self, request: Request, data: Option<Vec<u8>>) -> Result<Option<*mut u8>, ()> {
        match request.cmd {
            CommandType::READ => {
                println!("READ DATA SUCESSFULLY");
                Ok(Some(self.data_buffer.as_mut_ptr()))
            }
            CommandType::WRITE => {
                println!("WROTE DATA SUCESSFULLY");
                self.data_buffer.push(99);
                Ok(None)
            }
            CommandType::ERASE => {
                println!("WROTE DATA SUCESSFULLY");
                Ok(None)
            }            
        }
    }
    
}