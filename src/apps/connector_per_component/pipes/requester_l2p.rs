use core::hint::black_box;

use crate::{
    bindings::{generated::ssd_os_usleep, safe::ssd_os_sleep},
    make_stage_static, println,
    requester::requester::{Request, RequestError}, shared::macros::ensure_unique,
};



make_stage_static!(requester_l2p_stage, init, exit, context_handler_rr_l2p);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_rr_l2p(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ensure_unique!();
    // let req = context as *mut Result<Request, RequestError>;
    // // println!("req_l2p_stage: {:?}", unsafe { *req });
    // unsafe {
    //     println!("req -> l2p: {}", req.as_ref().unwrap().unwrap().id);
    // }
    context
}