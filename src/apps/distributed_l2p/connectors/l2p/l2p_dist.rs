use core::ffi::CStr;
use core::ptr::null_mut;

use crate::requester::requester::Status;
use crate::shared::macros::println;
use crate::{
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::ssd_os_get_connection,
    },
    make_connector_static,
    requester::requester::{CommandType, Request},
};

use super::l2p_tables::N_TABLES;

make_connector_static!(l2p_dist, init, exit, pipe_start, ring, 1);

pub const L2P_LRING_CAPACITY: usize = 128;

static LRING: LRing<L2P_LRING_CAPACITY> = LRing::new();
static PIPE_TABLE: [&CStr; N_TABLES] = [c"dist_l2p0", c"dist_l2p1", c"dist_l2p2", c"dist_l2p3"];

fn init() -> ::core::ffi::c_int {
    let mem_region = MemoryRegion::new_from_cpu(2);
    let Ok(()) = LRING.init(c"L2P_LRING", mem_region.free_start, 0) else {
        panic!("L2P_LRING WAS ALREADY INITIALIZED!");
    };

    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    let Ok(req): Result<&mut Request, LRingErr> = LRING.dequeue_as_mut_ctx(entry) else {
        return null_mut();
    };

    match req.cmd {
        CommandType::READ => {
            let pipe_name = PIPE_TABLE[req.logical_addr as usize % PIPE_TABLE.len()];
            return ssd_os_get_connection(l2p_dist.get_name(), pipe_name);
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
    match LRING.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}

fn write_handler(req: &mut Request) -> *mut pipeline {
    match *req {
        Request {
            physical_addr: None,
            ..
        } => return ssd_os_get_connection(l2p_dist.get_name(), c"l2p_prov"),
        Request {
            status: Status::PENDING | Status::IN_PROCESS,
            logical_addr,
            ..
        } => {
            let pipe_name = PIPE_TABLE[logical_addr as usize % PIPE_TABLE.len()];
            return ssd_os_get_connection(l2p_dist.get_name(), pipe_name);
        }
        _ => {
            println!("L2P DIST NO MATCH: {:?}", req);
            todo!()
        }
    }
}
