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

pub(crate) mod media_manager;
pub(crate) mod allocator;
pub mod apps;
pub(crate) mod bbt;
pub(crate) mod bindings;
pub(crate) mod l2p;
pub(crate) mod provisioner;
pub(crate) mod requester;
pub(crate) mod shared;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    bindings::safe::ssd_os_print_s(c"\nPANIC!\n");
    println!(info);
    loop {}
}
