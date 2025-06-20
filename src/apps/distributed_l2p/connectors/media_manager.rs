use core::ptr::null_mut;

use crate::requester::requester::{Request, Status};
use crate::shared::macros::println;
use crate::{
    allocator::linked_list_alloc::LinkedListAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::ssd_os_get_connection,
    },
    make_connector_static,
    media_manager::media_manager::MediaManager,
    shared::core_local_cell::CoreLocalCell,
};
make_connector_static!(mm, init, exit, pipe_start, ring, 1);

static LRING: LRing<128> = LRing::new();
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
static MM: CoreLocalCell<MediaManager<LinkedListAllocator>> = CoreLocalCell::new();

fn init() -> ::core::ffi::c_int {
    let mut mem_region = MemoryRegion::new_from_cpu(4);
    let Ok(()) = LRING.init(c"MM_LRING", mem_region.free_start, 0) else {
        panic!("MM_LRING WAS ALREADY INITIALIZED!");
    };
    mem_region.reserve(LRING.get_lring().unwrap().alloc_mem as usize);

    ALLOC.set(LinkedListAllocator::new());
    ALLOC
        .get()
        .initialize(mem_region.free_start.cast(), mem_region.end.cast());
    MM.set(MediaManager::new(ALLOC.get()));
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    let Ok(req): Result<&mut Request, LRingErr> = LRING.dequeue_as_mut_ctx(entry) else {
        return null_mut();
    };

    if req.physical_addr.is_none() {
        println!("PHYSICAL IS NONE: {:?}", req);
    }

    let Ok(res) = MM.get_mut().execute_request(req) else {
        println!("MMGR ERROR!: {:?}", MM.get_mut().execute_request(req));
        req.status = Status::BAD;
        return ssd_os_get_connection(c"mm", c"media_manager_bbt");
    };
    req.data = res;
    req.status = Status::DONE;

    return ssd_os_get_connection(c"mm", c"media_manager_requester");
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    match LRING.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}
