use crate::{make_stage_static, shared::macros::ensure_unique};

make_stage_static!(
    bbt_requester_stage,
    init,
    exit,
    context_handler_bbt_requester
);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_bbt_requester(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ensure_unique!();
    context
}
