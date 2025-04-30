use core::ptr::null_mut;

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::LRing,
        mem::MemoryRegion,
        safe::ssd_os_get_connection,
    },
    make_connector_static, println,
    requester::requester::RequestWorkloadGenerator,
    shared::core_local_cell::CoreLocalCell,
};

use crate::requester::requester::Request;

make_connector_static!(requester, init, exit, pipe_start, ring);

static lring: LRing<128> = LRing::new();
static ALLOC: SimpleAllocator = SimpleAllocator::new();
pub static WORKLOAD_GENERATOR: CoreLocalCell<RequestWorkloadGenerator<SimpleAllocator>> =
    CoreLocalCell::new();

pub const N_REQUESTS: usize = 1024;

fn init() -> ::core::ffi::c_int {
    #[cfg(feature = "debug")]
    println!("REQUESTER_INIT");

    let mut mem_region = MemoryRegion::new_from_cpu(1);
    let Ok(()) = lring.init(c"REQUESTER_LRING", mem_region.free_start, 0) else {
        panic!("REQUESTER_LRING WAS ALREADY INITIALIZED!");
    };
    mem_region.reserve(lring.get_lring().unwrap().alloc_mem as usize);

    ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());

    #[cfg(feature = "benchmark")]
    {
        WORKLOAD_GENERATOR.set(RequestWorkloadGenerator::new(
            crate::requester::requester::WorkloadType::READ,
            N_REQUESTS,
            &ALLOC,
        ));
        let workload = WORKLOAD_GENERATOR.get_mut();
        workload.init_workload();
    }

    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    let Some(entry) = lring_entry::new(entry) else {
        println!("NULL PTR!");
        return null_mut();
    };

    let workload = WORKLOAD_GENERATOR.get_mut();

    let cur_req: Option<&mut Request> = workload.next_request();

    match cur_req {
        Some(req) => {
            let pipe = ssd_os_get_connection(c"requester", c"requester_l2p");
            req.start_timer();
            entry.set_ctx(req);
            return pipe;
        }
        None => {
            return null_mut();
        }
    }
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    let Some(res) = lring_entry::new(entry) else {
        return 0;
    };
    let Some(req) = res.get_ctx_as_mut::<Request>() else {
        return 0;
    };

    req.end_timer();
    let workload = WORKLOAD_GENERATOR.get_mut();
    workload.request_returned += 1;

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

    if workload.request_returned == workload.get_n_requests() {
           workload.calculate_stats();
       }

    0
}
