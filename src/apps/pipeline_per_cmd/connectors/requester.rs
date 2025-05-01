use core::ptr::null_mut;

use crate::shared::macros::println;
use crate::{
    allocator::linked_list_alloc::LinkedListAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        mem::MemoryRegion,
        safe::ssd_os_get_connection,
    },
    make_connector_static,
    requester::requester::RequestWorkloadGenerator,
    shared::core_local_cell::CoreLocalCell,
};

use crate::requester::requester::{CommandType, Request};

make_connector_static!(requester1, init, exit, pipe_start, ring, 1);

static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
pub static WORKLOAD_GENERATOR: CoreLocalCell<RequestWorkloadGenerator<LinkedListAllocator>> =
    CoreLocalCell::new();

pub const N_REQUESTS: usize = 10000;

static mut READ_PIPE: *mut pipeline = core::ptr::null_mut();
static mut WRITE_PIPE: *mut pipeline = core::ptr::null_mut();

fn init() -> ::core::ffi::c_int {
    let mem_region = MemoryRegion::new_from_cpu(1);
    ALLOC.set(LinkedListAllocator::new());
    ALLOC
        .get()
        .initialize(mem_region.free_start.cast(), mem_region.end.cast());

    unsafe {
        READ_PIPE = ssd_os_get_connection(c"requester1", c"read");
    };
    unsafe {
        WRITE_PIPE = ssd_os_get_connection(c"requester1", c"write");
    };

    #[cfg(feature = "benchmark")]
    {
        WORKLOAD_GENERATOR.set(RequestWorkloadGenerator::new(
            crate::requester::requester::WorkloadType::WRITE,
            N_REQUESTS,
            ALLOC.get(),
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
                    return ssd_os_get_connection(c"requester1", c"read");
                }
                CommandType::WRITE => {
                    entry.set_ctx(req);
                    return ssd_os_get_connection(c"requester1", c"write");
                }
                CommandType::ERASE => {
                    entry.set_ctx(req);
                    return ssd_os_get_connection(c"requester1", c"erase");
                }
            }
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

    // stop timer
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

    #[cfg(feature = "benchmark")]
    if workload.request_returned == workload.get_n_requests() {
        workload.calculate_stats();
    }

    0
}
