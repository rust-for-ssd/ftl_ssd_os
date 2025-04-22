use crate::{make_stage_static, println};

use crate::requester::requester::{CommandType,RequestError};

make_stage_static!(read_l2p, init_l2p, exit, l2p_read_context_handler);
make_stage_static!(read_mm, init_mm, exit, mm_context_handler);


fn init_l2p() -> ::core::ffi::c_int {
    println!("1234!");
    0
}

fn init_mm() -> ::core::ffi::c_int {
    println!("1234!");
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn l2p_read_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("READ: L2P STAGE");
    // We just propagete the context here.
    context
}

fn mm_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("READ: MM STAGE");
    // let req = context as *mut Result<Request, RequestError>;
    
    // println!("REQUESTER TO L2P STAGE: {:?}", unsafe {*req});
    
    // We just propagete the context here.
    context
}