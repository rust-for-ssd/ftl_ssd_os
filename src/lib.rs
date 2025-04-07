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
use core::ffi::{CStr, c_void};

use bindings::{
    MAGIC_CONNECTOR, MAGIC_STAGE, connector, lring_entry, pipeline, ssd_os_ctrl_fn,
    ssd_os_stage_fn, stage,
};

use safe_bindings::{
    ssd_os_get_connection, ssd_os_mem_cpy, ssd_os_mem_get, ssd_os_print_i, ssd_os_print_lock, ssd_os_print_s, ssd_os_print_ss, ssd_os_print_unlock, ssd_os_sleep, ssd_os_this_cpu
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
pub static mut bbt_stage: stage = stage::new(
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
pub static mut bbt_conn: connector = connector {
    magic: *MAGIC_CONNECTOR,
    name: {
        let mut buf = [0u8; 32];
        let s = *b"bbt_conn";
        let mut i = 0;
        while i < s.len() {
            buf[i] = s[i];
            i += 1;
        }
        buf
    },
    init_fn: Some(bbt_init),
    exit_fn: Some(bbt_exit),
    conn_fn: Some(bbt_conn_fn),
    ring_fn: Some(bbt_ring),
};

impl connector {
    fn get_name(&self) -> &CStr {
        let Ok(s) = CStr::from_bytes_until_nul(&self.name) else {
            ssd_os_print_lock();
            ssd_os_print_s(c"here: ");
            ssd_os_print_unlock();
            return c"";
        };
        s
    }
}

#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_init() -> ::core::ffi::c_int {
    let cpu_id = ssd_os_this_cpu(unsafe { bbt_conn.get_name() });
    let memory_region = ssd_os_mem_get(cpu_id);
    let test = 90;
    let s = b"MEMORY REGION STR!\0";
    unsafe {
        // core::ptr::copy(s, memory_region as *mut [u8; 19], s.len());
        // core::ptr::write_volatile(memory_region as *mut u8, b"MEMORY REGION STR! \0"); // Writes 90 to the address [2][5]
    }

    ssd_os_print_lock();
    ssd_os_print_s(CStr::from_ptr(memory_region as *mut u8));
    ssd_os_print_s(c"bbt memory region: ");
    ssd_os_print_i(memory_region as u32);
    ssd_os_print_s(c"\nbbt cpu_id: ");
    ssd_os_print_i(cpu_id as u32);
    ssd_os_print_s(c"\nhelloo\n ");
    ssd_os_print_i(42);

    ssd_os_print_unlock();
    
    

    ssd_os_print_lock();
    ssd_os_mem_cpy(memory_region, s.as_ptr() as *const c_void, 19);
    ssd_os_print_s(c"Printing from mem region\n");
    ssd_os_print_s(unsafe { CStr::from_ptr(memory_region as *const u8) });

    ssd_os_print_unlock();
    unsafe {
        my_int = 0x0fffffffffffffff;
    }
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
    ssd_os_sleep(1);
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
        return core::ptr::null_mut();
    }
}
