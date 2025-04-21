use core::ptr::null_mut;

use alloc::vec::Vec;

use crate::{allocator::sdd_os_alloc::SimpleAllocator, bindings::{generated::{lring_entry, pipeline}, lring::{LRing, LRingErr}, mem::MemoryRegion, safe::{ssd_os_get_connection, ssd_os_sleep}}, make_connector_static, println, shared::core_local_cell::CoreLocalCell};


make_connector_static!(requester, init, exit, pipe_start, ring);

static lring: LRing<128> = LRing::new();
static ALLOC: SimpleAllocator = SimpleAllocator::new();
static requests: CoreLocalCell<Vec<i32, &SimpleAllocator>> = CoreLocalCell::new();


fn init() -> ::core::ffi::c_int {
    println!("REQUESTER_INIT");
    let mut mem_region = MemoryRegion::new_from_cpu(1);
    let Ok(()) = lring.init(c"REQUESTER_LRING", mem_region.free_start, 0) else {
        panic!("REQUESTER_LRING WAS ALREADY INITIALIZED!");
    };
    let ring = lring.get_lring().unwrap();
    mem_region.reserve(ring.alloc_mem as usize);

    println!("LRING_INIT");
    ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());
    println!("{:?}", mem_region.free_start);
    println!("{:?}", mem_region.end);

    
    requests.set(Vec::new_in(&ALLOC));
    requests.get_mut().push(99);
    
    0
}

fn exit() -> ::core::ffi::c_int {
    println!("EXIT!");
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    println!("REQUESTER_PIPE_START");
    ssd_os_sleep(1);
    
    let req_1 = requests.get_mut().pop();
    match req_1 {
        Some(req_1) => {
            println!("REQUESTER_PIPE_START: {:?}", req_1);
            let pipe_1 = ssd_os_get_connection(c"requester", c"requester_l2p");
            // TODO: SET THE CTX
            return pipe_1;
        },
        None => {
            println!("REQUESTER_PIPE_START: No request found");
            return null_mut();
        }
    }
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    println!("REQUESTER_LRING");
    match lring.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}