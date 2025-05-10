use core::ffi::CStr;
use core::ptr::null_mut;

use crate::requester::requester::{META_DATA, Status};
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
    requester::requester::{CommandType, Request},
    shared::core_local_cell::CoreLocalCell,
};

use super::l2p_dist_table::L2PDistributionTable;
use super::l2p_tables::N_TABLES;

make_connector_static!(l2p_dist, init, exit, pipe_start, ring, 1);

pub const L2P_LRING_CAPACITY: usize = 128;

static LRING: LRing<L2P_LRING_CAPACITY> = LRing::new();
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
static DIST_TABLE: CoreLocalCell<L2PDistributionTable<LinkedListAllocator, { PIPE_TABLE.len() }>> =
    CoreLocalCell::new();
static PIPE_TABLE: [&CStr; N_TABLES] = [c"dist_l2p0", c"dist_l2p1", c"dist_l2p2", c"dist_l2p3"];

fn init() -> ::core::ffi::c_int {
    let mut mem_region = MemoryRegion::new_from_cpu(2);
    let Ok(()) = LRING.init(c"L2P_LRING", mem_region.free_start, 0) else {
        panic!("L2P_LRING WAS ALREADY INITIALIZED!");
    };
    mem_region.reserve(LRING.get_lring().unwrap().alloc_mem as usize);

    ALLOC.set(LinkedListAllocator::new());
    ALLOC
        .get()
        .initialize(mem_region.free_start.cast(), mem_region.end.cast());
    DIST_TABLE.set(L2PDistributionTable::new(ALLOC.get(), PIPE_TABLE));

    #[cfg(feature = "benchmark")]
    {
        let n_requests = crate::apps::distributed_l2p::connectors::requester::WORKLOAD_GENERATOR
            .get()
            .get_n_requests();
        DIST_TABLE.get_mut().prepare_for_benchmark(n_requests);
        // let l2p_map = L2P_MAPPER.get_mut();
        // l2p_map.prepare_for_benchmark(n_requests);
    }

    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    let Ok(res) = LRING.dequeue_as_mut(entry) else {
        return null_mut();
    };

    let Some(req) = res.get_ctx_as_mut::<Request>() else {
        return null_mut();
    };

    match req.cmd {
        CommandType::READ => {
            return read_handler(req);
        }
        CommandType::WRITE => {
            return write_handler(req);
        }
        _ => {
            println!("UNEXPECTED MATCH IN L2P");
            return null_mut();
        }
    }
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    // TODO: should add 2 requests if comming from provisioner
    match LRING.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}

fn read_handler(req: &Request) -> *mut pipeline {
    let Some(table_idx) = DIST_TABLE.get().get_table_idx(&req.logical_addr) else {
        println!("READ FROM UNMAPPED ADDR");
        return null_mut();
    };
    return ssd_os_get_connection(
        l2p_dist.get_name(),
        DIST_TABLE.get().get_table_pipe_name(table_idx),
    );
}

fn write_handler(req: &mut Request) -> *mut pipeline {
    match *req {
        Request {
            physical_addr: None,
            ..
        } => return ssd_os_get_connection(l2p_dist.get_name(), c"l2p_prov"),
        Request {
            status: Status::MM_DONE,
            md: META_DATA::L2P_OLD_NEW_ID((Some(old), _)),
            ..
        } => {
            return ssd_os_get_connection(
                l2p_dist.get_name(),
                DIST_TABLE.get().get_table_pipe_name(old),
            );
        }
        Request {
            status: Status::MM_DONE,
            md: META_DATA::L2P_OLD_NEW_ID((None, _)),
            ..
        } => {
            return null_mut();
        }
        Request {
            status: Status::BAD,
            md: META_DATA::L2P_OLD_NEW_ID((Some(old), new)),
            logical_addr,
            ..
        } => {
            DIST_TABLE.get_mut().set_table_idx(logical_addr, old);
            return ssd_os_get_connection(
                l2p_dist.get_name(),
                DIST_TABLE.get().get_table_pipe_name(new),
            );
        }
        Request {
            status: Status::PENDING | Status::IN_PROCESS,
            logical_addr,
            ..
        } => {
            let tbl_id = DIST_TABLE.get_mut().pick_table_idx();
            if let Some(prev_id) = DIST_TABLE.get_mut().set_table_idx(logical_addr, tbl_id) {
                req.md = META_DATA::L2P_OLD_NEW_ID((Some(prev_id), tbl_id));
            } else {
                req.md = META_DATA::L2P_OLD_NEW_ID((None, tbl_id));
            };
            return ssd_os_get_connection(
                l2p_dist.get_name(),
                DIST_TABLE.get().get_table_pipe_name(tbl_id),
            );
        }
        _ => {
            println!("{:?}", req);
            todo!()
        }
    }
}
