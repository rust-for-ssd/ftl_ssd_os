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

use crate::requester::requester::{CommandType, Request};

make_connector_static!(requester, init, exit, pipe_start, ring, 1);

static lring: LRing<128> = LRing::new();
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
pub static WORKLOAD_GENERATOR: CoreLocalCell<RequestWorkloadGenerator<LinkedListAllocator>> =
    CoreLocalCell::new();

pub const N_REQUESTS: usize = 1024;

pub static mut AMOUNT_IN_LRING: i32 = 0;
pub static mut COUNT: u32 = 0;
pub static mut SUBMITTED: u32 = 0;
pub static mut LAST_COUNT: u32 = 0;

const RING_CAPACITY: usize = 128;

// ----- SUSTAINED THROUGHPUT EXPERIMENT ---------
fn timer_fn() {
    unsafe {
        let cur = COUNT;
        let diff = cur - LAST_COUNT;
        LAST_COUNT = cur;

        // println!("op/sec       : {:?}", diff);
        // println!("stages/sec   : {:?}", 6*diff); // we have 6 stages
        println!("{:?}", diff); // for benchmark
        // println!("in the rings : {:?}", AMOUNT_IN_LRING);
        // println!("total        : {:?}", COUNT);
        // println!("submitted    : {:?}", SUBMITTED);
    }
}

extern "C" fn timer_callback() {
    timer_fn();
}

// ----- SUSTAINED THROUGHPUT EXPERIMENT ---------
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

    unsafe { ssd_os_timer_interrupt_on(TICKS_SEC as i32, timer_callback as *mut c_void) };
    // #[cfg(feature = "benchmark")]
    // {
    WORKLOAD_GENERATOR.set(RequestWorkloadGenerator::new(
        crate::requester::requester::WorkloadType::MIXED,
        N_REQUESTS,
        ALLOC.get(),
    ));
    let workload = WORKLOAD_GENERATOR.get_mut();
    workload.init_workload();
    // }

    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    if unsafe { AMOUNT_IN_LRING } < RING_CAPACITY as i32 {
        let Some(entry) = lring_entry::new(entry) else {
            return null_mut();
        };

        let Some(req) = WORKLOAD_GENERATOR.get_mut().next_request() else {
            return null_mut();
        };

        entry.set_ctx(req);

        unsafe {
            SUBMITTED += 1;
            AMOUNT_IN_LRING += 1;
        }

        return ssd_os_get_connection(c"requester", c"requester_l2p");
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

    unsafe {
        COUNT += 1;
        AMOUNT_IN_LRING -= 1
    }

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
