use crate::{apps::connector_per_component::connectors::requester::{Request, RequestError}, make_stage_static, println};

make_stage_static!(media_manager_requester_stage, init, exit, context_handler);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("MEDIA MANAGER TO REQUEST STAGE");
    
    let req = context as *mut Result<Request, RequestError>;
    
    println!("MEDIA MANAGER TO REQUEST STAGE: {:?}", unsafe {*req});
    
    // We just propagete the context here.
    context
}