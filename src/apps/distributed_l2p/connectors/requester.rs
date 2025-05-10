use core::ffi::c_void;
use core::ptr::null_mut;

use crate::bindings::generated::{TICKS_SEC, ssd_os_timer_interrupt_on};
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

use crate::requester::requester::{CommandType, Request, Status, WorkloadType};

make_connector_static!(requester, init, exit, pipe_start, ring, 1);

const RING_CAPACITY: usize = 128;

static LRING: LRing<RING_CAPACITY> = LRing::new();
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
pub static WORKLOAD_GENERATOR: CoreLocalCell<RequestWorkloadGenerator<LinkedListAllocator>> =
    CoreLocalCell::new();

pub const N_REQUESTS: usize = 1024;

pub static mut AMOUNT_IN_LRING: i32 = 0;
pub static mut COUNT: u32 = 0;
pub static mut SUBMITTED: u32 = 0;
pub static mut LAST_COUNT: u32 = 0;

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

    unsafe { ssd_os_timer_interrupt_on(TICKS_SEC as i32, timer_callback as *mut c_void) };
    // #[cfg(feature = "benchmark")]
    // {
    WORKLOAD_GENERATOR.set(RequestWorkloadGenerator::new(
        WorkloadType::WRITE,
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

static mut INIT_ENQUED: usize = 0;

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    match LRING.dequeue_as_mut(entry) {
        Ok(entry) => {
            let Some(req) = entry.get_ctx_as_mut::<Request>() else {
                return null_mut();
            };
            // println!("start: {:?}", req);

            if req.status != Status::MM_DONE {
                unsafe {
                    SUBMITTED += 1;
                    AMOUNT_IN_LRING += 1;
                }
            }

            return ssd_os_get_connection(c"requester", c"requester_l2p");
        }
        Err(_) => {
            // TODO: should check be removed???
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
    }
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    let Some(res) = lring_entry::new(entry) else {
        return 0;
    };
    let Some(req) = res.get_ctx_as_mut::<Request>() else {
        return 0;
    };

    match *req {
        Request {
            status: Status::DONE,
            ..
        } => {
            unsafe {
                COUNT += 1;
                AMOUNT_IN_LRING -= 1
            }
            WORKLOAD_GENERATOR.get_mut().reset_request(req);
            // println!("end: {:?}", req);
            // req.status = Status::PENDING;
            // req.physical_addr = None;
            // match LRING.enqueue(entry) {
            //     Ok(()) => return 0,
            //     Err(LRingErr::Enqueue(i)) => return i,
            //     _ => {
            //         println!("DID NOT MATCH RES FROM ENQUEUE!");
            //         return -1;
            //     }
            // }
            return 0;
        }
        Request {
            status: Status::MM_DONE,
            cmd: CommandType::WRITE,
            ..
        } => match LRING.enqueue(entry) {
            Ok(()) => return 0,
            Err(LRingErr::Enqueue(i)) => return i,
            _ => {
                println!("DID NOT MATCH RES FROM ENQUEUE!");
                return -1;
            }
        },
        Request {
            status: Status::BAD,
            ..
        } => {
            println!("BAD RING: {:?}", req);
            todo!()
        }

        _ => {
            println!("NO MATCH RING: {:?}", req);
            todo!()
        }
    }
}
