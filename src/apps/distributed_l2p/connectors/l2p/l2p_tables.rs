use core::ffi::CStr;
use core::ptr::null_mut;

use crate::requester::requester::Status;
use crate::shared::macros::println;
use crate::{
    allocator::linked_list_alloc::LinkedListAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::ssd_os_get_connection,
    },
    l2p::l2p::L2pMapper,
    make_connector_static,
    requester::requester::{CommandType, Request},
    shared::core_local_cell::CoreLocalCell,
};

make_connector_static!(l2p_0, init0, exit, pipe_start0, ring0, 1);
make_connector_static!(l2p_1, init1, exit, pipe_start1, ring1, 1);
make_connector_static!(l2p_2, init2, exit, pipe_start2, ring2, 1);
make_connector_static!(l2p_3, init3, exit, pipe_start3, ring3, 1);

const CONN_NAMES: [&CStr; N_TABLES] = [
    l2p_0.get_name(),
    l2p_1.get_name(),
    l2p_2.get_name(),
    l2p_3.get_name(),
];
const L2P_LRING_CAPACITY: usize = 128;
const START_CPU_MEM_REGION: i32 = 6;
pub const N_TABLES: usize = 4;

static LRINGS: [LRing<L2P_LRING_CAPACITY>; N_TABLES] = [const { LRing::new() }; N_TABLES];
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
static L2P_MAPS: [CoreLocalCell<L2pMapper<LinkedListAllocator>>; N_TABLES] =
    [const { CoreLocalCell::new() }; N_TABLES];

fn init(id: i32) -> i32 {
    let mut mem_region = MemoryRegion::new_from_cpu(START_CPU_MEM_REGION + id);
    let Ok(()) = LRINGS[id as usize].init(c"L2P_LRING", mem_region.free_start, 0) else {
        panic!("L2P_LRING WAS ALREADY INITIALIZED!");
    };
    mem_region.reserve(LRINGS[id as usize].get_lring().unwrap().alloc_mem as usize);

    ALLOC.set(LinkedListAllocator::new());
    ALLOC
        .get()
        .initialize(mem_region.free_start.cast(), mem_region.end.cast());
    L2P_MAPS[id as usize].set(L2pMapper::new(ALLOC.get()));

    // #[cfg(feature = "benchmark")]
    // {
    //     let n_requests = super::requester::WORKLOAD_GENERATOR.get().get_n_requests();
    //     let l2p_map = L2P_MAPPER.get_mut();
    //     l2p_map.prepare_for_benchmark(n_requests);
    // }

    0
}

fn init0() -> ::core::ffi::c_int {
    init(0)
}
fn init1() -> ::core::ffi::c_int {
    init(1)
}
fn init2() -> ::core::ffi::c_int {
    init(2)
}
fn init3() -> ::core::ffi::c_int {
    init(3)
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn pipe_start0(entry: *mut lring_entry) -> *mut pipeline {
    pipe_start(0, entry)
}
fn pipe_start1(entry: *mut lring_entry) -> *mut pipeline {
    pipe_start(1, entry)
}
fn pipe_start2(entry: *mut lring_entry) -> *mut pipeline {
    pipe_start(2, entry)
}
fn pipe_start3(entry: *mut lring_entry) -> *mut pipeline {
    pipe_start(3, entry)
}

fn pipe_start(id: usize, entry: *mut lring_entry) -> *mut pipeline {
    let Ok(res) = LRINGS[id].dequeue_as_mut(entry) else {
        return null_mut();
    };

    let Some(req) = res.get_ctx_as_mut::<Request>() else {
        return null_mut();
    };
    match *req {
        Request {
            cmd: CommandType::READ,
            logical_addr,
            ..
        } => {
            req.physical_addr = L2P_MAPS[id].get_mut().lookup(logical_addr);
            return get_mmgr_conn(id);
        }
        Request {
            cmd: CommandType::WRITE,
            status: Status::PENDING | Status::IN_PROCESS,
            logical_addr,
            physical_addr: Some(ppa),
            ..
        } => {
            // WARNING: ASSUMING that the physical addr is only set from the provisioner in the write path.
            L2P_MAPS[id].get_mut().map(logical_addr, ppa);

            return get_mmgr_conn(id);
        }
        Request {
            cmd: CommandType::WRITE,
            status: Status::BAD | Status::MM_DONE,
            logical_addr,
            ..
        } => {
            L2P_MAPS[id].get_mut().unmap(logical_addr);
            req.status = Status::DONE;
            return get_mmgr_conn(id);
            // TODO: should free the physical address
            // maybe by going to GC?
            // return null_mut();
        }
        _ => todo!(),
    }
}

const MMGR_PIPES: [&CStr; N_TABLES] = [
    c"l2p_media_manager0",
    c"l2p_media_manager1",
    c"l2p_media_manager2",
    c"l2p_media_manager3",
];

fn get_mmgr_conn(id: usize) -> *mut pipeline {
    ssd_os_get_connection(CONN_NAMES[id], MMGR_PIPES[id])
}

const PROV_PIPES: [&CStr; N_TABLES] = [c"l2p_prov0", c"l2p_prov1", c"l2p_prov2", c"l2p_prov3"];

fn get_prov_conn(id: usize) -> *mut pipeline {
    ssd_os_get_connection(CONN_NAMES[id], PROV_PIPES[id])
}

fn ring0(entry: *mut lring_entry) -> i32 {
    ring(0, entry)
}
fn ring1(entry: *mut lring_entry) -> i32 {
    ring(1, entry)
}
fn ring2(entry: *mut lring_entry) -> i32 {
    ring(2, entry)
}
fn ring3(entry: *mut lring_entry) -> i32 {
    ring(3, entry)
}

fn ring(id: usize, entry: *mut lring_entry) -> ::core::ffi::c_int {
    let Some(entry) = lring_entry::new(entry) else {
        panic!("null entry");
    };

    let Some(req) = entry.get_ctx_as_mut::<Request>() else {
        panic!("null ctx");
    };

    match LRINGS[id].enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}
