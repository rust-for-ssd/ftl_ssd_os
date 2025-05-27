use core::ffi::CStr;

use crate::{apps::distributed_l2p::connectors::l2p::l2p_tables::N_TABLES, make_stage_static};

const STAGE_NAMES: [&CStr; N_TABLES] = [
    dist_l2p_stage0.get_name(),
    dist_l2p_stage1.get_name(),
    dist_l2p_stage2.get_name(),
    dist_l2p_stage3.get_name(),
];

make_stage_static!(dist_l2p_stage0, init, exit, context_handler_dist_l2p0);
make_stage_static!(dist_l2p_stage1, init, exit, context_handler_dist_l2p1);
make_stage_static!(dist_l2p_stage2, init, exit, context_handler_dist_l2p2);
make_stage_static!(dist_l2p_stage3, init, exit, context_handler_dist_l2p3);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_dist_l2p0(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    context
}
#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_dist_l2p1(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    context
}
#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_dist_l2p2(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    context
}
#[inline(never)]
#[unsafe(no_mangle)]
fn context_handler_dist_l2p3(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    context
}
