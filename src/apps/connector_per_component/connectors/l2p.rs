use core::ptr::null_mut;

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator,
    apps::connector_per_component::connectors::requester::N_REQUESTS,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::{ssd_os_get_connection, ssd_os_sleep},
    },
    l2p::l2p::L2pMapper,
    make_connector_static, println,
    requester::requester::{CommandType, Request, RequestError},
    shared::core_local_cell::CoreLocalCell,
};

make_connector_static!(l2p, init, exit, pipe_start, ring);

static lring: LRing<128> = LRing::new();
static ALLOC: SimpleAllocator = SimpleAllocator::new();
static l2p_mapper: CoreLocalCell<L2pMapper<SimpleAllocator>> = CoreLocalCell::new();

fn init() -> ::core::ffi::c_int {
    // println!("L2P_INIT_START");
    let mut mem_region = MemoryRegion::new_from_cpu(2);
    let Ok(()) = lring.init(c"L2P_LRING", mem_region.free_start, 0) else {
        panic!("L2P_LRING WAS ALREADY INITIALIZED!");
    };
    let ring = lring.get_lring().unwrap();
    mem_region.reserve(ring.alloc_mem as usize);

    ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());
    l2p_mapper.set(L2pMapper::new(&ALLOC));
    // l2p_mapper.get_mut().map(0x1, 0x1234);
    // l2p_mapper.get_mut().map(0x2, 0x1111);
    let l2p_map = l2p_mapper.get_mut();
    for i in 0..N_REQUESTS {
        let i = i as u32;
        l2p_map.map(i, i);
    }

    // println!("L2P_LRING_INIT_END");
    0
}

fn exit() -> ::core::ffi::c_int {
    // println!("EXIT!");
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    // ssd_os_sleep(1);

    let Ok(res) = lring.dequeue_as_mut(entry) else {
        return null_mut();
    };
    let Some(req) = res.get_ctx_as_mut::<Request>() else {
        return null_mut();
    };

    // println!("l2p request: {:?}", req);
    match req.cmd {
        CommandType::READ => {
            req.physical_addr = l2p_mapper.get_mut().lookup(req.logical_addr);
            return ssd_os_get_connection(c"l2p", c"l2p_media_manager");
        }
        CommandType::WRITE if req.physical_addr.is_none() => {
            // println!("l2p -> prov");
            return ssd_os_get_connection(c"l2p", c"l2p_prov");
        }
        CommandType::WRITE if req.physical_addr.is_some() => {
            // println!("l2p -> mm");
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
    // ssd_os_sleep(1);
    let res = lring_entry::new(entry).unwrap();
    let Some(req) = res.get_ctx_as_mut::<Request>() else {
        return 0;
    };
    // println!("l2p recived: {:?}", req);
    match lring.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}
