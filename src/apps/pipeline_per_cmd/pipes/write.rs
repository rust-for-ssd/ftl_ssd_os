use core::ptr::null_mut;

use crate::allocator::semaphore_alloc::SemaphoreAllocator;
use crate::apps::pipeline_per_cmd::connectors::requester::WORKLOAD_GENERATOR;
use crate::bindings::mem::MemoryRegion;
use crate::bindings::safe::ssd_os_sleep;
use crate::l2p::l2p::L2pMapper;
use crate::media_manager::media_manager::{Geometry, MediaManager};
use crate::provisioner::provisioner::Provisioner;
use crate::shared::addresses::PhysicalBlockAddress;
use crate::shared::core_local_cell::CoreLocalCell;
use crate::shared::macros::ensure_unique;
use crate::shared::semaphore::Semaphore;
use crate::{make_stage_static, println};

use crate::requester::requester::{Request, RequestError};

pub static PROV_ALLOC: SemaphoreAllocator = SemaphoreAllocator::new();
pub static PROVISIONER: Semaphore<Provisioner<SemaphoreAllocator>> = Semaphore::new();

pub static L2P_ALLOC: SemaphoreAllocator = SemaphoreAllocator::new();
pub static L2P_MAPPER: Semaphore<L2pMapper<SemaphoreAllocator>> = Semaphore::new();

pub static MM_ALLOC: SemaphoreAllocator = SemaphoreAllocator::new();
pub static MM: Semaphore<MediaManager<SemaphoreAllocator>> = Semaphore::new();

make_stage_static!(write_l2p, init_l2p, exit, l2p_context_handler);
make_stage_static!(write_prov, init_prov, exit, prov_context_handler);
make_stage_static!(write_mm, init_mm, exit, mm_context_handler);

fn init_l2p() -> ::core::ffi::c_int {
    ensure_unique!();

    #[cfg(feature = "debug")]
    {
    ssd_os_sleep(1);
    println!("WRITE: INIT: L2P STAGE");
        
    }

    let mem_region = MemoryRegion::new_from_cpu(2);
    println!("{:?}", mem_region.free_start);
    println!("{:?}", mem_region.end);

    L2P_ALLOC.init(mem_region.free_start.cast(), mem_region.end.cast());
    L2P_MAPPER.init(L2pMapper::new(&L2P_ALLOC));
    let n_requests = WORKLOAD_GENERATOR.get().get_n_requests();
    let mut l2p_map = L2P_MAPPER.lock();
    l2p_map.prepare_for_benchmark(n_requests);

    
    // #[cfg(feature = "benchmark")]
    // {
    // let l2p_map = l2p_mapper.get_mut();
    // l2p_map.prepare_for_benchmark(n_requests);
    // }
    
    // l2p_mapper.map(0x0, 0x1234);
    // l2p_mapper.map(0x2, 0x5555);
    // for i in 0..128 {
    //     let i = i as u32;
    //     l2p_mapper.map(i, i);
    // }
    0
}

fn init_prov() -> ::core::ffi::c_int {
    ensure_unique!();

    // ssd_os_sleep(1);
    // println!("WRITE: INIT: PROV STAGE");
    let mem_region = MemoryRegion::new_from_cpu(3);
    println!("{:?}", mem_region.free_start);
    println!("{:?}", mem_region.end);
    PROV_ALLOC.init(mem_region.free_start.cast(), mem_region.end.cast());
    
    let geo = WORKLOAD_GENERATOR.get().get_geo();
    PROVISIONER.init(Provisioner::new(&geo, &PROV_ALLOC));
    PROVISIONER.lock().init_all_free();
    
    0
}

fn init_mm() -> ::core::ffi::c_int {
    ensure_unique!();

    // ssd_os_sleep(1);
    // println!("WRITE: INIT: MM STAGE");
    let mem_region = MemoryRegion::new_from_cpu(4);
    println!("{:?}", mem_region.free_start);
    println!("{:?}", mem_region.end);

    MM_ALLOC.init(mem_region.free_start.cast(), mem_region.end.cast());
    MM.init(MediaManager::new(&MM_ALLOC));
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn prov_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ensure_unique!();

    // ssd_os_sleep(1);
    // println!("WRITE: PROV STAGE");
    // We just propagete the context here.

    let req: &mut Result<Request, RequestError> = unsafe {
        context
            .cast::<Result<Request, RequestError>>()
            .as_mut()
            .unwrap()
    };

    if let Ok(request) = req {
        // println!("L2P_WRITE_STAGE: {:?}", request);
        // Modify the value behind the context pointer
        // request.physical_addr = PROVISIONER.get_mut().provision_page().unwrap().into();

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

    // ssd_os_sleep(1);
    // println!("WRITE: L2P STAGE");
    // We just propagete the context here.

    // TEST WE CAN GET IT
    // let res = L2P_MAPPER.get_mut().lookup(0x1);
    let req: &mut Result<Request, RequestError> = unsafe {
        context
            .cast::<Result<Request, RequestError>>()
            .as_mut()
            .unwrap()
    };

    if let Ok(request) = req {
        // println!("L2P_WRITE_STAGE: {:?}", request);
        // Modify the value behind the context pointer
        L2P_MAPPER
            .lock()
            .map(request.logical_addr, request.physical_addr.unwrap());
    }

    // let Some(res) = L2P_MAPPER.get_mut().map() else {
    //     return context;
    // };

    // println!("RES {:?}", res);

    // let req : &mut Result<Request, RequestError> =  unsafe { context.cast::<Result<Request, RequestError>>().as_mut().unwrap() };

    // if let Ok(request) = req {
    //     // println!("L2P_WRITE_STAGE: {:?}", request);
    //     // Modify the value behind the context pointer
    //     request.physical_addr = Some(L2P_MAPPER.get_mut().lookup(request.logical_addr).unwrap());
    // }
    context
}

fn mm_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ensure_unique!();
    // ssd_os_sleep(1);

    // println!("WRITE: MM STAGE");
    // let req = context as *mut Result<Request, RequestError>;
    // let req : &mut Result<Request, RequestError> =  unsafe { context.cast::<Result<Request, RequestError>>().as_mut().unwrap() };

    // if let Ok(request) = req {
    //     // println!("L2P_WRITE_STAGE: {:?}", request);
    //     // Modify the value behind the context pointer
    //     request.data = MM.get_mut().execute_request(request, None).unwrap();
    // }

    // println!("REQUESTER TO L2P STAGE: {:?}", unsafe {*req});

    // We just propagete the context here.

    let req: &mut Result<Request, RequestError> = unsafe {
        context
            .cast::<Result<Request, RequestError>>()
            .as_mut()
            .unwrap()
    };

    if let Ok(request) = req {
        // println!("L2P_READ_STAGE: {:?}", request);
        // Modify the value behind the context pointer
        request.data = MM.lock().execute_request(request).unwrap();
    }

    context
}
