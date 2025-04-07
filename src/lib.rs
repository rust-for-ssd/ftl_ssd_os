#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    // safe_print("PANICING\n");
    loop {}
}
use core::ffi::CStr;

use bindings::{
    MAGIC_CONNECTOR, MAGIC_STAGE, connector, lring_entry, pipeline, ssd_os_ctrl_fn,
    ssd_os_stage_fn, stage,
};

use safe_bindings::{
    safe_print, ssd_os_get_connection, ssd_os_print_lock, ssd_os_print_s, ssd_os_print_ss,
    ssd_os_print_unlock, ssd_os_sleep,
};

mod bindings;
mod safe_bindings;

static mut my_int: u64 = 0;
const hello: [u8; 32] = *b"hello world\0....................";

impl stage {
    const fn new(
        name: &[u8],
        init: ssd_os_ctrl_fn,
        exit: ssd_os_ctrl_fn,
        stage_fn: ssd_os_stage_fn,
    ) -> Self {
        stage {
            magic: *MAGIC_STAGE,
            name: {
                let mut buf = [0u8; 32];
                let mut i = 0;
                while i < name.len() {
                    buf[i] = name[i];
                    i += 1;
                }
                buf
            },
            init_fn: init,
            exit_fn: exit,
            stage_fn,
        }
    }
}

#[unsafe(no_mangle)]
pub static mut stage1: stage = stage::new(b"stage1", Some(s1_init), Some(s1_init), Some(my_stage1));
#[unsafe(no_mangle)]
pub static mut stage2: stage = stage::new(b"stage2", Some(s1_init), Some(s1_init), Some(my_stage2));
#[unsafe(no_mangle)]
pub static mut stage3: stage = stage::new(b"stage3", Some(s1_init), Some(s1_init), Some(my_stage3));
#[unsafe(no_mangle)]
pub static mut stage4: stage = stage::new(b"stage4", Some(s1_init), Some(s1_init), Some(my_stage4));
#[unsafe(no_mangle)]
pub static mut stage5: stage = stage::new(b"stage5", Some(s1_init), Some(s1_init), Some(my_stage5));
#[unsafe(no_mangle)]
pub static mut stage6: stage = stage::new(b"stage6", Some(s1_init), Some(s1_init), Some(my_stage6));

#[unsafe(no_mangle)]
pub unsafe extern "C" fn s1_init() -> ::core::ffi::c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn s1_exit() -> ::core::ffi::c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_stage1(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    // print!("{}", context as *const u8);
    // safe_print("I'M PRINTED");

    // safe_print("IM PRINTEED");
    ssd_os_print_lock();
    ssd_os_print_ss(unsafe { CStr::from_ptr(context as *const u8) }, c"1\n");
    ssd_os_print_unlock();
    context
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_stage2(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_print_lock();
    ssd_os_print_ss(unsafe { CStr::from_ptr(context as *const u8) }, c"2\n");
    ssd_os_print_unlock();
    context
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_stage3(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_print_lock();
    ssd_os_print_ss(unsafe { CStr::from_ptr(context as *const u8) }, c"3\n");
    ssd_os_print_unlock();
    context
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_stage4(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_print_lock();
    ssd_os_print_ss(unsafe { CStr::from_ptr(context as *const u8) }, c"4\n");
    ssd_os_print_unlock();
    context
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_stage5(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_print_lock();
    ssd_os_print_ss(unsafe { CStr::from_ptr(context as *const u8) }, c"5\n");
    ssd_os_print_unlock();
    context
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_stage6(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_print_lock();
    ssd_os_print_ss(unsafe { CStr::from_ptr(context as *const u8) }, c"6\n");
    ssd_os_print_unlock();
    context
}

#[unsafe(no_mangle)]
pub static mut conn1: connector = connector {
    magic: *MAGIC_CONNECTOR,
    name: {
        let mut buf = [0u8; 32];
        let s = *b"conn1";
        let mut i = 0;
        while i < s.len() {
            buf[i] = s[i];
            i += 1;
        }
        buf
    },
    init_fn: Some(conn1_init),
    exit_fn: Some(conn1_exit),
    conn_fn: Some(conn1_conn),
    ring_fn: Some(conn1_ring),
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn conn1_init() -> ::core::ffi::c_int {
    ssd_os_print_lock();
    ssd_os_print_s(c"Connector Initialized: conn1\n");
    ssd_os_print_unlock();
    unsafe {
        my_int = 0x0fffffffffffffff;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn conn1_exit() -> ::core::ffi::c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn conn1_ring(ring: *mut lring_entry) -> ::core::ffi::c_int {
    ssd_os_print_lock();
    ssd_os_print_ss(
        unsafe { CStr::from_ptr(ring.as_ref().unwrap().ctx as *const u8) },
        c"END\n",
    );
    ssd_os_print_unlock();
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn conn1_conn(entry: *mut lring_entry) -> *mut pipeline {
    ssd_os_sleep(1);
    let pipe = ssd_os_get_connection(c"conn1".as_ptr().cast_mut(), c"pipe1".as_ptr().cast_mut());
    if !pipe.is_null() {
        unsafe { entry.as_mut().unwrap() }.ctx = hello.as_ptr() as *mut ::core::ffi::c_void;
        ssd_os_print_ss(
            unsafe { CStr::from_ptr(entry.as_ref().unwrap().ctx as *const u8) },
            c"START\n",
        );
        return pipe;
    } else {
        return core::ptr::null_mut();
    }
}
