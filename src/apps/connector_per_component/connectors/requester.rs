use core::ptr::{null, null_mut};

use alloc::vec::Vec;

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::{ssd_os_get_connection, ssd_os_sleep},
    },
    make_connector_static, println,
    shared::core_local_cell::CoreLocalCell,
};

make_connector_static!(requester, init, exit, pipe_start, ring);

static lring: LRing<128> = LRing::new();
static ALLOC: SimpleAllocator = SimpleAllocator::new();
static requests: CoreLocalCell<Vec<Result<Request, RequestError>, &SimpleAllocator>> =
    CoreLocalCell::new();
static mut requestIdx: usize = 0;

#[derive(Debug, Clone, Copy)]
pub enum CommandType {
    READ,
    WRITE,
    ERASE,
}

#[derive(Debug, Clone, Copy)]
pub struct Request {
    pub id: u32,
    pub cmd: CommandType,
    pub logical_addr: u32,
    pub physical_addr: Option<u32>,
    pub data: *mut u8,
}

#[derive(Debug, Clone, Copy)]
pub enum RequestError {
    ConnectorError,
    StageError,
}

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
    requests.get_mut().push(Ok(Request {
        id: 0,
        cmd: CommandType::WRITE,
        logical_addr: 0x1,
        physical_addr: None,
        data: null_mut(),
    }));

    requests.get_mut().push(Ok(Request {
        id: 1,
        cmd: CommandType::READ,
        logical_addr: 0x2,
        physical_addr: None,
        data: null_mut(),
    }));

    requests.get_mut().push(Ok(Request {
        id: 2,
        cmd: CommandType::WRITE,
        logical_addr: 0x2,
        physical_addr: None,
        data: null_mut(),
    }));

    requests.get_mut().push(Ok(Request {
        id: 3,
        cmd: CommandType::READ,
        logical_addr: 0x2,
        physical_addr: None,
        data: null_mut(),
    }));

    0
}

fn exit() -> ::core::ffi::c_int {
    println!("EXIT!");
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    println!("REQUESTER_PIPE_START");
    ssd_os_sleep(1);

    // 1 if there is a request in the ring, it means it's back around
    let Ok(res) = lring.dequeue_as_mut(entry) else {
        // Else we make a new request to get things started
        let Some(entry) = lring_entry::new(entry) else {
            println!("NULL PTR!");
            return null_mut();
        };

        let cur_req = requests.get_mut().get(unsafe { requestIdx });
        unsafe { requestIdx += 1 };

        match cur_req {
            Some(req) => {
                println!("REQUEST: {:?}", req);
                let pipe_1 = ssd_os_get_connection(c"requester", c"requester_l2p");
                //SET THE CTX
                entry.set_ctx(req);
                return pipe_1;
            }
            None => {
                println!("REQUESTER_PIPE_START: No request found");
                return null_mut();
            }
        }
    };
    let Some(Ok(req)) = res.get_ctx_as_mut::<Result<Request, RequestError>>() else {
        return null_mut();
    };

    // We read the result!
    println!("REQUESTER: RESULT ARRIVED BACK: {:?}", req.data);
    return null_mut();
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
