use core::{
    ffi::CStr,
    fmt::{Error, Result, Write},
};

use super::generated;

#[cfg(not(feature = "qemu_testing"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
    dest: *mut ::core::ffi::c_void,
    src: *const ::core::ffi::c_void,
    n: u32,
) -> *mut ::core::ffi::c_void {
    // We'll assume that the "word-sized" copy in ssd_os_mem_cpy requires the pointers to be aligned to usize.
    let alignment = ::core::mem::size_of::<usize>();

    assert!(dest as usize % alignment == 0);
    assert!(src as usize % alignment == 0);

    // Now that we know the pointers are properly aligned, call the underlying implementation.
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

pub fn safe_print(s: &str) {
    let Ok(()) = ssd_os_printer.write_str(s) else {
        ssd_os_print_lock();
        ssd_os_print_s(c"Err\n");
        ssd_os_print_unlock();
        return ();
    };
    let _ = ssd_os_printer.write_fmt(format_args!("{}", s));
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
macro_rules! println_s {
    ($msg:expr) => {{
        $crate::bindings::safe::ssd_os_print_lock();
        $crate::bindings::safe::ssd_os_print_s($msg);
        $crate::bindings::safe::ssd_os_print_s(c"\n");
        $crate::bindings::safe::ssd_os_print_unlock();
    }};
}

#[macro_export]
macro_rules! println_i {
    ($msg:expr) => {{
        $crate::bindings::safe::ssd_os_print_lock();
        $crate::bindings::safe::ssd_os_print_i($msg);
        $crate::bindings::safe::ssd_os_print_s(c"\n");
        $crate::bindings::safe::ssd_os_print_unlock();
    }};
}
