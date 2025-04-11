use crate::bindings::{connector, lring_entry, pipeline, stage};
use crate::safe_bindings::{
    ssd_os_get_connection, ssd_os_print_lock, ssd_os_print_ss, ssd_os_print_unlock, ssd_os_sleep,
};
use ::core::ffi::CStr;

use crate::ssd_os::lring::LRing;
use crate::{make_connector, make_connector_static, make_stage, make_stage_static, println_s};

const hello: [u8; 32] = *b"hello world\0....................";

make_stage_static!(gc_sstage, s1_init, s1_init, gc_sstage_fn);
make_stage_static!(gc_cstage, s1_init, s1_init, gc_cstage_fn);

fn gc_sstage_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_print_lock();
    ssd_os_print_ss(
        unsafe { CStr::from_ptr(context as *const u8) },
        c"gc_sstage\n",
    );
    ssd_os_print_unlock();
    context
}

fn gc_cstage_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_print_lock();
    ssd_os_print_ss(
        unsafe { CStr::from_ptr(context as *const u8) },
        c"gc_cstage\n",
    );
    ssd_os_print_unlock();
    context
}

fn s1_init() -> ::core::ffi::c_int {
    0
}

fn s1_exit() -> ::core::ffi::c_int {
    0
}

static gc_lring: LRing<128> = LRing::new();

make_connector_static!(gc_conn, gc_init, gc_exit, gc_conn_fn, gc_ring);

fn gc_init() -> ::core::ffi::c_int {
    println_s!(c"init start:");
    // let cpu_id = ssd_os_this_cpu(gc_conn.get_name());
    // let memory_region = ssd_os_mem_get(cpu_id) as usize;
    // let memory_size = ssd_os_mem_size(cpu_id) as usize;

    gc_lring.init(c"gc_lring", 0);

    0
}

fn gc_exit() -> ::core::ffi::c_int {
    0
}

fn gc_ring(ring: *mut lring_entry) -> ::core::ffi::c_int {
    ssd_os_print_lock();
    ssd_os_print_ss(
        unsafe { CStr::from_ptr(ring.as_ref().unwrap().ctx as *const u8) },
        c"END\n",
    );
    ssd_os_print_unlock();
    0
}

fn gc_conn_fn(entry: *mut lring_entry) -> *mut pipeline {
    let pipe = ssd_os_get_connection(
        c"gc_conn".as_ptr().cast_mut(),
        c"gc_pipe".as_ptr().cast_mut(),
    );
    if !pipe.is_null() {
        unsafe { entry.as_mut().unwrap() }.ctx = hello.as_ptr() as *mut ::core::ffi::c_void;
        ssd_os_print_ss(
            unsafe { CStr::from_ptr(entry.as_ref().unwrap().ctx as *const u8) },
            c"START\n",
        );
        ssd_os_sleep(1);
        return pipe;
    } else {
        return ::core::ptr::null_mut();
    }
}
