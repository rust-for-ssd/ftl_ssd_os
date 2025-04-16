use core::ptr::null_mut;

use crate::{
    bindings::generated::ssd_os::{
        lring, lring_entry, pipeline, ssd_os_lring_create, ssd_os_lring_dequeue,
        ssd_os_lring_enqueue,
    },
    bindings::safe::{ssd_os_get_connection, ssd_os_mem_get, ssd_os_sleep, ssd_os_this_cpu},
    make_connector_static, make_stage_static, println_s,
};

// static mut i: i32 = 0;

make_stage_static!(stage_1, s1, s1, stage_fn);
make_stage_static!(stage_2, s1, s1, stage_fn);
fn s1() -> ::core::ffi::c_int {
    0
}
fn stage_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println_s!(c"STAGE");
    context
}

make_connector_static!(conn_1, init_1, exit_1, conn_fn_1, ring_1);
make_connector_static!(conn_2, init_2, exit_2, conn_fn_2, ring_2);
fn init_1() -> ::core::ffi::c_int {
    ssd_os_sleep(1);
    println_s!(c"INIT_1");
    0
}
static mut conn2_lring: *mut lring = null_mut();
fn init_2() -> ::core::ffi::c_int {
    ssd_os_sleep(1);
    println_s!(c"INIT_2");
    let cpu_id = ssd_os_this_cpu(c"conn_2");
    let mem = ssd_os_mem_get(cpu_id);
    unsafe {
        // conn2_lring = ssd_os_lring_create(c"r1".as_ptr().cast_mut(), 128, mem, 0);
    }
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

fn ring_1(ring: *mut lring_entry) -> ::core::ffi::c_int {
    println_s!(c"RING_1");
    ssd_os_sleep(1);
    0
}

fn ring_2(ring: *mut lring_entry) -> ::core::ffi::c_int {
    println_s!(c"RING_2");
    // unsafe { ssd_os_lring_enqueue(conn2_lring, ring) };
    // unsafe { ring.as_mut().unwrap().ctx = 69 as *mut _ };
    ssd_os_sleep(1);
    0
}

static mut pipe_1: *mut pipeline = null_mut();
static mut pipe_2: *mut pipeline = null_mut();

fn conn_fn_1(entry: *mut lring_entry) -> *mut pipeline {
    println_s!(c"CON_FN_1");
    unsafe { entry.as_mut().unwrap().ctx = 69 as *mut _ };
    if unsafe { pipe_1.is_null() } {
        unsafe {
            pipe_1 =
                ssd_os_get_connection(c"conn_1".as_ptr().cast_mut(), c"pipe_1".as_ptr().cast_mut())
        };
    }
    ssd_os_sleep(1);
    return null_mut();
}

fn conn_fn_2(entry: *mut lring_entry) -> *mut pipeline {
    println_s!(c"CON_FN_2");
    // unsafe { ssd_os_lring_dequeue(conn2_lring, entry) };
    if unsafe { pipe_2.is_null() } {
        unsafe {
            pipe_2 =
                ssd_os_get_connection(c"conn_2".as_ptr().cast_mut(), c"pipe_2".as_ptr().cast_mut())
        };
    }
    ssd_os_sleep(1);
    return null_mut();
}
