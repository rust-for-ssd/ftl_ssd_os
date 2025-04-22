#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(static_mut_refs)]
#![feature(allocator_api)]
#![feature(btreemap_alloc)]

extern crate alloc;

pub mod allocator;
pub mod bbt;
mod bindings;
pub mod provisioner;
pub mod shared;
pub mod l2p;
pub mod apps;
pub mod media_manager;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    bindings::safe::ssd_os_print_s(c"\nPANIC!\n");
    println!(info);
    loop {}
}
