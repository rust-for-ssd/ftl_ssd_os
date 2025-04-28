use crate::bindings::safe::ssd_os_sleep;
use crate::{make_stage_static, println};

use crate::requester::requester::{Request, RequestError};

make_stage_static!(read_l2p, init_l2p, exit, l2p_read_context_handler);
make_stage_static!(read_mm, init_mm, exit, mm_context_handler);

use super::write::{L2P_MAPPER, MM};

fn init_l2p() -> ::core::ffi::c_int {
    0
}

fn init_mm() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn l2p_read_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    // ssd_os_sleep(1);
    println!("READ: L2P STAGE");
    // We just propagete the context here.

    let req: &mut Result<Request, RequestError> = unsafe {
        context
            .cast::<Result<Request, RequestError>>()
            .as_mut()
            .unwrap()
    };

    if let Ok(request) = req {
        println!("L2P_READ_STAGE: {:?}", request);
        // Modify the value behind the context pointer
        request.physical_addr = Some(L2P_MAPPER.lock().lookup(request.logical_addr).unwrap());
    }
    context
}

fn mm_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    // ssd_os_sleep(1);

    // println!("READ: MM STAGE");
    // let req = context as *mut Result<Request, RequestError>;
    let req: &mut Result<Request, RequestError> = unsafe {
        context
            .cast::<Result<Request, RequestError>>()
            .as_mut()
            .unwrap()
    };

    if let Ok(request) = req {
        // println!("L2P_READ_STAGE: {:?}", request);
        // Modify the value behind the context pointer
        request.data = MM.lock().execute_request(request).unwrap();
    }

    // println!("REQUESTER TO L2P STAGE: {:?}", unsafe {*req});

    // We just propagete the context here.
    context
}
