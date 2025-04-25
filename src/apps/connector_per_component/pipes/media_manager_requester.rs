use core::hint::black_box;

use crate::{
    bindings::{generated::{ssd_os_msleep, ssd_os_usleep}, safe::ssd_os_sleep},
    make_stage_static, println,
    requester::requester::{Request, RequestError},
};

make_stage_static!(media_manager_requester_stage, init, exit, context_handler867);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}


#[unsafe(no_mangle)]
fn context_handler867(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    // ssd_os_sleep(1);
    // unsafe { ssd_os_msleep(1) };
    // unsafe {ssd_os_usleep(1);}
    // let req = context as *mut Result<Request, RequestError>;
    // // println!("mm_req_stage req: {:?}", unsafe { *req });
    // unsafe {
    //     println!("mm -> req: {}", req.as_ref().unwrap().unwrap().id);
    // }
    // // i
    // // println!("mm_req_stage data: {:?}", unsafe {
    // //     *((*req).unwrap().data)
    // // });
    black_box(context)
    // context
}
