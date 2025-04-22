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
    
    pub fn execute_request(&mut self, request: Request, data: Option<Vec<u8>>) -> () {
        match request.cmd {
            CommandType::READ => {
                self.read(request);
            }
            CommandType::WRITE => {
                self.write(request, data);
            }
            CommandType::ERASE => {
                self.erase(request);
            }            
        }
    }
    
    fn read(&self, request: Request) {
        println!("READ SUCCESSFUL")
    }
    
    fn write(&mut self, request: Request, data: Option<Vec<u8>>) {
        println!("WRITE SUCCESSFUL")
    }
    
    fn erase(&self, request: Request) {
        println!("ERASE SUCCESSFUL")
    }
}