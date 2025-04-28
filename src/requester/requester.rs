use crate::{bindings::generated::ssd_os_sleep, media_manager::media_manager::mm_page, println};

#[derive(Debug, Clone, Copy)]
pub enum CommandType {
    READ,
    WRITE,
    ERASE
}

#[derive(Debug, Clone, Copy)]
pub struct Request {
    pub id: u32, 
    pub cmd: CommandType, 
    pub logical_addr: u32,
    pub physical_addr: Option<u32>,
    pub data: *mut mm_page,
    
    // Timing metadata
    pub start_time: u32,
    pub end_time: u32 
}

#[derive(Debug, Clone, Copy)]
pub enum RequestError {
    ConnectorError, 
    StageError,
} 

impl Default for Request {
    fn default() -> Self {
        Self {
            id: 0,
            cmd: CommandType::READ,
            logical_addr: 0,
            physical_addr: None,
            data: core::ptr::null_mut(),
            start_time: 0,
            end_time: 0,
        }
    }
}

impl Request {
    pub fn calc_round_trip_time_clock_cycles(&self) -> u32 {
        // println!("Start time {:?}", self.start_time);
        // println!("End time {:?}", self.end_time);
        // println!(self.end_time)
        self.end_time - self.start_time
    }
    pub fn start_timer(&mut self) -> () {
        self.start_time = read_mtime();
    }
    
    pub fn end_timer(&mut self) -> () {
        self.end_time = read_mtime();
    }
}

// Timer 
const MTIME_REG: usize = 0x200BFF8;
const VIRT_FREQ: u32 = 10000000;


#[inline]
pub fn read_mtime() -> u32 { // it's in ms 
    unsafe {
        // Access the memory-mapped MTIME register
        let mtime_addr = MTIME_REG as *const u32;
        core::ptr::read_volatile(mtime_addr)
    }
}