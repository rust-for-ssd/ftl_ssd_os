#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#[allow(static_mut_refs)]
mod bindings;
mod my_alloc;
mod safe_bindings;
extern crate alloc;

use ::core::ffi::CStr;
use alloc::boxed::Box;
use bindings::{
    MAGIC_CONNECTOR, MAGIC_STAGE, connector, lring_entry, pipeline, ssd_os_ctrl_fn,
    ssd_os_stage_fn, stage,
};
use my_alloc::SimpleAllocator;
use safe_bindings::{
    ssd_os_get_connection, ssd_os_mem_get, ssd_os_mem_size, ssd_os_print_lock, ssd_os_print_s,
    ssd_os_print_ss, ssd_os_print_unlock, ssd_os_sleep, ssd_os_this_cpu,
};

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    ssd_os_print_lock();
    ssd_os_print_s(c"PANICING\n");
    ssd_os_print_unlock();

    loop {}
}

#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator::new();


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
pub static bbt_stage: stage = stage::new(
    b"bbt_stage",
    Some(s1_init),
    Some(s1_init),
    Some(bbt_stage_fn),
);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn s1_init() -> ::core::ffi::c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn s1_exit() -> ::core::ffi::c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_stage_fn(
    context: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    ssd_os_print_lock();
    ssd_os_print_ss(
        unsafe { CStr::from_ptr(context as *const u8) },
        c"BBT_STAGE\n",
    );
    ssd_os_print_unlock();
    context
}

#[unsafe(no_mangle)]
pub static bbt_conn: connector =
    connector::new(c"bbt_conn", bbt_init, bbt_exit, bbt_conn_fn, bbt_ring);

impl connector {
    const fn new(
        name: &CStr,
        init_fn: unsafe extern "C" fn() -> i32,
        exit_fn: unsafe extern "C" fn() -> i32,
        conn_fn: unsafe extern "C" fn(*mut lring_entry) -> *mut pipeline,
        ring_fn: unsafe extern "C" fn(*mut lring_entry) -> i32,
    ) -> Self {
        Self {
            magic: *MAGIC_CONNECTOR,
            name: {
                let mut buf = [0u8; 32];
                let s = name.to_bytes();
                let mut i = 0;
                while i < s.len() {
                    buf[i] = s[i];
                    i += 1;
                }
                buf
            },
            init_fn: Some(init_fn),
            exit_fn: Some(exit_fn),
            conn_fn: Some(conn_fn),
            ring_fn: Some(ring_fn),
        }
    }
    fn get_name(&self) -> &CStr {
        let Ok(s) = CStr::from_bytes_until_nul(&self.name) else {
            println_s!(c"ERROR!");
            return c"";
        };
        s
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_init() -> ::core::ffi::c_int {
    println_s!(c"init start:");
    let cpu_id = ssd_os_this_cpu(bbt_conn.get_name());
    let memory_region = ssd_os_mem_get(cpu_id) as usize;
    let memory_size = ssd_os_mem_size(cpu_id) as usize;

    assert_eq!(
        (&ALLOCATOR as *const _ as usize) % core::mem::align_of::<usize>(),
        0
    );
    ALLOCATOR.initialize(memory_region, memory_region + memory_size);


    let mut heap_val1: alloc::vec::Vec<u32> = alloc::vec::Vec::with_capacity(3);

    heap_val1.push(42);
    println_i!(heap_val1[0]);
    heap_val1[0] = 69;
    println_i!(heap_val1[0]);
    heap_val1.push(3);
    // heap_val1.push(3);
    // heap_val1.push(3);
    // println_i!(heap_val1[3]);

    let b1 = Box::new(41u32);
    let b2 = Box::new(42u32);
    let b3 = Box::new(43u32);
    println_i!(*b1);
    println_i!(*b2);
    println_i!(*b3);

    ssd_os_sleep(10);

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_exit() -> ::core::ffi::c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_ring(ring: *mut lring_entry) -> ::core::ffi::c_int {
    ssd_os_print_lock();
    ssd_os_print_ss(
        unsafe { CStr::from_ptr(ring.as_ref().unwrap().ctx as *const u8) },
        c"END\n",
    );
    ssd_os_print_unlock();
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_conn_fn(entry: *mut lring_entry) -> *mut pipeline {
    let pipe = ssd_os_get_connection(
        c"bbt_conn".as_ptr().cast_mut(),
        c"bbt_pipe".as_ptr().cast_mut(),
    );
    if !pipe.is_null() {
        unsafe { entry.as_mut().unwrap() }.ctx = hello.as_ptr() as *mut ::core::ffi::c_void;
        ssd_os_print_ss(
            unsafe { CStr::from_ptr(entry.as_ref().unwrap().ctx as *const u8) },
            c"START\n",
        );
        return pipe;
    } else {
        return ::core::ptr::null_mut();
    }
}
