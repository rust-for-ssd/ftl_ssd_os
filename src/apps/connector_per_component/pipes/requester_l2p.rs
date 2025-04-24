use crate::{
    bindings::safe::ssd_os_sleep,
    make_stage_static, println,
    requester::requester::{Request, RequestError},
};

make_stage_static!(requester_l2p_stage, init, exit, context_handler);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_sleep(1);
    let req = context as *mut Result<Request, RequestError>;
    // println!("req_l2p_stage: {:?}", unsafe { *req });
    unsafe {
        println!("req -> l2p: {}", req.as_ref().unwrap().unwrap().id);
    }
    context
}
