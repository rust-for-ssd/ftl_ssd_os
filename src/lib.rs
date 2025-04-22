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

pub mod requester;
pub mod media_manager;
pub mod allocator;
pub mod bbt;
pub mod bindings;
pub mod provisioner;
pub mod shared;
pub mod l2p;
pub mod apps;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    bindings::safe::ssd_os_print_s(c"\nPANIC!\n");
    println!(info);
    loop {}
}
