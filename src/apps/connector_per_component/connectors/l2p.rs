use core::ptr::null_mut;

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator,
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
    println!("L2P_INIT");
    let mut mem_region = MemoryRegion::new_from_cpu(2);
    println!("{:?}", mem_region.free_start);
    println!("{:?}", mem_region.end);
    let Ok(()) = lring.init(c"L2P_LRING", mem_region.free_start, 0) else {
        panic!("L2P_LRING WAS ALREADY INITIALIZED!");
    };
    let ring = lring.get_lring().unwrap();
    mem_region.reserve(ring.alloc_mem as usize);

    println!("L2P_LRING_INIT");
    ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());
    println!("L2P_LRING_INITEND");
    l2p_mapper.set(L2pMapper::new(&ALLOC));
    println!("L2P_LRING_INITEND");
    l2p_mapper.get_mut().map(0x1, 0x1234);
    println!("L2P_LRING_INITEND");
    l2p_mapper.get_mut().map(0x2, 0x1111);

    println!("L2P_LRING_INITEND");
    0
}

fn exit() -> ::core::ffi::c_int {
    println!("EXIT!");
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    println!("L2P_PIPE_START");
    // println!("L2P_PIPE_START: {:?}", l2p_mapper.get_mut().lookup(0x1));
    ssd_os_sleep(1);

    println!("A");
    let Ok(res) = lring.dequeue_as_mut(entry) else {
        return null_mut();
    };
    println!("B");
    let Some(Ok(req)) = res.get_ctx_as_mut::<Result<Request, RequestError>>() else {
        return null_mut();
    };

    println!("C");
    match req.cmd {
        CommandType::READ => {
            req.physical_addr = l2p_mapper.get_mut().lookup(req.logical_addr);
            return ssd_os_get_connection(c"l2p", c"l2p_media_manager");
        }
        CommandType::WRITE if req.physical_addr.is_none() => {
            println!("No PPA! {:?}", req);
            return ssd_os_get_connection(c"l2p", c"l2p_prov");
        }
        CommandType::WRITE if req.physical_addr.is_some() => {
            println!("PPA! {:?}", req);
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
    println!("L2P_LRING");
    match lring.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}
