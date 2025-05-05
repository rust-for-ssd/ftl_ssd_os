use core::ffi::c_void;

use crate::{make_stage_static, shared::macros::ensure_unique};

make_stage_static!(
    media_manager_requester_stage,
    init,
    exit,
    context_handler_mmr_req
);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_mmr_req(context: *mut c_void) -> *mut c_void {
    ensure_unique!();
    context
}
