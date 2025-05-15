use core::ptr::null_mut;

use crate::bindings::lring::LRingErr;
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
    CommandType, Request, Status, WorkloadType, get_current_num_submissions, set_timer_interupt,
};

make_connector_static!(requester, init, exit, pipe_start, ring, 1);

const RING_CAPACITY: usize = 128;

static LRING: LRing<RING_CAPACITY> = LRing::new();
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
pub static WORKLOAD_GENERATOR: CoreLocalCell<RequestWorkloadGenerator<LinkedListAllocator>> =
    CoreLocalCell::new();

pub const N_REQUESTS: usize = 1024;

fn init() -> ::core::ffi::c_int {
    let mut mem_region = MemoryRegion::new_from_cpu(1);
    let Ok(()) = LRING.init(c"REQUESTER_LRING", mem_region.free_start, 0) else {
        panic!("REQUESTER_LRING WAS ALREADY INITIALIZED!");
    };
    mem_region.reserve(LRING.get_lring().unwrap().alloc_mem as usize);

    ALLOC.set(LinkedListAllocator::new());
    ALLOC
        .get()
        .initialize(mem_region.free_start.cast(), mem_region.end.cast());

    #[cfg(feature = "benchmark")]
    {
        WORKLOAD_GENERATOR.set(RequestWorkloadGenerator::new(
            WorkloadType::WRITE,
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
    match LRING.dequeue_as_mut(entry) {
        Ok(_entry) => {
            return ssd_os_get_connection(c"requester", c"requester_l2p");
        }
        Err(_) => {
            if get_current_num_submissions() < RING_CAPACITY {
                let Some(entry) = lring_entry::new(entry) else {
                    return null_mut();
                };

                let Some(req) = WORKLOAD_GENERATOR.get_mut().next_request() else {
                    return null_mut();
                };

                entry.set_ctx(req);

                return ssd_os_get_connection(c"requester", c"requester_l2p");
            } else {
                return null_mut();
            }
        }
    }
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    let Some(req) = lring_entry::get_mut_ctx_raw(entry) else {
        return 0;
    };

    match *req {
        Request {
            status: Status::DONE,
            ..
        } => {
            WORKLOAD_GENERATOR.get_mut().reset_request(req);
            return 0;
        }
        Request {
            status: Status::BAD,
            ..
        } => match LRING.enqueue(entry) {
            Ok(()) => 0,
            Err(LRingErr::Enqueue(i)) => i,
            _ => {
                println!("DID NOT MATCH RES FROM ENQUEUE!");
                -1
            }
        },

        _ => {
            println!("NO MATCH RING: {:?}", req);
            todo!()
        }
    }
}
