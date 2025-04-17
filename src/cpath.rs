use core::{alloc::Allocator, mem::MaybeUninit, ptr::null_mut};

use alloc::boxed::Box;

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator,
    bbt::bbt::{BadBlockStatus, BadBlockTable},
    bindings::{
        generated::{
            lring, lring_entry, nvm_mmgr_geometry, pipeline, ssd_os_lring_create,
            ssd_os_lring_dequeue, ssd_os_lring_enqueue, volt_get_geometry,
        },
        safe::{
            ssd_os_get_connection, ssd_os_mem_get, ssd_os_mem_size, ssd_os_sleep, ssd_os_this_cpu,
        },
    },
    make_connector_static, make_stage_static, println_i, println_s,
    shared::addresses::PhysicalBlockAddress,
};

make_stage_static!(stage_1, s1, s1, stage_1_fn);
make_stage_static!(stage_2, s1, s1, stage_2_fn);
fn s1() -> ::core::ffi::c_int {
    0
}
fn stage_1_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println_s!(c"STAGE_1");
    println_i!(context as u32);
    // unsafe { context.add(1) }
    context
}

fn stage_2_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println_s!(c"STAGE_2");
    println_i!(context as u32);
    // unsafe { context.add(1) }
    context
}
make_connector_static!(conn_1, init_1, exit_1, conn_fn_1, ring_1);
make_connector_static!(conn_2, init_2, exit_2, conn_fn_2, ring_2);
fn init_1() -> ::core::ffi::c_int {
    println_s!(c"INIT_1");
    0
}

static BBT_ALLOCATOR: SimpleAllocator = SimpleAllocator::new();
static BBT: BadBlockTable<SimpleAllocator> = BadBlockTable::new();

fn init_2() -> ::core::ffi::c_int {
    println_s!(c"INIT_2");
    let cpu = ssd_os_this_cpu(c"conn_2");
    let mem = ssd_os_mem_get(cpu);

    unsafe { conn2_lring = ssd_os_lring_create(c"CONN2_LRING".as_ptr().cast_mut(), 128, mem, 0x0) };

    let ring = unsafe { conn2_lring.as_ref().unwrap() };
    let start: *mut u8 = unsafe { mem.byte_add(ring.alloc_mem as usize).cast() };

    let start = unsafe { start.byte_add(start.align_offset(8)) };
    let end: *mut u8 = unsafe { mem.byte_add(ssd_os_mem_size(cpu) as usize).cast() };

    println_s!(c"INIT_2_ALLOC_INIT");
    BBT_ALLOCATOR.initialize(start, end);
    let b: Box<u32, &SimpleAllocator> = Box::new_in(69, &BBT_ALLOCATOR);
    println_i!(*b);

    let mut geo: MaybeUninit<nvm_mmgr_geometry> = MaybeUninit::uninit();

    unsafe { volt_get_geometry(geo.as_ptr().cast_mut()) };

    let geo = unsafe { geo.assume_init() };
    let _ = BBT.init(&geo, &BBT_ALLOCATOR);

    println_s!(c"INIT_2_DONE");
    0
}

fn exit_1() -> ::core::ffi::c_int {
    println_s!(c"EXIT_1!");
    0
}
fn exit_2() -> ::core::ffi::c_int {
    println_s!(c"EXIT_2!");
    0
}

fn ring_1(entry: *mut lring_entry) -> ::core::ffi::c_int {
    println_s!(c"RING_1");
    let ctx: *mut BadBlockStatus = unsafe { entry.as_ref().unwrap().ctx.cast() };
    let status: &BadBlockStatus = unsafe { ctx.as_ref().unwrap() };
    match *status {
        BadBlockStatus::Good => println_s!(c"RECIVED GOOD"),
        BadBlockStatus::Bad => println_s!(c"RECIVED BAD"),
        _ => println_s!(c"NO MATCH"),
    }
    unsafe {
        println_i!(entry.as_mut().unwrap().ctx as u32);
    }
    0
}

fn ring_2(entry: *mut lring_entry) -> ::core::ffi::c_int {
    println_s!(c"RING_2");
    return unsafe { ssd_os_lring_enqueue(conn2_lring, entry) };
}

static mut pipe_1: *mut pipeline = null_mut();
static mut pipe_2: *mut pipeline = null_mut();
static mut conn2_lring: *mut lring = null_mut();

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

static mut idx: usize = 0;

static mut idx_2: usize = 0;
static mut status_res: [BadBlockStatus; 3] = [BadBlockStatus::Reserved; 3];

fn conn_fn_1(entry: *mut lring_entry) -> *mut pipeline {
    println_s!(c"CON_FN_1");
    if unsafe { idx } == 0 {
        ssd_os_sleep(5);
    }
    ssd_os_sleep(1);
    unsafe {
        println_i!(idx as u32);
    }

    if unsafe { idx } < reqs.len() {
        let ctx: *mut BBTReq = unsafe { reqs.get_unchecked(idx) as *const BBTReq }.cast_mut();
        unsafe {
            idx = idx + 1;
        }
        unsafe { entry.as_mut().unwrap().ctx = ctx.cast() };
        if unsafe { pipe_1.is_null() } {
            unsafe {
                pipe_1 = ssd_os_get_connection(
                    c"conn_1".as_ptr().cast_mut(),
                    c"pipe_1".as_ptr().cast_mut(),
                )
            };
        }
        return unsafe { pipe_1 };
    } else {
        return null_mut();
    }
}

fn conn_fn_2(entry: *mut lring_entry) -> *mut pipeline {
    println_s!(c"CON_FN_2");
    ssd_os_sleep(1);
    let res = unsafe { ssd_os_lring_dequeue(conn2_lring, entry) };
    if res == -1 {
        return null_mut();
    }
    let req_ptr: *mut BBTReq = unsafe { entry.as_ref().unwrap().ctx.cast() };
    let req = unsafe { req_ptr.as_ref().unwrap() };

    match req.req_type {
        BBTRequestType::Get => {
            let ctx = BBT.get_block_status(&req.pba);
            match ctx {
                BadBlockStatus::Good => println_s!(c"SENDING GOOD"),
                BadBlockStatus::Bad => println_s!(c"SENDING BAD"),
                BadBlockStatus::Reserved => println_s!(c"SENDING RESERVED"),
            }
            unsafe {
                status_res[idx_2] = ctx;
                entry.as_mut().unwrap().ctx = (status_res.get_unchecked(idx_2)
                    as *const BadBlockStatus)
                    .cast_mut()
                    .cast()
            }
            // unsafe { entry.as_mut().unwrap().ctx = (ctx as u8 + 1u8) as *mut _ };

            if unsafe { pipe_2.is_null() } {
                unsafe {
                    pipe_2 = ssd_os_get_connection(
                        c"conn_2".as_ptr().cast_mut(),
                        c"pipe_2".as_ptr().cast_mut(),
                    )
                };
            }
            ssd_os_sleep(1);
            return unsafe { pipe_2 };
        }
        BBTRequestType::Set => {
            BBT.set_bad_block(&req.pba);
            null_mut()
        }
    }
    // unsafe { entry.as_mut().unwrap().ctx = entry.as_mut().unwrap().ctx.add(1) };

    // if unsafe { pipe_2.is_null() } {
    //     unsafe {
    //         pipe_2 =
    //             ssd_os_get_connection(c"conn_2".as_ptr().cast_mut(), c"pipe_2".as_ptr().cast_mut())
    //     };
    // }
    // ssd_os_sleep(1);
    // return unsafe { pipe_2 };
}

enum BBTRequestType {
    Get,
    Set,
}

struct BBTReq {
    req_type: BBTRequestType,
    pba: PhysicalBlockAddress,
}
