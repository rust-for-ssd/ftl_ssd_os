use core::ptr::null_mut;

use crate::bindings::safe::ssd_os_get_connection;
use crate::shared::macros::println;
use crate::{
    allocator::linked_list_alloc::LinkedListAllocator,
    bbt::bbt::BadBlockTable,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
    },
    make_connector_static,
    requester::requester::Request,
    shared::{addresses::PhysicalBlockAddress, core_local_cell::CoreLocalCell},
};

use super::requester::WORKLOAD_GENERATOR;

make_connector_static!(bbt, init, exit, pipe_start, ring, 1);

static lring: LRing<128> = LRing::new();
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
pub static BBT: CoreLocalCell<BadBlockTable<LinkedListAllocator>> = CoreLocalCell::new();

fn init() -> ::core::ffi::c_int {
    let mut mem_region = MemoryRegion::new_from_cpu(5);
    let Ok(()) = lring.init(c"BBT_LRING", mem_region.free_start, 0) else {
        panic!("BBT_LRING WAS ALREADY INITIALIZED!");
    };
    mem_region.reserve(lring.get_lring().unwrap().alloc_mem as usize);

    ALLOC.set(LinkedListAllocator::new());
    ALLOC
        .get()
        .initialize(mem_region.free_start.cast(), mem_region.end.cast());

    let geo = WORKLOAD_GENERATOR.get().get_geo();
    BBT.set(BadBlockTable::new(&geo, ALLOC.get()));
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    let Ok(res) = lring.dequeue_as_mut(entry) else {
        return null_mut();
    };
    let Some(req) = res.get_ctx_as_mut::<Request>() else {
        return null_mut();
    };

    let Some(_ppa) = req.physical_addr else {
        return null_mut();
    };

    // TODO: placeholder for now as we cannot convert u32 into PhysicalBlockAddress
    // if ssd_os supports 64-bit operations for LLVM, then this should be possible with the nvm_addr
    let pba = &PhysicalBlockAddress {
        channel: 0,
        lun: 0,
        plane: 0,
        block: 0,
    };

    BBT.get_mut().set_bad_block(pba);

    return ssd_os_get_connection(c"bbt", c"bbt_requester");
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    match lring.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}
