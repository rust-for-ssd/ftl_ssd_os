// -- Imports and setup ---
#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(rv_unit::test_runner)]
#![feature(allocator_api)]
#![feature(box_as_ptr)]

use ftl_ssd_os;
use riscv_rt::entry;
use semihosting::println;

// -- Custom panic handler
#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    rv_unit::test_panic_handler(info);
    test_main();
    loop {}
}

unsafe extern "C" {
    static _heap_size: u8;

}

#[entry]
fn main() -> ! {
    test_main();
    loop {}
}

mod unit;
