use crate::{make_stage_static, shared::macros::ensure_unique};

make_stage_static!(l2p_media_manager_stage, init, exit, context_handler_l2p_mm);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_l2p_mm(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ensure_unique!();
    context
}
