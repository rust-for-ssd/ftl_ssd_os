use core::{hint::black_box};

use crate::{
    bindings::safe::ssd_os_sleep,
    make_stage_static, println,
    requester::requester::{Request, RequestError}, shared::macros::ensure_unique,
};

make_stage_static!(l2p_media_manager_stage, init, exit, context_handler_mm);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_mm(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ensure_unique!();
    // ssd_os_sleep(1);
    // let req = context as *mut Result<Request, RequestError>;
    // unsafe {
    //     println!("l2p -> mm: {}", req.as_ref().unwrap().unwrap().id);
    // }
    context
}
