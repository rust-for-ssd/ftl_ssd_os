#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(static_mut_refs)]
#![feature(allocator_api)]

extern crate alloc;

mod allocator;
mod bbt;
mod bindings;
mod shared;
mod test;

use ::core::ffi::CStr;

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
