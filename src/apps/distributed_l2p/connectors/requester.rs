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

use crate::requester::requester::{CommandType, Request, Status};

make_connector_static!(requester, init, exit, pipe_start, ring, 1);

static LRING: LRing<{ 128 * 2 }> = LRing::new();
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
pub static WORKLOAD_GENERATOR: CoreLocalCell<RequestWorkloadGenerator<LinkedListAllocator>> =
    CoreLocalCell::new();

pub const N_REQUESTS: usize = 1024;

pub static mut AMOUNT_IN_LRING: i32 = 0;
pub static mut COUNT: u32 = 0;
pub static mut SUBMITTED: u32 = 0;
pub static mut LAST_COUNT: u32 = 0;

pub static DATA_TO_WRITE: [u8; 2] = [99, 99];

static POOL_SIZE: usize = 4000;
static RING_SIZE: usize = 128;

// ----- SUSTAINED THROUGHPUT EXPERIMENT ---------
static mut MESSAGE_POOL: [Request; POOL_SIZE] = [Request::empty(); POOL_SIZE];
static mut MSG_USAGE_BITMAP: [bool; POOL_SIZE] = [false; POOL_SIZE];

fn get_free_message_index() -> Option<usize> {
    unsafe {
        for i in 0..POOL_SIZE {
            if !MSG_USAGE_BITMAP[i] {
                MSG_USAGE_BITMAP[i] = true;
                return Some(i);
            }
        }
    }
    println!("No Messages left");
    None
}

// Release a message back to the pool
fn release_message(index: usize) {
    if index < POOL_SIZE {
        unsafe {
            MSG_USAGE_BITMAP[index] = false;
            MESSAGE_POOL[index] = Request::empty();
        }
    } else {
        panic!("something is wrong")
    }
}

// Helper to get a pointer to a message from the pool
fn get_message_ptr(index: usize) -> *mut Request {
    if index < POOL_SIZE {
        unsafe { &mut MESSAGE_POOL[index] as *mut Request }
    } else {
        null_mut()
    }
}

// Helper to get index from a pointer
fn get_index_from_ptr(ptr: *const Request) -> Option<usize> {
    if ptr.is_null() {
        return None;
    }

    unsafe {
        let base_addr = &MESSAGE_POOL[0] as *const Request;
        let offset = (ptr as usize - base_addr as usize) / core::mem::size_of::<Request>();

        if offset < POOL_SIZE {
            Some(offset)
        } else {
            None
        }
    }
}

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
    unsafe {
        AMOUNT_IN_LRING = 0;
    }

    // println!("AMOUNT: {}", unsafe {AMOUNT});
    crate::shared::macros::dbg_println!("REQUESTER_INIT");

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
        crate::requester::requester::WorkloadType::READ,
        N_REQUESTS,
        ALLOC.get(),
    ));
    //     let workload = WORKLOAD_GENERATOR.get_mut();
    //     workload.init_workload();
    // }

    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

static mut INIT_ENQUED: usize = 0;

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    if unsafe { INIT_ENQUED } < 64 {
        unsafe { INIT_ENQUED += 1 };
        unsafe {
            if let Some(idx) = get_free_message_index() {
                // println!(idx as u32 % N_REQUESTS as u32);
                // idx as u32 % N_REQUESTS as u32
                let msg_ptr = get_message_ptr(idx);
                (*msg_ptr).id = idx as u32;
                // (*msg_ptr).logical_addr = 0x1;
                (*msg_ptr).logical_addr = (idx % N_REQUESTS) as u32;
                (*msg_ptr).cmd = CommandType::READ;

                // (*msg_ptr).cmd = {
                //         if idx % 2 == 0 {
                //             CommandType::READ
                //         } else {
                //             CommandType::WRITE
                //         }
                // };
                (*msg_ptr).data = DATA_TO_WRITE.as_ptr().cast_mut().cast();

                SUBMITTED += 1;
                AMOUNT_IN_LRING += 1;

                (*entry).ctx = msg_ptr as *mut c_void;
            }
        }
        return ssd_os_get_connection(c"requester", c"requester_l2p");
    }
    match LRING.dequeue_as_mut(entry) {
        Ok(entry) => {
            // println!("ok");

            let Some(req) = entry.get_ctx_as_mut::<Request>() else {
                return null_mut();
            };
            if req.status != Status::MM_DONE {
                unsafe {
                    SUBMITTED += 1;
                    AMOUNT_IN_LRING += 1;
                }
            }
            return ssd_os_get_connection(c"requester", c"requester_l2p");
        }
        Err(_) => {
            return null_mut();
            println!("{:p}", entry);
            unsafe {
                println!("{:?}", *entry);
            }
            // println!("err");
            // let workload = WORKLOAD_GENERATOR.get_mut();

            // let cur_req: Option<&mut Request> = workload.next_request();

            // match cur_req {
            //     Some(req) => {
            //         let pipe = ssd_os_get_connection(c"requester", c"requester_l2p");
            //         req.start_timer();
            //         entry.set_ctx(req);
            //         return pipe;
            //     }
            //     None => {
            //         return null_mut();
            //     }
            // }

            unsafe {
                if AMOUNT_IN_LRING < { RING_SIZE - 32 } as i32 {
                    // println!("THIS HAPPEND");
                    if let Some(idx) = get_free_message_index() {
                        // println!(idx as u32 % N_REQUESTS as u32);
                        // idx as u32 % N_REQUESTS as u32
                        let msg_ptr = get_message_ptr(idx);
                        (*msg_ptr).id = idx as u32;
                        // (*msg_ptr).logical_addr = 0x1;
                        (*msg_ptr).logical_addr = (idx % N_REQUESTS) as u32;
                        (*msg_ptr).cmd = CommandType::WRITE;

                        // (*msg_ptr).cmd = {
                        //         if idx % 2 == 0 {
                        //             CommandType::READ
                        //         } else {
                        //             CommandType::WRITE
                        //         }
                        // };
                        (*msg_ptr).data = DATA_TO_WRITE.as_ptr().cast_mut().cast();

                        SUBMITTED += 1;
                        AMOUNT_IN_LRING += 1;

                        (*entry).ctx = msg_ptr as *mut c_void;
                    }
                    return ssd_os_get_connection(c"requester", c"requester_l2p");
                } else {
                    return null_mut();
                }
            }

            // println!("STARTING CONN");
            // return ssd_os_get_connection(c"requester", c"requester_l2p");
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

    // println!("{:?}", req);
    match *req {
        Request {
            status: Status::DONE,
            ..
        } => {
            if let Some(idx) = get_index_from_ptr(req) {
                release_message(idx);
            }

            unsafe {
                COUNT += 1;
                AMOUNT_IN_LRING -= 1
            }
            req.status = Status::IN_PROCESS;
            match LRING.enqueue(entry) {
                Ok(()) => return 0,
                Err(LRingErr::Enqueue(i)) => return i,
                _ => {
                    println!("DID NOT MATCH RES FROM ENQUEUE!");
                    return -1;
                }
            }

            return 0;
        }
        Request {
            status: Status::MM_DONE,
            cmd: CommandType::WRITE,
            ..
        } => {
            // req.end_timer();
            // let workload = WORKLOAD_GENERATOR.get_mut();
            // workload.request_returned += 1;

            // #[cfg(feature = "debug")]
            // {
            //     if !req.data.is_null() {
            //         unsafe {
            //             println!("request {} data is: {:?}", req.id, req.data.as_ref());
            //         }
            //     }
            //     println!("REQUEST {} DONE!", req.id);
            //     println!(
            //         "Round trip time {} DONE!",
            //         req.calc_round_trip_time_clock_cycles()
            //     );
            // }

            // if workload.request_returned == workload.get_n_requests() {
            //     workload.calculate_stats();
            // }

            match LRING.enqueue(entry) {
                Ok(()) => return 0,
                Err(LRingErr::Enqueue(i)) => return i,
                _ => {
                    println!("DID NOT MATCH RES FROM ENQUEUE!");
                    return -1;
                }
            }
        }
        Request {
            status: Status::BAD,
            ..
        } => {
            todo!()
        }

        _ => todo!(),
    }
}
