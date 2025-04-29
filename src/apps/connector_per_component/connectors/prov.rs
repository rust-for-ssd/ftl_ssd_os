use core::ptr::null_mut;

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator,
    bindings::{
        generated::{lring_entry, pipeline},
        lring::{LRing, LRingErr},
        mem::MemoryRegion,
        safe::{ssd_os_get_connection, ssd_os_sleep},
    },
    make_connector_static,
    media_manager::media_manager::Geometry,
    println,
    provisioner::provisioner::Provisioner,
    requester::requester::{Request, RequestError},
    shared::{addresses::PhysicalBlockAddress, core_local_cell::CoreLocalCell},
};

make_connector_static!(prov, init, exit, pipe_start, ring);

static lring: LRing<128> = LRing::new();
static ALLOC: SimpleAllocator = SimpleAllocator::new();
static provisioner: CoreLocalCell<Provisioner<SimpleAllocator>> = CoreLocalCell::new();

fn init() -> ::core::ffi::c_int {
    // println!("PROV_INIT");
    let mut mem_region = MemoryRegion::new_from_cpu(3);
    let Ok(()) = lring.init(c"L2P_LRING", mem_region.free_start, 0) else {
        panic!("L2P_LRING WAS ALREADY INITIALIZED!");
    };
    let ring = lring.get_lring().unwrap();
    mem_region.reserve(ring.alloc_mem as usize);

    ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());

    provisioner.set(Provisioner::new(
        &Geometry {
            n_pages: 16,
            n_of_ch: 4,
            lun_per_ch: 8,
            blk_per_lun: 16,
            pg_per_blk: 16,
        },
        &ALLOC,
    ));
    provisioner
        .get_mut()
        .push_free_block(&PhysicalBlockAddress {
            channel: 0,
            lun: 0,
            plane: 0,
            block: 0,
        });
    // println!("PROV_INIT_END");
    0
}

fn exit() -> ::core::ffi::c_int {
    // println!("EXIT!");
    0
}

fn pipe_start(entry: *mut lring_entry) -> *mut pipeline {
    // ssd_os_sleep(1);

    let Ok(res) = lring.dequeue_as_mut(entry) else {
        return null_mut();
    };
    let Some(Ok(req)) = res.get_ctx_as_mut::<Result<Request, RequestError>>() else {
        return null_mut();
    };

    let Ok(ppa) = provisioner.get_mut().provision_page() else {
        // println!("COULD NOT PROVISION!");
        return null_mut();
    };

    // TODO: problem because we cannot work with u64 as per 23/5
    // let nvm_ppa: nvm_ppa_addr = ppa.into();
    // req.physical_addr = Some(unsafe { nvm_ppa.__bindgen_anon_1.ppa } as u32);
    req.physical_addr = Some(ppa.into());

    return ssd_os_get_connection(c"prov", c"prov_l2p");
}

fn ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
    // ssd_os_sleep(1);
    match lring.enqueue(entry) {
        Ok(()) => 0,
        Err(LRingErr::Enqueue(i)) => i,
        _ => {
            println!("DID NOT MATCH RES FROM ENQUEUE!");
            -1
        }
    }
}
