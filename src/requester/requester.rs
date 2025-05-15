use core::alloc::Allocator;
use core::ffi::c_void;
use core::u8;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use crate::bindings::generated::{TICKS_SEC, ssd_os_timer_interrupt_on};
use crate::shared::macros::println;
use crate::{
    bindings::generated::ssd_os_sleep,
    l2p::l2p::LogicalAddr,
    media_manager::media_manager::{Geometry, mm_page},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandType {
    READ,
    WRITE,
    ERASE,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    BAD,
    DONE,
    IN_PROCESS,
    PENDING,
}

#[derive(Debug, Clone, Copy)]
pub enum RequestError {
    ConnectorError,
    StageError,
}

#[derive(Debug, Clone, Copy)]
pub enum META_DATA {
    NONE,
    OLD_PPA(u32),
}

#[derive(Debug, Clone, Copy)]
pub struct Request {
    pub id: u32,
    pub status: Status,
    pub cmd: CommandType,
    pub logical_addr: u32,
    pub physical_addr: Option<u32>,
    pub data: *mut mm_page,
    pub md: META_DATA,

    // Timing metadata
    pub start_time: u32,
    pub end_time: u32,
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
            status: Status::IN_PROCESS,
            md: META_DATA::NONE,
        }
    }
}

impl Request {
    pub fn new(id: u32, cmd: CommandType, logical_addr: LogicalAddr, data: *mut mm_page) -> Self {
        Request {
            id: id,
            status: Status::PENDING,
            cmd: cmd,
            logical_addr: logical_addr,
            physical_addr: None,
            data: data,
            start_time: 0,
            end_time: 0,
            md: META_DATA::NONE,
        }
    }

    pub const fn empty() -> Self {
        Self {
            id: 0,
            cmd: CommandType::READ,
            logical_addr: 0,
            physical_addr: None,
            data: core::ptr::null_mut(),
            start_time: 0,
            end_time: 0,
            status: Status::IN_PROCESS,
            md: META_DATA::NONE,
        }
    }

    pub fn calc_round_trip_time_clock_cycles(&self) -> u32 {
        self.end_time - self.start_time
    }
    pub fn start_timer(&mut self) -> () {
        self.start_time = read_mtime();
    }

    pub fn end_timer(&mut self) -> () {
        self.end_time = read_mtime();
    }

    pub fn from_ctx_ptr<'c>(ctx: *mut c_void) -> &'c mut Self {
        unsafe { ctx.cast::<Request>().as_mut().unwrap() }
    }
}

// Timer
const MTIME_REG: usize = 0x200BFF8;
const VIRT_FREQ: u32 = 10000000;

#[inline]
pub fn read_mtime() -> u32 {
    // it's in ms
    unsafe {
        // Access the memory-mapped MTIME register
        let mtime_addr = MTIME_REG as *const u32;
        core::ptr::read_volatile(mtime_addr)
    }
}

pub enum WorkloadType {
    READ,
    WRITE,
    MIXED,
}

pub struct RequestWorkloadGenerator<A: Allocator + 'static> {
    requests: Vec<Request, &'static A>,
    pending: VecDeque<usize, &'static A>,
    cur_request_idx: usize,
    pub request_returned: usize,
    workload_type: WorkloadType,
    start_time: u32,
    end_time: u32,
    write_data: mm_page,
}

impl<A: Allocator + 'static> RequestWorkloadGenerator<A> {
    pub fn new(workload_type: WorkloadType, size: usize, alloc: &'static A) -> Self {
        RequestWorkloadGenerator {
            requests: Vec::with_capacity_in(size, alloc),
            pending: VecDeque::with_capacity_in(size, alloc),
            cur_request_idx: 0,
            request_returned: 0,
            workload_type,
            start_time: 0,
            end_time: 0,
            write_data: [42, 42],
        }
    }

    pub fn init_workload(&mut self) {
        for i in 0..self.requests.capacity() {
            match self.workload_type {
                WorkloadType::READ => self.requests.push(Request::new(
                    i as u32,
                    CommandType::READ,
                    i as LogicalAddr,
                    core::ptr::null_mut(),
                )),
                WorkloadType::WRITE => self.requests.push(Request::new(
                    i as u32,
                    CommandType::WRITE,
                    i as LogicalAddr,
                    &self.write_data as *const mm_page as *mut mm_page,
                )),
                WorkloadType::MIXED => {
                    if i % 2 == 0 {
                        self.requests.push(Request::new(
                            i as u32,
                            CommandType::READ,
                            i as LogicalAddr,
                            core::ptr::null_mut(),
                        ))
                    } else {
                        self.requests.push(Request::new(
                            i as u32,
                            CommandType::WRITE,
                            i as LogicalAddr,
                            &self.write_data as *const mm_page as *mut mm_page,
                        ))
                    }
                }
            }
        }
        for i in 0..self.requests.len() {
            self.pending.push_back(i);
        }
    }

    pub fn next_request(&mut self) -> Option<&Request> {
        let id = self.pending.pop_front()?;
        let req = self.requests.get(id)?;
        unsafe {
            SUBMITTED += 1;
            AMOUNT_IN_LRING += 1;
        }
        return Some(req);
    }

    pub fn reset_request(&mut self, req: &mut Request) {
        req.status = Status::PENDING;
        req.physical_addr = None;
        req.md = META_DATA::NONE;
        req.data = &self.write_data as *const mm_page as *mut mm_page;
        unsafe {
            COUNT += 1;
            AMOUNT_IN_LRING -= 1
        }
        self.pending.push_back(req.id as usize)
    }

    pub fn calculate_stats(&mut self) {
        unsafe { ssd_os_sleep(1) };
        for i in 0..self.requests.capacity() {
            let Some(res) = self.requests.get_mut(i) else {
                return;
            };
            println!(res.calc_round_trip_time_clock_cycles())
        }
    }

    pub fn get_geo(&self) -> Geometry {
        let n_of_ch = 8;
        let n_of_planes = 2;
        let lun_per_ch = 4;
        let blk_per_lun = 64;
        let pg_per_blk = 64;
        let n_pages = n_of_ch * lun_per_ch * blk_per_lun * pg_per_blk;
        let total = self.requests.len();
        assert!(total <= n_pages);
        Geometry {
            n_of_ch: n_of_ch as u8,
            n_of_planes,
            lun_per_ch: lun_per_ch as u8,
            blk_per_lun: blk_per_lun as u16,
            pg_per_blk: pg_per_blk as u16,
            n_pages: n_pages as u32,
        }
    }

    pub fn get_n_requests(&self) -> usize {
        self.requests.capacity()
    }
}

static mut AMOUNT_IN_LRING: usize = 0;
static mut COUNT: u32 = 0;
static mut SUBMITTED: u32 = 0;
static mut LAST_COUNT: u32 = 0;

pub fn get_current_num_submissions() -> usize {
    return unsafe { AMOUNT_IN_LRING };
}

pub extern "C" fn timer_fn() {
    unsafe {
        let cur = COUNT;
        let diff = cur - LAST_COUNT;
        LAST_COUNT = cur;
        println!("{:?}", diff);
    }
}

pub fn set_timer_interupt() {
    unsafe { ssd_os_timer_interrupt_on(TICKS_SEC as i32, timer_fn as *mut c_void) };
}
