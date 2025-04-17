use crate::bindings::{lring_entry, pipeline};
use crate::safe_bindings::{
    ssd_os_get_connection, ssd_os_print_lock, ssd_os_print_ss, ssd_os_print_unlock, ssd_os_sleep,
};
use ::core::ffi::CStr;

use crate::ssd_os::lring::LRing;
use crate::{make_connector_static, make_stage_static, println_s};

make_stage_static!(gc_sstage, s1_init, s1_init, gc_sstage_fn);

fn gc_sstage_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println_s!(c"GC SUBMISSION STAGE");
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
    println_s!(c"GC INIT: START");
    println_s!(c"GC INIT: RING");
    gc_lring.init(c"gc_lring", 0);
    println_s!(c"GC INIT: END");
    0
}

fn gc_exit() -> ::core::ffi::c_int {
    0
}

fn gc_ring(ring: *mut lring_entry) -> ::core::ffi::c_int {
    println_s!(c"GC RING");
    0
}

fn gc_conn_fn(entry: *mut lring_entry) -> *mut pipeline {
    ssd_os_sleep(1);
    println_s!(c"GC CONN FN: START");
    let pipe = ssd_os_get_connection(
        c"gc_conn".as_ptr().cast_mut(),
        c"gc_spipe".as_ptr().cast_mut(),
    );
    if !pipe.is_null() {
        unsafe { entry.as_mut().unwrap().ctx = 42 as *mut _ };
        return pipe;
    } else {
        return ::core::ptr::null_mut();
    }
}



// GC INIT -> GC CONN -> GC_PIPE (GC STAGE1 -> ...) -> BBT RING -> 