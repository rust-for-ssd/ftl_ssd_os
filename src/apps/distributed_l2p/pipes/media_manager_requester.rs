use core::ffi::{CStr, c_void};

use crate::{
    apps::distributed_l2p::connectors::l2p::l2p_tables::N_TABLES, make_stage_static,
    shared::macros::ensure_unique,
};

const STAGE_NAMES: [&CStr; N_TABLES] = [
    media_manager_requester_stage0.get_name(),
    media_manager_requester_stage1.get_name(),
    media_manager_requester_stage2.get_name(),
    media_manager_requester_stage3.get_name(),
];

make_stage_static!(
    media_manager_requester_stage0,
    init,
    exit,
    context_handler_mmr_req0
);
make_stage_static!(
    media_manager_requester_stage1,
    init,
    exit,
    context_handler_mmr_req1
);
make_stage_static!(
    media_manager_requester_stage2,
    init,
    exit,
    context_handler_mmr_req2
);
make_stage_static!(
    media_manager_requester_stage3,
    init,
    exit,
    context_handler_mmr_req3
);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_mmr_req0(context: *mut c_void) -> *mut c_void {
    ensure_unique!();
    context
}
#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_mmr_req1(context: *mut c_void) -> *mut c_void {
    ensure_unique!();
    context
}
#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_mmr_req2(context: *mut c_void) -> *mut c_void {
    ensure_unique!();
    context
}
#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_mmr_req3(context: *mut c_void) -> *mut c_void {
    ensure_unique!();
    context
}
