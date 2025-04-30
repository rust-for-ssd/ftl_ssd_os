use core::ptr::null_mut;

use crate::allocator::semaphore_alloc::SemaphoreAllocator;
use crate::apps::pipeline_per_cmd::connectors::requester::WORKLOAD_GENERATOR;
use crate::bbt::bbt::BadBlockTable;
use crate::bindings::mem::MemoryRegion;
use crate::l2p::l2p::L2pMapper;
use crate::media_manager::media_manager::MediaManager;
use crate::provisioner::provisioner::Provisioner;
use crate::shared::addresses::PhysicalBlockAddress;
use crate::shared::macros::ensure_unique;
use crate::shared::semaphore::Semaphore;
use crate::{make_stage_static, println};

use crate::requester::requester::{Request, RequestError};

pub static PROV_ALLOC: SemaphoreAllocator = SemaphoreAllocator::new();
pub static PROVISIONER: Semaphore<Provisioner<SemaphoreAllocator>> = Semaphore::new();
pub static BBT: Semaphore<BadBlockTable<SemaphoreAllocator>> = Semaphore::new();

pub static L2P_ALLOC: SemaphoreAllocator = SemaphoreAllocator::new();
pub static L2P_MAPPER: Semaphore<L2pMapper<SemaphoreAllocator>> = Semaphore::new();

pub static MM_ALLOC: SemaphoreAllocator = SemaphoreAllocator::new();
pub static MM: Semaphore<MediaManager<SemaphoreAllocator>> = Semaphore::new();

make_stage_static!(write_l2p, init_l2p, exit, l2p_context_handler);
make_stage_static!(write_prov, init_prov, exit, prov_context_handler);
make_stage_static!(write_mm, init_mm, exit, mm_context_handler);

fn init_l2p() -> ::core::ffi::c_int {
    ensure_unique!();

    let mem_region = MemoryRegion::new_from_cpu(2);
    L2P_ALLOC.init(mem_region.free_start.cast(), mem_region.end.cast());
    L2P_MAPPER.init(L2pMapper::new(&L2P_ALLOC));
    let n_requests = WORKLOAD_GENERATOR.get().get_n_requests();
    let mut l2p_map = L2P_MAPPER.lock();
    l2p_map.prepare_for_benchmark(n_requests);

    0
}

fn init_prov() -> ::core::ffi::c_int {
    ensure_unique!();

    let mem_region = MemoryRegion::new_from_cpu(3);
    PROV_ALLOC.init(mem_region.free_start.cast(), mem_region.end.cast());

    let geo = WORKLOAD_GENERATOR.get().get_geo();
    PROVISIONER.init(Provisioner::new(&geo, &PROV_ALLOC));
    BBT.init(BadBlockTable::new(&geo, &PROV_ALLOC));
    PROVISIONER.lock().init_free_from_bbt(&geo, &BBT.lock());

    0
}

fn init_mm() -> ::core::ffi::c_int {
    ensure_unique!();

    let mem_region = MemoryRegion::new_from_cpu(4);

    MM_ALLOC.init(mem_region.free_start.cast(), mem_region.end.cast());
    MM.init(MediaManager::new(&MM_ALLOC));
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn prov_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ensure_unique!();

    let req: &mut Result<Request, RequestError> = unsafe {
        context
            .cast::<Result<Request, RequestError>>()
            .as_mut()
            .unwrap()
    };

    if let Ok(request) = req {
        let Ok(ppa) = PROVISIONER.lock().provision_page() else {
            println!("COULD NOT PROVISION!");
            return null_mut();
        };

        request.physical_addr = Some(ppa.into());
    }
    context
}

fn l2p_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ensure_unique!();

    let req: &mut Result<Request, RequestError> = unsafe {
        context
            .cast::<Result<Request, RequestError>>()
            .as_mut()
            .unwrap()
    };

    if let Ok(request) = req {
        L2P_MAPPER
            .lock()
            .map(request.logical_addr, request.physical_addr.unwrap());
    }

    context
}

fn mm_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ensure_unique!();
    let req: &mut Result<Request, RequestError> = unsafe {
        context
            .cast::<Result<Request, RequestError>>()
            .as_mut()
            .unwrap()
    };

    if let Ok(request) = req {
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
                BBT.lock().set_bad_block(&pba);
            }
            return context;
        };
        request.data = data;
    }

    context
}
