use crate::{
    bindings::safe::ssd_os_sleep,
    make_stage_static, println,
    requester::requester::{Request, RequestError},
};

make_stage_static!(media_manager_requester_stage, init, exit, context_handler);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_sleep(1);
    let req = context as *mut Result<Request, RequestError>;
    // println!("mm_req_stage req: {:?}", unsafe { *req });
    unsafe {
        println!("mm -> req: {}", req.as_ref().unwrap().unwrap().id);
    }
    // i
    // println!("mm_req_stage data: {:?}", unsafe {
    //     *((*req).unwrap().data)
    // });
    context
}
