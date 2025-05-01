use core::ptr::null_mut;

use crate::{
    allocator::linked_list_alloc::LinkedListAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::ssd_os_get_connection,
    },
    l2p::l2p::L2pMapper,
    make_connector_static, println,
    requester::requester::{CommandType, Request},
    shared::core_local_cell::CoreLocalCell,
};

make_connector_static!(l2p, init, exit, pipe_start, ring);

static lring: LRing<128> = LRing::new();
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
static l2p_mapper: CoreLocalCell<L2pMapper<LinkedListAllocator>> = CoreLocalCell::new();

fn init() -> ::core::ffi::c_int {
    let mut mem_region = MemoryRegion::new_from_cpu(2);
    let Ok(()) = lring.init(c"L2P_LRING", mem_region.free_start, 0) else {
        panic!("L2P_LRING WAS ALREADY INITIALIZED!");
    };
    mem_region.reserve(lring.get_lring().unwrap().alloc_mem as usize);

    ALLOC.set(LinkedListAllocator::new());
    ALLOC
        .get()
        .initialize(mem_region.free_start.cast(), mem_region.end.cast());
    l2p_mapper.set(L2pMapper::new(ALLOC.get()));

    #[cfg(feature = "benchmark")]
    {
        let n_requests = super::requester::WORKLOAD_GENERATOR.get().get_n_requests();
        let l2p_map = l2p_mapper.get_mut();
        l2p_map.prepare_for_benchmark(n_requests);
    }

    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    let Ok(res) = lring.dequeue_as_mut(entry) else {
        return null_mut();
    };

    let Some(req) = res.get_ctx_as_mut::<Request>() else {
        return null_mut();
    };

    match req.cmd {
        CommandType::READ => {
            req.physical_addr = l2p_mapper.get_mut().lookup(req.logical_addr);
            return ssd_os_get_connection(c"l2p", c"l2p_media_manager");
        }
        CommandType::WRITE if req.physical_addr.is_none() => {
            return ssd_os_get_connection(c"l2p", c"l2p_prov");
        }
        CommandType::WRITE if req.physical_addr.is_some() => {
            // WARNING: ASSUMING that the physical addr is only set from the provisioner in the write path.
            l2p_mapper
                .get_mut()
                .map(req.logical_addr, req.physical_addr.unwrap());

            return ssd_os_get_connection(c"l2p", c"l2p_media_manager");
        }
        _ => {
            println!("UNEXPECTED MATCH IN L2P");
            return null_mut();
        }
    }
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    match lring.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}
