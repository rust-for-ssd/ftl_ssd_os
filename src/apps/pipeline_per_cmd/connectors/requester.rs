use core::ptr::{null, null_mut};

use alloc::vec::Vec;

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator, bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::{ssd_os_get_connection, ssd_os_sleep},
    }, make_connector_static, media_manager::media_manager::mm_page, println, requester::requester::{RequestWorkloadGenerator, Status, WorkloadType}, shared::core_local_cell::CoreLocalCell
};

use crate::requester::requester::{Request, RequestError, CommandType};


make_connector_static!(requester1, init, exit, pipe_start, ring);

// static lring: LRing<128> = LRing::new();
static ALLOC: SimpleAllocator = SimpleAllocator::new();
pub static WORKLOAD_GENERATOR: CoreLocalCell<RequestWorkloadGenerator<SimpleAllocator>> =
    CoreLocalCell::new();

pub const N_REQUESTS: usize = 1024;

static mut READ_PIPE: *mut pipeline = core::ptr::null_mut();
static mut WRITE_PIPE: *mut pipeline = core::ptr::null_mut();

fn init() -> ::core::ffi::c_int {
    println!("REQUESTER_INIT");
    let mut mem_region = MemoryRegion::new_from_cpu(1);
    // let Ok(()) = lring.init(c"REQUESTER_LRING", mem_region.free_start, 0) else {
    //     panic!("REQUESTER_LRING WAS ALREADY INITIALIZED!");
    // };
    // let ring = lring.get_lring().unwrap();
    // mem_region.reserve(ring.alloc_mem as usize);

    println!("LRING_INIT");
    ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());
    println!("{:?}", mem_region.free_start);
    println!("{:?}", mem_region.end);
    
    
    unsafe { READ_PIPE = ssd_os_get_connection(c"requester1", c"read"); };
    unsafe { WRITE_PIPE = ssd_os_get_connection(c"requester1", c"write"); };

    
    
    // #[cfg(feature = "benchmark")]
    // {
    WORKLOAD_GENERATOR.set(RequestWorkloadGenerator::new(
        WorkloadType::WRITE,
        N_REQUESTS,
        &ALLOC,
    ));
    let workload = WORKLOAD_GENERATOR.get_mut();
    workload.init_workload();  
    // }
    0
}

fn exit() -> ::core::ffi::c_int {
    println!("EXIT!");
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    #[cfg(feature = "debug")]
    ssd_os_sleep(1);

    let Some(entry) = lring_entry::new(entry) else {
        println!("NULL PTR!");
        return null_mut();
    };

    let workload = WORKLOAD_GENERATOR.get_mut();

    let cur_req: Option<&mut Request> = workload.next_request();

    match cur_req {
        Some(req) => {
            req.start_timer();

            match req.cmd {
                CommandType::READ => {
                    entry.set_ctx(req);
                    // println!("HERE1");
                    return ssd_os_get_connection(c"requester1", c"read");
                },
                CommandType::WRITE => {
                    // println!("HERE2");
                    entry.set_ctx(req);
                    return ssd_os_get_connection(c"requester1", c"write")
                },
                CommandType::ERASE => {
                    entry.set_ctx(req);
                    return ssd_os_get_connection(c"requester1", c"erase")
                },
            }
        }
        None => {
            return null_mut();
        }
    }
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    
    // println!("BACK");
    #[cfg(feature = "debug")]
    ssd_os_sleep(1);

    let res = lring_entry::new(entry).unwrap();
    let Some(req) = res.get_ctx_as_mut::<Request>() else {
        return 0;
    };

    // println!("BACK1");

    // stop timer
    req.end_timer();
    
    // println!("BACK2");


    #[cfg(feature = "debug")]
    {
        if !req.data.is_null() {
            unsafe {
                println!("request {} data is: {:?}", req.id, req.data.as_ref());
            }
        }
        println!("REQUEST {} DONE!", req.id);
        println!(
            "Round trip time {} DONE!",
            req.calc_round_trip_time_clock_cycles()
        );
    }

    #[cfg(feature = "benchmark")]
    println!(req.calc_round_trip_time_clock_cycles());

    // println!("BACK3");

    0
}
