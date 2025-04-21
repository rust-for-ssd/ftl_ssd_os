use crate::{make_stage_static, println};

make_stage_static!(requester_l2p_stage, init, exit, context_handler);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("requester_l2p_STAGE");
    println!(context as u32);
    context
}