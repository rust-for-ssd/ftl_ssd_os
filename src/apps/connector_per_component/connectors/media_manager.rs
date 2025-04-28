use core::ptr::null_mut;

use crate::apps::connector_per_component::connectors::requester::N_REQUESTS;
use crate::media_manager::media_manager::mm_page;
use crate::requester::requester::{CommandType, Request, RequestError};
use crate::{
    allocator::sdd_os_alloc::SimpleAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::{ssd_os_get_connection, ssd_os_sleep},
    },
    make_connector_static,
    media_manager::media_manager::MediaManager,
    println,
    shared::core_local_cell::CoreLocalCell,
};
make_connector_static!(mm, init, exit, pipe_start, ring);

static lring: LRing<128> = LRing::new();
static ALLOC: SimpleAllocator = SimpleAllocator::new();
static MM: CoreLocalCell<MediaManager<SimpleAllocator>> = CoreLocalCell::new();

fn init() -> ::core::ffi::c_int {
    // println!("MM_INIT_START");
    let mut mem_region = MemoryRegion::new_from_cpu(4);
    let Ok(()) = lring.init(c"MM_LRING", mem_region.free_start, 0) else {
        panic!("MM_LRING WAS ALREADY INITIALIZED!");
    };
    let ring = lring.get_lring().unwrap();
    mem_region.reserve(ring.alloc_mem as usize);

    ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());
    MM.set(MediaManager::new(&ALLOC));
    let mmgr = MM.get_mut();
    for i in 0..N_REQUESTS {
        static arr: mm_page = [0, 0];
        let _ = mmgr.execute_request(&Request {
            id: i as u32,
            cmd: CommandType::WRITE,
            logical_addr: i as u32,
            physical_addr: Some(i as u32),
            data: arr.as_ptr().cast_mut().cast(),
            start_time: 0,
            end_time: 0,
        });
    }
    // println!("MM_INIT_END");
    0
}

fn exit() -> ::core::ffi::c_int {
    println!("EXIT!");
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    ssd_os_sleep(1);

    let Ok(res) = lring.dequeue_as_mut(entry) else {
        return null_mut();
    };
    let Some(Ok(req)) = res.get_ctx_as_mut::<Result<Request, RequestError>>() else {
        return null_mut();
    };

    let Ok(res) = MM.get_mut().execute_request(req) else {
        println!("MM ERROR!: {:?}", MM.get_mut().execute_request(req));
        return null_mut();
    };
    req.data = res;

    return ssd_os_get_connection(c"mm", c"media_manager_requester");
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    ssd_os_sleep(1);
    match lring.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}
