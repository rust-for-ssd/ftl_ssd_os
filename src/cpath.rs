use core::{mem::MaybeUninit, ptr::null_mut};

use alloc::{boxed::Box, vec::Vec};

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator,
    bbt::bbt::{BadBlockStatus, BadBlockTable},
    bindings::{
        generated::{lring_entry, nvm_mmgr_geometry, pipeline, volt_get_geometry},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::{ssd_os_get_connection, ssd_os_sleep},
    },
    make_connector_static, make_stage_static, println,
    shared::addresses::PhysicalBlockAddress,
};

make_stage_static!(stage_1, s1, s1, stage_1_fn);
make_stage_static!(stage_2, s1, s1, stage_2_fn);
fn s1() -> ::core::ffi::c_int {
    0
}
fn stage_1_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("STAGE_1");
    println!(context as u32);
    context
}

fn stage_2_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("STAGE_2");
    println!(context as u32);
    context
}
static ALLOC_1: SimpleAllocator = SimpleAllocator::new();
static ALLOC_2: SimpleAllocator = SimpleAllocator::new();
static BBT: BadBlockTable<SimpleAllocator> = BadBlockTable::new();
static mut pipe_1: *mut pipeline = null_mut();
static mut pipe_2: *mut pipeline = null_mut();
static conn2_lring: LRing<128> = LRing::new();

static reqs: [BBTReq; 3] = {
    let pba = PhysicalBlockAddress {
        channel: 0,
        lun: 0,
        plane: 0,
        block: 0,
    };
    [
        BBTReq {
            req_type: BBTRequestType::Get,
            pba,
        },
        BBTReq {
            req_type: BBTRequestType::Set,
            pba,
        },
        BBTReq {
            req_type: BBTRequestType::Get,
            pba,
        },
    ]
};

static mut status_res: Vec<BadBlockStatus, &SimpleAllocator> = Vec::new_in(&ALLOC_2);

make_connector_static!(conn_1, init_1, exit_1, conn_fn_1, ring_1);
make_connector_static!(conn_2, init_2, exit_2, conn_fn_2, ring_2);
fn init_1() -> ::core::ffi::c_int {
    println!("INIT_1");
    let mem_region = MemoryRegion::new(c"conn_1");
    ALLOC_1.initialize(mem_region.free_start.cast(), mem_region.end.cast());
    let b: Box<u32, &SimpleAllocator> = Box::new_in(69, &ALLOC_1);
    println!(*b);

    println!("INIT_1_DONE");

    0
}

fn init_2() -> ::core::ffi::c_int {
    println!("INIT_2");
    let mut mem_region = MemoryRegion::new(c"conn_2");
    let Ok(()) = conn2_lring.init(c"CONN2_LRING", mem_region.free_start, 0) else {
        panic!("RING WAS ALREADY INITIALIZED!");
    };
    let ring = conn2_lring.get_lring().unwrap();
    mem_region.reserve(ring.alloc_mem as usize);

    println!("INIT_2_ALLOC_INIT");
    ALLOC_2.initialize(mem_region.free_start.cast(), mem_region.end.cast());
    let b: Box<u32, &SimpleAllocator> = Box::new_in(69, &ALLOC_2);
    println!(*b);

    unsafe { status_res = Vec::with_capacity_in(3, &ALLOC_2) };
    let geo: MaybeUninit<nvm_mmgr_geometry> = MaybeUninit::uninit();

    unsafe { volt_get_geometry(geo.as_ptr().cast_mut()) };

    let geo = unsafe { geo.assume_init() };
    let _ = BBT.init(&geo, &ALLOC_2);

    println!("INIT_2_DONE");
    0
}

fn exit_1() -> ::core::ffi::c_int {
    println!("EXIT_1!");
    0
}
fn exit_2() -> ::core::ffi::c_int {
    println!("EXIT_2!");
    0
}

fn ring_1(entry: *mut lring_entry) -> ::core::ffi::c_int {
    println!("RING_1");
    let entry = lring_entry::new(entry).unwrap();
    let status = entry.get_ctx_as_ref().unwrap();
    match *status {
        BadBlockStatus::Good => println!("RECIVED GOOD"),
        BadBlockStatus::Bad => println!("RECIVED BAD"),
        _ => println!("NO MATCH"),
    }
    println!(*status as u32);
    0
}

fn ring_2(entry: *mut lring_entry) -> ::core::ffi::c_int {
    println!("RING_2");
    match conn2_lring.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}

static mut idx: usize = 0;
fn conn_fn_1(entry: *mut lring_entry) -> *mut pipeline {
    println!("CON_FN_1");
    if unsafe { idx } == 0 {
        ssd_os_sleep(5);
    }
    ssd_os_sleep(1);
    unsafe {
        println!(idx as u32);
    }

    if unsafe { idx } < 3 {
        let Some(entry) = lring_entry::new(entry) else {
            println!("NULL PTR!");
            return null_mut();
        };

        let ctx: &BBTReq = unsafe { reqs.get(idx).unwrap() };
        unsafe { idx += 1 };
        entry.set_ctx(ctx);
        if unsafe { pipe_1.is_null() } {
            unsafe { pipe_1 = ssd_os_get_connection(c"conn_1", c"pipe_1") };
        }
        return unsafe { pipe_1 };
    } else {
        return null_mut();
    }
}

fn conn_fn_2(entry: *mut lring_entry) -> *mut pipeline {
    println!("CON_FN_2");
    ssd_os_sleep(1);
    let Ok(res) = conn2_lring.dequeue_as_mut(entry) else {
        return null_mut();
    };
    let Some(req) = res.get_ctx_as_ref::<BBTReq>() else {
        return null_mut();
    };

    match req.req_type {
        BBTRequestType::Get => {
            let ctx = BBT.get_block_status(&req.pba);
            match ctx {
                BadBlockStatus::Good => println!("SENDING GOOD"),
                BadBlockStatus::Bad => println!("SENDING BAD"),
                BadBlockStatus::Reserved => println!("SENDING RESERVED"),
            }
            unsafe { status_res.push(ctx) };
            res.set_ctx(unsafe { status_res.get(status_res.len() - 1).unwrap() });

            if unsafe { pipe_2.is_null() } {
                unsafe { pipe_2 = ssd_os_get_connection(c"conn_2", c"pipe_2") };
            }
            ssd_os_sleep(1);
            return unsafe { pipe_2 };
        }
        BBTRequestType::Set => {
            BBT.set_bad_block(&req.pba);
            null_mut()
        }
    }
}

#[derive(Copy, Clone)]
enum BBTRequestType {
    Get,
    Set,
}

#[derive(Copy, Clone)]
struct BBTReq {
    req_type: BBTRequestType,
    pba: PhysicalBlockAddress,
}
