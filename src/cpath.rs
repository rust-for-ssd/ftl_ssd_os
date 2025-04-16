use core::ptr::null_mut;

use crate::{
    bindings::{
        generated::ssd_os::{
            lring, lring_entry, pipeline, ssd_os_lring_create, ssd_os_lring_dequeue,
            ssd_os_lring_enqueue,
        },
        safe::{ssd_os_get_connection, ssd_os_mem_get, ssd_os_sleep, ssd_os_this_cpu},
    },
    make_connector_static, make_stage_static, println_i, println_s,
};

// static mut i: i32 = 0;

make_stage_static!(stage_1, s1, s1, stage_1_fn);
make_stage_static!(stage_2, s1, s1, stage_2_fn);
fn s1() -> ::core::ffi::c_int {
    0
}
fn stage_1_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println_s!(c"STAGE_1");
    println_i!(context as u32);
    unsafe { context.add(1) }
}

fn stage_2_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println_s!(c"STAGE_2");
    println_i!(context as u32);
    unsafe { context.add(1) }
}
make_connector_static!(conn_1, init_1, exit_1, conn_fn_1, ring_1);
make_connector_static!(conn_2, init_2, exit_2, conn_fn_2, ring_2);
fn init_1() -> ::core::ffi::c_int {
    println_s!(c"INIT_1");
    0
}
fn init_2() -> ::core::ffi::c_int {
    println_s!(c"INIT_2");
    let mem = ssd_os_mem_get(0);

    unsafe { conn2_lring = ssd_os_lring_create(c"CONN2_LRING".as_ptr().cast_mut(), 128, mem, 0x0) };

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

fn conn_fn_1(entry: *mut lring_entry) -> *mut pipeline {
    println_s!(c"CON_FN_1");
    unsafe { entry.as_mut().unwrap().ctx = 1 as *mut _ };
    if unsafe { pipe_1.is_null() } {
        unsafe {
            pipe_1 =
                ssd_os_get_connection(c"conn_1".as_ptr().cast_mut(), c"pipe_1".as_ptr().cast_mut())
        };
    }
    ssd_os_sleep(1);
    return unsafe { pipe_1 };
}

fn conn_fn_2(entry: *mut lring_entry) -> *mut pipeline {
    println_s!(c"CON_FN_2");
    unsafe { ssd_os_lring_dequeue(conn2_lring, entry) };
    unsafe { entry.as_mut().unwrap().ctx = entry.as_mut().unwrap().ctx.add(1) };
    if unsafe { pipe_2.is_null() } {
        unsafe {
            pipe_2 =
                ssd_os_get_connection(c"conn_2".as_ptr().cast_mut(), c"pipe_2".as_ptr().cast_mut())
        };
    }
    ssd_os_sleep(1);
    return unsafe { pipe_2 };
}
