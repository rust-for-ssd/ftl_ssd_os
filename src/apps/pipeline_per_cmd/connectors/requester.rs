use core::ffi::c_void;
use core::ptr::null_mut;

use crate::bindings::generated::{TICKS_SEC, ssd_os_timer_interrupt_on};
use crate::shared::macros::println;
use crate::{
    allocator::linked_list_alloc::LinkedListAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::LRing,
        mem::MemoryRegion,
        safe::ssd_os_get_connection,
    },
    make_connector_static,
    requester::requester::RequestWorkloadGenerator,
    shared::core_local_cell::CoreLocalCell,
};

use crate::requester::requester::{
    CommandType, Request, get_current_num_submissions, set_timer_interupt,
};

make_connector_static!(requester1, init, exit, pipe_start, ring, 1);

static lring: LRing<128> = LRing::new();
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
pub static WORKLOAD_GENERATOR: CoreLocalCell<RequestWorkloadGenerator<LinkedListAllocator>> =
    CoreLocalCell::new();

pub const N_REQUESTS: usize = 1024;
const RING_CAPACITY: usize = 128;

fn init() -> ::core::ffi::c_int {
    let mut mem_region = MemoryRegion::new_from_cpu(1);
    let Ok(()) = lring.init(c"REQUESTER_LRING", mem_region.free_start, 0) else {
        panic!("REQUESTER_LRING WAS ALREADY INITIALIZED!");
    };
    mem_region.reserve(lring.get_lring().unwrap().alloc_mem as usize);

    ALLOC.set(LinkedListAllocator::new());
    ALLOC
        .get()
        .initialize(mem_region.free_start.cast(), mem_region.end.cast());

    #[cfg(feature = "benchmark")]
    {
        WORKLOAD_GENERATOR.set(RequestWorkloadGenerator::new(
            crate::requester::requester::WorkloadType::MIXED,
            N_REQUESTS,
            ALLOC.get(),
        ));
        let workload = WORKLOAD_GENERATOR.get_mut();
        workload.init_workload();
        set_timer_interupt();
    }

    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    if get_current_num_submissions() < RING_CAPACITY {
        let Some(entry) = lring_entry::new(entry) else {
            return null_mut();
        };

        let Some(req) = WORKLOAD_GENERATOR.get_mut().next_request() else {
            return null_mut();
        };

        entry.set_ctx(req);

        match req.cmd {
            CommandType::READ => return ssd_os_get_connection(c"requester1", c"read"),
            CommandType::WRITE => return ssd_os_get_connection(c"requester1", c"write"),
            _ => return null_mut(),
        }
    } else {
        return null_mut();
    }
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    let Some(res) = lring_entry::new(entry) else {
        return 0;
    };

    let Some(req) = res.get_ctx_as_mut::<Request>() else {
        return 0;
    };
    WORKLOAD_GENERATOR.get_mut().reset_request(req);

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

    0
}
