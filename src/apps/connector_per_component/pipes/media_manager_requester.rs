use core::{ffi::c_void, hint::black_box};

use crate::{
    bindings::{generated::{ssd_os_msleep, ssd_os_usleep}, safe::ssd_os_sleep},
    make_stage_static, println,
    requester::requester::{Request, RequestError}, shared::macros::ensure_unique,
};

make_stage_static!(media_manager_requester_stage, init, exit, context_handler_mmr);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

// #[inline(never)]
// // #[unsafe(no_mangle)]
// fn good_stuff(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
//     // ssd_os_sleep(1);
//     // unsafe { ssd_os_msleep(1) };
//     // unsafe {ssd_os_usleep(1);}
//     // let req = context as *mut Result<Request, RequestError>;
//     // // println!("mm_req_stage req: {:?}", unsafe { *req });
//     // unsafe {
//     //     println!("mm -> req: {}", req.as_ref().unwrap().unwrap().id);
//     // }
//     // // i
//     // // println!("mm_req_stage data: {:?}", unsafe {
//     // //     *((*req).unwrap().data)
//     // // });
//     black_box(context)
//     // println!("here!");
//     // context
//     // context
// }

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_mmr(context: *mut c_void) -> *mut c_void {
    ensure_unique!();

    // unsafe { core::ptr::read_volatile(&DUMMY_MM1) };

    // black_box(context)
    // NO OPS to avoid function merge...
    unsafe { core::ptr::read_volatile(&DUMMY_L2P) };
    static DUMMY_L2P: u8 = 0;
    context
}
// static DUMMY_MM1: u8 = 0;
