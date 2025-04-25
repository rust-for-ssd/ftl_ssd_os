use core::ptr::null_mut;

use alloc::vec::Vec;

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::{ssd_os_get_connection, ssd_os_sleep},
    },
    make_connector_static,
    media_manager::media_manager::mm_page,
    println,
    shared::core_local_cell::CoreLocalCell,
};

use crate::requester::requester::{CommandType, Request, RequestError};

make_connector_static!(requester, init, exit, pipe_start, ring);

static lring: LRing<128> = LRing::new();
static ALLOC: SimpleAllocator = SimpleAllocator::new();
static requests: CoreLocalCell<Vec<Result<Request, RequestError>, &SimpleAllocator>> =
    CoreLocalCell::new();
static mut requestIdx: usize = 0;

static request_pages: CoreLocalCell<Vec<(usize, mm_page), &SimpleAllocator>> = CoreLocalCell::new();

pub const N_REQUESTS: usize = 128;

fn init() -> ::core::ffi::c_int {
    // println!("REQUESTER_INIT");
    let mut mem_region = MemoryRegion::new_from_cpu(1);
    let Ok(()) = lring.init(c"REQUESTER_LRING", mem_region.free_start, 0) else {
        panic!("REQUESTER_LRING WAS ALREADY INITIALIZED!");
    };
    let ring = lring.get_lring().unwrap();
    mem_region.reserve(ring.alloc_mem as usize);

    ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());

    requests.set(Vec::with_capacity_in(N_REQUESTS, &ALLOC));
    request_pages.set(Vec::with_capacity_in(N_REQUESTS, &ALLOC));
    let pages = request_pages.get_mut();
    let request = requests.get_mut();
    for i in 0..N_REQUESTS {
        pages.push((i, [i as u8, i as u8]));
        if i % 2 == 0 {
            request.push(Ok(Request {
                id: i as u32,
                cmd: CommandType::WRITE,
                logical_addr: i as u32,
                physical_addr: None,
                data: pages[i].1.as_ptr().cast_mut().cast(),
                start_time: 0,
                end_time: 0,
            }));
        } else {
            request.push(Ok(Request {
                id: i as u32,
                cmd: CommandType::READ,
                logical_addr: i as u32,
                physical_addr: None,
                data: null_mut(),
                start_time: 0,
                end_time: 0,
            }))
        }
    }
    // let mut i = 0;
    // req_pages.push((i, [0, 0]));
    // requests.get_mut().push(Ok(Request {
    //     id: i as u32,
    //     cmd: CommandType::WRITE,
    //     logical_addr: 0x1,
    //     physical_addr: None,
    //     data: req_pages[i].1.as_ptr().cast_mut().cast(),
    // }));

    // i += 1;
    // req_pages.push((i, [1, 1]));
    // requests.get_mut().push(Ok(Request {
    //     id: i as u32,
    //     cmd: CommandType::WRITE,
    //     logical_addr: 0x2,
    //     physical_addr: None,
    //     data: req_pages[i].1.as_ptr().cast_mut().cast(),
    // }));

    // requests.get_mut().push(Ok(Request {
    //     id: 2,
    //     cmd: CommandType::READ,
    //     logical_addr: 0x1,
    //     physical_addr: None,
    //     data: null_mut(),
    // }));

    // requests.get_mut().push(Ok(Request {
    //     id: 3,
    //     cmd: CommandType::READ,
    //     logical_addr: 0x2,
    //     physical_addr: None,
    //     data: null_mut(),
    // }));

    // println!("REQUESTER_INIT_END");
    0
}

fn exit() -> ::core::ffi::c_int {
    // println!("EXIT!");
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    ssd_os_sleep(1);

    // 1 if there is a request in the ring, it means it's back around
    let Ok(res) = lring.dequeue_as_mut(entry) else {
        // Else we make a new request to get things started
        let Some(entry) = lring_entry::new(entry) else {
            println!("NULL PTR!");
            return null_mut();
        };

        let cur_req : Option<&mut Result<Request, RequestError>> = requests.get_mut().get_mut(unsafe { requestIdx });
        unsafe { requestIdx += 1 };

        match cur_req {
            Some(mut req) => {
                let pipe_1 = ssd_os_get_connection(c"requester", c"requester_l2p");
                
                match req {
                    Ok(elem) => {
                        elem.start_timer();

                    },
                    Err(_) => todo!(),
                }
               
                // Start the timer!
                (*req).unwrap().start_timer();
                
                // println!("Start value!!! request: {:?}", (*req).unwrap().start_time);
                
                
                entry.set_ctx(req);
                return pipe_1;
            }
            None => {
                return null_mut();
            }
        }
    };
    return null_mut();
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    ssd_os_sleep(1);
    let res = lring_entry::new(entry).unwrap();
    let Some(Ok(req)) = res.get_ctx_as_mut::<Result<Request, RequestError>>() else {
        return 0;
    };
    
    // stop timer 
    req.end_timer();
    

    if !req.data.is_null() {
        unsafe {
            println!("request {} data is: {:?}", req.id, req.data.as_ref());
        }
    }
    println!("REQUEST {} DONE!", req.id);
    println!("Round trip time {} DONE!", req.calc_round_trip_time_ms());

    // match lring.enqueue(entry) {
    //     Ok(()) => {
    //         let res = lring_entry::new(entry).unwrap();
    //         let Some(Ok(req)) = res.get_ctx_as_mut::<Result<Request, RequestError>>() else {
    //             return 0;
    //         };
    //         unsafe {
    //             println!("request data is: {:?}", req.data.as_ref());
    //         }
    //         0
    //     }
    //     Err(LRingErr::Enqueue(i)) => i,
    //     _ => {
    //         println!("DID NOT MATCH RES FROM ENQUEUE!");
    //         -1
    //     }
    // }
    0
}
