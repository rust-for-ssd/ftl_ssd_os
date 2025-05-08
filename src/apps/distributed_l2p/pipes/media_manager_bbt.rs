use crate::{make_stage_static, shared::macros::ensure_unique};

make_stage_static!(media_manager_bbt_stage, init, exit, context_handler_mm_bbt);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_mm_bbt(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ensure_unique!();
    context
}
