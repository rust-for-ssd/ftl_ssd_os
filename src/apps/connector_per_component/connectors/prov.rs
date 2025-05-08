use core::ptr::null_mut;

use crate::shared::macros::println;
use crate::{
    allocator::linked_list_alloc::LinkedListAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::ssd_os_get_connection,
    },
    make_connector_static,
    provisioner::provisioner::Provisioner,
    requester::requester::Request,
    shared::core_local_cell::CoreLocalCell,
};

use super::requester::WORKLOAD_GENERATOR;

make_connector_static!(prov, init, exit, pipe_start, ring, 1);

static lring: LRing<128> = LRing::new();
static ALLOC: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
static provisioner: CoreLocalCell<Provisioner<LinkedListAllocator>> = CoreLocalCell::new();

fn init() -> ::core::ffi::c_int {
    let mut mem_region = MemoryRegion::new_from_cpu(3);
    let Ok(()) = lring.init(c"L2P_LRING", mem_region.free_start, 0) else {
        panic!("L2P_LRING WAS ALREADY INITIALIZED!");
    };
    mem_region.reserve(lring.get_lring().unwrap().alloc_mem as usize);

    ALLOC.set(LinkedListAllocator::new());
    ALLOC
        .get()
        .initialize(mem_region.free_start.cast(), mem_region.end.cast());

    let geo = WORKLOAD_GENERATOR.get().get_geo();
    provisioner.set(Provisioner::new(&geo, ALLOC.get()));

    // SAFETY: we access the BBT here directly because it is in the init, which is data race safe.
    provisioner
        .get_mut()
        .init_free_from_bbt(&geo, super::bbt::BBT.get());
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

    // let Ok(ppa) = provisioner.get_mut().provision_page() else {
    //     return null_mut();
    // };

    // TODO: problem because we cannot work with u64 as per 23/5
    // let nvm_ppa: nvm_ppa_addr = ppa.into();
    // req.physical_addr = Some(unsafe { nvm_ppa.__bindgen_anon_1.ppa } as u32);
    // req.physical_addr = Some(ppa.into());
    req.physical_addr = Some(0x1);


    return ssd_os_get_connection(c"prov", c"prov_l2p");
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
