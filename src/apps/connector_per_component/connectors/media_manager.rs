use core::ptr::null_mut;

use alloc::borrow::ToOwned;

use crate::{allocator::sdd_os_alloc::SimpleAllocator, apps::connector_per_component::connectors::requester::{Request, RequestError}, bindings::{generated::{lring_entry, pipeline}, lring::{LRing, LRingErr}, mem::MemoryRegion, safe::{ssd_os_get_connection, ssd_os_sleep}, symbols::memmove}, make_connector_static, media_manager::media_manager::MediaManager, println, shared::core_local_cell::CoreLocalCell};


make_connector_static!(mm, init, exit, pipe_start, ring);

static lring: LRing<128> = LRing::new();
static ALLOC: SimpleAllocator = SimpleAllocator::new();
static MM: CoreLocalCell<MediaManager<SimpleAllocator>> = CoreLocalCell::new();


fn init() -> ::core::ffi::c_int {
    println!("MM_INIT");
    let mut mem_region = MemoryRegion::new_from_cpu(4);
    println!("{:?}", mem_region.free_start);
    println!("{:?}", mem_region.end);
    let Ok(()) = lring.init(c"MM_LRING", mem_region.free_start, 0) else {
        panic!("MM_LRING WAS ALREADY INITIALIZED!");
    };
    let ring = lring.get_lring().unwrap();
    mem_region.reserve(ring.alloc_mem as usize);

    println!("MM_LRING_INIT");
    ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());
    MM.set(MediaManager::new(&ALLOC));  
    0
}

fn exit() -> ::core::ffi::c_int {
    println!("EXIT!");
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    println!("MM_PIPE_START");
    // println!("MM_PIPE_START: {:?}", MM_mapper.get_mut().lookup(0x1));
    ssd_os_sleep(1);
    
    let Some(entry) = lring_entry::new(entry) else {
               println!("NULL PTR!");
               return null_mut();
           };

    let Ok(res) = lring.dequeue_as_mut(entry) else {
        return null_mut();
    };
    let Some(Ok(req)) = res.get_ctx_as_mut::<Result<Request, RequestError>>() else {
        return null_mut();
    };
        
    req.data = MM.get_mut().execute_request(req.to_owned(), None).unwrap();
    
    let pipe_1 = ssd_os_get_connection(c"mm", c"media_manager_requester");
    //SET THE CTX
    entry.set_ctx(req);
    return pipe_1;
    
    // return null_mut();
    
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    println!("MM_LRING");
    match lring.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}