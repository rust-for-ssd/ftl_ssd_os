use core::hint::black_box;

use crate::{
    bindings::{generated::ssd_os_usleep, safe::ssd_os_sleep},
    make_stage_static, println,
    requester::requester::{Request, RequestError},
};

make_stage_static!(requester_l2p_stage, init, exit, context_handler1);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}


#[unsafe(no_mangle)]
fn context_handler1(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    // ssd_os_sleep(1);
    unsafe { ssd_os_usleep(1) };
    // println!("jo");
    // let req = context as *mut Result<Request, RequestError>;
    // // println!("req_l2p_stage: {:?}", unsafe { *req });
    // unsafe {
    //     println!("req -> l2p: {}", req.as_ref().unwrap().unwrap().id);
    // }
    // black_box(context)
    context
}
