use core::ffi::CStr;

use crate::{apps::distributed_l2p::connectors::l2p::l2p_tables::N_TABLES, make_stage_static};

const STAGE_NAMES: [&CStr; N_TABLES] = [
    l2p_media_manager_stage0.get_name(),
    l2p_media_manager_stage1.get_name(),
    l2p_media_manager_stage2.get_name(),
    l2p_media_manager_stage3.get_name(),
];

make_stage_static!(
    l2p_media_manager_stage0,
    init,
    exit,
    context_handler_l2p_media_manager0
);
make_stage_static!(
    l2p_media_manager_stage1,
    init,
    exit,
    context_handler_l2p_media_manager1
);
make_stage_static!(
    l2p_media_manager_stage2,
    init,
    exit,
    context_handler_l2p_media_manager2
);
make_stage_static!(
    l2p_media_manager_stage3,
    init,
    exit,
    context_handler_l2p_media_manager3
);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_l2p_media_manager0(
    context: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    context
}
#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_l2p_media_manager1(
    context: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    context
}
#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_l2p_media_manager2(
    context: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    context
}
#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_l2p_media_manager3(
    context: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    context
}
