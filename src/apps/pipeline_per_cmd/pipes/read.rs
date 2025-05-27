use crate::make_stage_static;
use crate::shared::addresses::PhysicalBlockAddress;
use crate::shared::macros::println;

use crate::requester::requester::Request;

make_stage_static!(read_l2p, init_l2p, exit, l2p_read_context_handler);
make_stage_static!(read_mm, init_mm, exit, mm_context_handler);

use super::write::{L2P_MAPPER, MM};

fn init_l2p() -> ::core::ffi::c_int {
    0
}

fn init_mm() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn l2p_read_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    let request = Request::from_ctx_ptr(context);
    let physcial_add = L2P_MAPPER.lock().lookup(request.logical_addr);
    match physcial_add {
        Some(_) => {
            request.physical_addr = physcial_add;
        }
        None => {
            println!("Logical address not mapped!");
            request.physical_addr = None;
        }
    }

    context
}

fn mm_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    let request = Request::from_ctx_ptr(context);

    let Ok(data) = MM.lock().execute_request(request) else {
        if let Some(_pba) = request.physical_addr {
            //TODO: only do this because ssd_os does not support LLVM 64-bit operations,
            // so we cannot convert ppa correctly.
            // if it works, then use pba directly.
            let pba = PhysicalBlockAddress {
                channel: 0,
                lun: 0,
                plane: 0,
                block: 0,
            };
            crate::apps::pipeline_per_cmd::pipes::write::BBT
                .lock()
                .set_bad_block(&pba);
        }
        return context;
    };
    request.data = data;
    context
}
