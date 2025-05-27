use crate::make_stage_static;

make_stage_static!(l2p_prov_stage, init, exit, context_handler_l2p_prov);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_l2p_prov(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    context
}
