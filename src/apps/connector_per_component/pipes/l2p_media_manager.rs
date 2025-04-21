use crate::{
    apps::connector_per_component::connectors::requester::{Request, RequestError},
    make_stage_static, println,
};

make_stage_static!(l2p_media_manager_stage, init, exit, context_handler);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("L2P_MM_STAGE");

    let req = context as *mut Result<Request, RequestError>;

    match unsafe { *req } {
        Ok(ref mut request) => {
            println!("L2P_MM_STAGE: {:?}", request);
            // TRANSFORM TO CMD THAT MM UNDERSTANDS
            
        }
        Err(ref err) => {
            println!("L2P_MM_STAGE ERROR: {:?}", err);
        }
    }

    context
}
