use crate::allocator::sdd_os_alloc::SimpleAllocator;
use crate::bindings::generated::nvm_mmgr_geometry;
use crate::bindings::mem::MemoryRegion;
use crate::bindings::safe::ssd_os_sleep;
use crate::l2p::l2p::L2pMapper;
use crate::media_manager::media_manager::MediaManager;
use crate::provisioner::provisioner::Provisioner;
use crate::shared::addresses::PhysicalBlockAddress;
use crate::shared::core_local_cell::CoreLocalCell;
use crate::{make_stage_static, println};

use crate::requester::requester::{CommandType, Request, RequestError};

pub static PROV_ALLOC: SimpleAllocator = SimpleAllocator::new();
pub static PROVISIONER: CoreLocalCell<Provisioner<SimpleAllocator>> = CoreLocalCell::new();

// pub static MM_ALLOC: SimpleAllocator = SimpleAllocator::new();
// pub static MM: CoreLocalCell<MediaManager<SimpleAllocator>> = CoreLocalCell::new();

use super::read::{L2P_MAPPER, MM};

make_stage_static!(write_l2p, init_l2p, exit, l2p_context_handler);
make_stage_static!(write_prov, init_prov, exit, prov_context_handler);
make_stage_static!(write_mm, init_mm, exit, mm_context_handler);

fn init_l2p() -> ::core::ffi::c_int {
    ssd_os_sleep(1);
    println!("WRITE: INIT: L2P STAGE");
    // we already init INIT in read pipeline

    0
}

fn init_prov() -> ::core::ffi::c_int {
    ssd_os_sleep(1);
    println!("WRITE: INIT: PROV STAGE");
    let mem_region = MemoryRegion::new_from_cpu(3);
    println!("{:?}", mem_region.free_start);
    println!("{:?}", mem_region.end);
    PROV_ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());

    
    PROVISIONER.set(Provisioner::new(&MM.get().geometry, &PROV_ALLOC));
    
    let example_free_block = PhysicalBlockAddress { channel: 1, lun: 1, plane: 1, block: 1 };
    
    PROVISIONER.get_mut().push_free_block(&example_free_block);
    0
}

fn init_mm() -> ::core::ffi::c_int {
    ssd_os_sleep(1);
    println!("WRITE: INIT: MM STAGE");
    // we already init MM in read pipeline
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn prov_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_sleep(1);
    println!("WRITE: PROV STAGE");
    // We just propagete the context here.

    let req : &mut Result<Request, RequestError> =  unsafe { context.cast::<Result<Request, RequestError>>().as_mut().unwrap() };

    if let Ok(request) = req {
        // println!("L2P_WRITE_STAGE: {:?}", request);
        // Modify the value behind the context pointer
        request.physical_addr = Some(PROVISIONER.get_mut().provision_page().unwrap().into());
    }
    context
}

fn l2p_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_sleep(1);
    println!("WRITE: L2P STAGE");
    // We just propagete the context here.

    // TEST WE CAN GET IT
    // let res = L2P_MAPPER.get_mut().lookup(0x1);
    let Some(res) = L2P_MAPPER.get_mut().lookup(0x1) else {
        return context;
    };

    println!("RES {:?}", res);

    // let req : &mut Result<Request, RequestError> =  unsafe { context.cast::<Result<Request, RequestError>>().as_mut().unwrap() };

    // if let Ok(request) = req {
    //     // println!("L2P_WRITE_STAGE: {:?}", request);
    //     // Modify the value behind the context pointer
    //     request.physical_addr = Some(L2P_MAPPER.get_mut().lookup(request.logical_addr).unwrap());
    // }
    context
}

fn mm_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_sleep(1);

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
    context
}
