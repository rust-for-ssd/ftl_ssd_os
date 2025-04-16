use crate::allocator::sdd_os_alloc::SimpleAllocator;
use crate::bindings::generated::ssd_os::lring;
use crate::bindings::generated::ssd_os::lring_entry;
use crate::bindings::generated::ssd_os::pipeline;
use crate::bindings::generated::ssd_os::ssd_os_lring_create;
use crate::bindings::generated::ssd_os::ssd_os_lring_dequeue;
use crate::bindings::generated::ssd_os::ssd_os_lring_enqueue;
use crate::bindings::generated::volt::nvm_mmgr_geometry;
use crate::bindings::generated::volt::volt_get_geometry;
use crate::bindings::lring::LRing;
use crate::bindings::safe::{
    ssd_os_get_connection, ssd_os_mem_get, ssd_os_mem_size, ssd_os_print_lock, ssd_os_print_ss,
    ssd_os_print_unlock, ssd_os_sleep, ssd_os_this_cpu,
};
use crate::{make_connector_static, make_stage_static, shared};
use ::core::ffi::CStr;
use alloc::boxed::Box;
use core::mem::MaybeUninit;
use core::ptr::null_mut;
use shared::addresses::PhysicalBlockAddress;

use crate::{println_i, println_s};

use super::bbt::BadBlockTable;

static BBT_ALLOCATOR: SimpleAllocator = SimpleAllocator::new();

fn s1_init() -> ::core::ffi::c_int {
    0
}

fn s1_exit() -> ::core::ffi::c_int {
    0
}

make_connector_static!(bbt_conn, bbt_init, bbt_exit, bbt_conn_fn, bbt_ring);
make_stage_static!(bbt_cstage, s1_init, s1_init, bbt_cstage_fn);

fn bbt_cstage_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println_s!(c"BBT COMPLETION STAGE");
    context
}

static BBT: BadBlockTable<SimpleAllocator> = BadBlockTable::new();

fn bbt_init() -> ::core::ffi::c_int {
    ssd_os_sleep(1);
    println_s!(c"BBT CONNECTOR: INIT START");
    let cpu_id = ssd_os_this_cpu(bbt_conn.get_name());
    let memory_region = ssd_os_mem_get(cpu_id);

    unsafe { ssd_os_lring_create(c"bbt_lring".as_ptr().cast_mut(), 128, memory_region, 0) };

    ssd_os_sleep(1);

    println_s!(c"BBT CONNECTOR: INIT END");
    0
}

fn bbt_exit() -> ::core::ffi::c_int {
    0
}

fn bbt_ring(ring: *mut lring_entry) -> ::core::ffi::c_int {
    ssd_os_sleep(4);
    // unsafe { ssd_os_lring_enqueue(bbt_lring, ring) };
    0
}

static mut bbt_lring: *mut lring = null_mut();

fn bbt_conn_fn(entry: *mut lring_entry) -> *mut pipeline {
    ssd_os_sleep(4);
    let res = unsafe { ssd_os_lring_dequeue(bbt_lring, entry) };
    println_i!(res as u32);

    // if entry.is_null() {
    //     println_s!(c"HERE");

    //     return ::core::ptr::null_mut();
    // }

    println_s!(c"I'm NOT CRSHED");

    // let pipe = ssd_os_get_connection(
    //     c"bbt_conn".as_ptr().cast_mut(),
    //     c"bbt_cpipe".as_ptr().cast_mut(),
    // );
    // if !pipe.is_null() {
    //     println_s!(c"CREATED COMPLETION PIPE");

    //     return pipe;
    // }

    return ::core::ptr::null_mut();
}
