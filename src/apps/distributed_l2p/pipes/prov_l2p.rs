use crate::make_stage_static;

make_stage_static!(prov_l2p_stage, init, exit, context_handler_prov_l2p);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_prov_l2p(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    context
}
