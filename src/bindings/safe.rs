use core::{
    ffi::{CStr, c_void},
    fmt::Write,
};

use super::generated;


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
    #[cfg(not(feature = "test"))]
    unsafe {
        generated::ssd_os_print_lock();
    }
}

pub fn ssd_os_print_unlock() {
    #[cfg(not(feature = "test"))]
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
    #[cfg(feature = "test")]
    {
        semihosting::print!("{}", s.to_str().unwrap());
    }
    #[cfg(not(feature = "test"))]
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
        ssd_os_print_lock();
        const BUF_SIZE: usize = 256;
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
            ssd_os_print_s(unsafe { CStr::from_ptr(buffer.as_ptr()) });

            remaining = rest;
        }
        ssd_os_print_unlock();
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    // Case where there's only one argument and it is a literal format string
    ($arg:expr) => {{
        use core::fmt::Write;
        let mut printer = $crate::bindings::safe::SSD_OS_Printer {};
        let _ = writeln!(printer, "{}", $arg);
    }};
    // Case where there are multiple arguments, including format string and parameters
    ($fmt:expr, $($args:tt)+) => {{
        use core::fmt::Write;
        let mut printer = $crate::bindings::safe::SSD_OS_Printer {};
        let _ = writeln!(printer, $fmt, $($args)+);
    }};
}