#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(static_mut_refs)]
mod bbt;
mod bindings;
mod my_alloc;
mod safe_bindings;
mod shared;
mod ssd_os;

extern crate alloc;

use crate::bbt::bbt::BadBlockTable;
use ::core::ffi::CStr;
// use bbt::BadBlockTable;
use bindings::{
    MAGIC_CONNECTOR, MAGIC_STAGE, connector, lring_entry, pipeline, ssd_os_ctrl_fn,
    ssd_os_stage_fn, stage,
};

#[inline(never)]
fn panic_printer(info: &core::panic::PanicInfo) {
    const BUFFER_SIZE: usize = 128;
    static mut buffer: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];

    if let Some(localtion) = info.location() {
        let file = localtion.file();
        let line = localtion.line();
        let col = localtion.column();
        for i in 0..(BUFFER_SIZE - 1).min(file.len()) {
            unsafe {
                buffer[i + 1] = 0;
                buffer[i] = file.as_bytes()[i];
            }
        }
        unsafe {
            println_s!(CStr::from_bytes_with_nul_unchecked(&buffer));
        }
        println_s!(c"line: ");
        println_i!(line);
        println_s!(c"column: ");
        println_i!(col);
    }
    if let Some(msg) = info.message().as_str() {
        for i in 0..(BUFFER_SIZE - 1).min(msg.len()) {
            unsafe {
                buffer[i + 1] = 0;
                buffer[i] = msg.as_bytes()[i];
            }
        }
        unsafe {
            println_s!(CStr::from_bytes_with_nul_unchecked(&buffer));
        }
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println_s!(c"PANIC!");
    panic_printer(info);
    loop {}
}

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
