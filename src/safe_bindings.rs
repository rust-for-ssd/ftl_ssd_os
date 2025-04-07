use core::{
    ffi::CStr,
    fmt::{Error, Result, Write},
};

use crate::bindings::{self, pipeline};

pub fn ssd_os_get_connection(
    connector_name: *mut ::core::ffi::c_char,
    pipe_name: *mut ::core::ffi::c_char,
) -> *mut pipeline {
    unsafe { bindings::ssd_os_get_connection(connector_name, pipe_name) }
}

pub fn ssd_os_print_lock() {
    unsafe {
        bindings::ssd_os_print_lock();
    }
}

pub fn ssd_os_print_unlock() {
    unsafe {
        bindings::ssd_os_print_unlock();
    }
}

pub fn ssd_os_sleep(i: u32) {
    unsafe {
        bindings::ssd_os_sleep(i as i32);
    }
}

pub fn ssd_os_print_ss(s1: &CStr, s2: &CStr) {
    unsafe {
        bindings::ssd_os_print_ss(s1.as_ptr(), s2.as_ptr());
    }
}

pub fn ssd_os_print_i(i: u32) {
    unsafe {
        bindings::ssd_os_print_i(i);
    }
}

pub fn ssd_os_print_s(s: &CStr) {
    unsafe {
        bindings::ssd_os_print_s(s.as_ptr());
    }
}

pub fn safe_print(s: &str) {
    let Ok(()) = ssd_os_printer.write_str(s) else {
        ssd_os_print_lock();
        ssd_os_print_s(c"Err\n");
        ssd_os_print_unlock();
        return ();
    };
    // let _ = ssd_os_printer.write_fmt(format_args!("{}", s));
}

pub struct ssd_os_printer;

impl Write for ssd_os_printer {
    fn write_str(&mut self, s: &str) -> Result {
        let mut buffer: [u8; 32] = [0u8; 32];

        let mut i = 0;
        for c in s.bytes() {
            if i == 31 {
                let Ok(cstr) = CStr::from_bytes_with_nul(&buffer) else {
                    return Err(Error);
                };
                ssd_os_print_lock();
                ssd_os_print_s(cstr);
                ssd_os_print_unlock();
                i = 0;
            }
            buffer[i] = c;
            buffer[i + 1] = 0;
            i += 1;
        }
        if s.len() % 32 != 0 {
            let Ok(cstr) = CStr::from_bytes_with_nul(&buffer) else {
                return Err(Error);
            };
            ssd_os_print_lock();
            ssd_os_print_s(cstr);
            ssd_os_print_unlock();
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($msg:expr) => {{
        $crate::ssd_os_print_lock();
        $crate::ssd_os_print_s($msg);
        $crate::ssd_os_print_unlock();
    }};
}

#[macro_export]
macro_rules! println_ {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _= $crate::safe_bindings::ssd_os_printer.write_fmt(format_args!("{}\n", format_args!($($arg)*)));
    }};
}

#[macro_export]
macro_rules! print {
    // Match case with just a string literal
    ($msg:expr) => {{
        $crate::ssd_os_print_lock();
        $crate::ssd_os_print_s($msg);
        $crate::ssd_os_print_unlock();
    }};

    // Match case with a string literal and a single argument
    ($msg:expr, $arg:expr) => {{
        $crate::ssd_os_print_lock();
        $crate::ssd_os_print_ss($msg.as_ptr(), $arg);
        $crate::ssd_os_print_unlock();
    }};
}
