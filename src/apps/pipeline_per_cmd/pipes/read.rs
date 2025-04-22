use crate::bindings::safe::ssd_os_this_cpu;
use crate::shared::semaphore::Semaphore;
use crate::{make_stage_static, println};

use crate::requester::requester::{CommandType, RequestError};

make_stage_static!(read_l2p, init_l2p, exit, l2p_read_context_handler);
make_stage_static!(read_mm, init_mm, exit, mm_context_handler);

fn init_l2p() -> ::core::ffi::c_int {
    println!("INIT FUNCTION!");
    let cpu = ssd_os_this_cpu(c"read_l2p");
    println!("{}", cpu);
    0
}

fn init_mm() -> ::core::ffi::c_int {
    println!("INIT FUNCTION!222!");
    let cpu = ssd_os_this_cpu(c"read_l2p");
    println!("{}", cpu);
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

static x: Semaphore<i32> = Semaphore::new(0);

fn l2p_read_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("READ: L2P STAGE");
    x.with_lock(|y| {
        println!("A: {}", y);
        *y += 1;
    });
    context
}

fn mm_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("READ: MM STAGE");
    let mut y = x.lock();
    println!("B: {}", *y);
    *y += 1;
    context
}
