use core::{
    ffi::{CStr, c_void},
    fmt::{Error, Result, Write},
};

use super::generated;

#[cfg(not(feature = "test"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
    dest: *mut ::core::ffi::c_void,
    src: *const ::core::ffi::c_void,
    n: u32,
) -> *mut ::core::ffi::c_void {
    let alignment = ::core::mem::size_of::<usize>();
    let dest_usize = dest as usize;
    let src_usize = src as usize;

    if dest_usize % alignment != 0 || src_usize % alignment != 0 {
        // Fallback: byte-by-byte safe copy
        let dest_u8 = dest as *mut u8;
        let src_u8 = src as *const u8;
        for i in 0..n {
            unsafe { *dest_u8.add(i as usize) = *src_u8.add(i as usize) };
        }
        return dest;
    }

    // Safe to use optimized C function
    unsafe { generated::ssd_os_mem_cpy(dest, src, n) }
}

pub fn ssd_os_get_connection(connector_name: &CStr, pipe_name: &CStr) -> *mut generated::pipeline {
    unsafe {
        generated::ssd_os_get_connection(
            connector_name.as_ptr().cast_mut(),
            pipe_name.as_ptr().cast_mut(),
        )
    }
}

pub fn ssd_os_this_cpu(name: &CStr) -> ::core::ffi::c_int {
    unsafe { generated::ssd_os_this_cpu(name.as_ptr().cast_mut()) }
}

pub fn ssd_os_mem_get(key: ::core::ffi::c_int) -> *mut ::core::ffi::c_void {
    if cfg!(feature = "test") {
        return 0x80000000 as *mut c_void;
    };

    unsafe { generated::ssd_os_mem_get(key) }
}

pub fn ssd_os_print_lock() {
    unsafe {
        generated::ssd_os_print_lock();
    }
}

pub fn ssd_os_print_unlock() {
    unsafe {
        generated::ssd_os_print_unlock();
    }
}

pub fn ssd_os_sleep(i: u32) {
    unsafe {
        generated::ssd_os_sleep(i as i32);
    }
}

pub fn ssd_os_print_ss(s1: &CStr, s2: &CStr) {
    unsafe {
        generated::ssd_os_print_ss(s1.as_ptr(), s2.as_ptr());
    }
}

pub fn ssd_os_print_i(i: u32) {
    unsafe {
        generated::ssd_os_print_i(i);
    }
}

pub fn ssd_os_print_s(s: &CStr) {
    unsafe {
        generated::ssd_os_print_s(s.as_ptr());
    }
}

pub fn ssd_os_mem_size(key: i32) -> ::core::ffi::c_int {
    unsafe { generated::ssd_os_mem_size(key) }
}

pub struct SSD_OS_Printer {}
impl Write for SSD_OS_Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Write in chunks, using a fixed-size buffer with space for a null terminator
        unsafe { generated::ssd_os_print_lock() };
        const BUF_SIZE: usize = 32;
        static mut buffer: [u8; BUF_SIZE] = [0u8; BUF_SIZE];

        let mut remaining = s.as_bytes();

        while !remaining.is_empty() {
            let len = core::cmp::min(BUF_SIZE - 1, remaining.len()); // Leave space for null terminator
            let (chunk, rest) = remaining.split_at(len);

            // Copy chunk into buffer and null-terminate
            unsafe {
                buffer[..len].copy_from_slice(chunk);
                buffer[len] = 0; // null terminator
            }

            // SAFETY: We ensure buffer is null-terminated and has no internal nulls
            unsafe { generated::ssd_os_print_s(buffer.as_ptr()) };

            remaining = rest;
        }
        unsafe { generated::ssd_os_print_unlock() };
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($msg:expr) => {{
        use core::fmt::Write;
        let mut printer = $crate::bindings::safe::SSD_OS_Printer {};
        let _ = writeln!(printer, "{}", $msg);
    }};
}
