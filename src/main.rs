#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(static_mut_refs)]
#![feature(allocator_api)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(rv_unit::test_runner)]
#![feature(box_as_ptr)]
#![feature(btreemap_alloc)]

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(test)]
    {
        use rv_unit::test_panic_handler;

        test_panic_handler(info);
        test_main();
    }
    loop {}
}

#[cfg(test)]
unsafe extern "C" {
    static _heap_size: u8;

}

#[cfg(test)]
#[riscv_rt::entry]
fn main() -> ! {
    #[cfg(test)]
    test_main();
    loop {}
}

extern crate alloc;

pub(crate) mod allocator;
pub(crate) mod bbt;
pub(crate) mod bindings;
pub(crate) mod l2p;
pub(crate) mod media_manager;
pub(crate) mod provisioner;
pub(crate) mod requester;
pub(crate) mod shared;

#[cfg(test)]
mod tests;
