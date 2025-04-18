#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(static_mut_refs)]
#![feature(allocator_api)]

extern crate alloc;

pub mod allocator;
mod bbt;
pub mod bindings;
mod cpath;
mod provisioner;
mod shared;

#[cfg(not(feature = "test"))]
#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    bindings::safe::ssd_os_print_s(c"\nPANIC!\n");
    println!(info);
    loop {}
}
