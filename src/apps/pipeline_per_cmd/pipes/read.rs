use crate::allocator::sdd_os_alloc::SimpleAllocator;
use crate::bindings::mem::MemoryRegion;
use crate::bindings::safe::ssd_os_sleep;
use crate::l2p::l2p::L2pMapper;
use crate::media_manager::media_manager::MediaManager;
use crate::shared::core_local_cell::CoreLocalCell;
use crate::{make_stage_static, println};

use crate::requester::requester::{CommandType, Request, RequestError};

pub static L2P_ALLOC: SimpleAllocator = SimpleAllocator::new();
pub static L2P_MAPPER: CoreLocalCell<L2pMapper<SimpleAllocator>> = CoreLocalCell::new();

pub static MM_ALLOC: SimpleAllocator = SimpleAllocator::new();
pub static MM: CoreLocalCell<MediaManager<SimpleAllocator>> = CoreLocalCell::new();

make_stage_static!(read_l2p, init_l2p, exit, l2p_read_context_handler);
make_stage_static!(read_mm, init_mm, exit, mm_context_handler);

fn init_l2p() -> ::core::ffi::c_int {
    ssd_os_sleep(1);
    println!("READ: INIT: L2P STAGE");
    let mem_region = MemoryRegion::new_from_cpu(1);
    println!("{:?}", mem_region.free_start);
    println!("{:?}", mem_region.end);

    L2P_ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());
    L2P_MAPPER.set(L2pMapper::new(&L2P_ALLOC));
    L2P_MAPPER.get_mut().map(0x1, 0x1234);
    L2P_MAPPER.get_mut().map(0x2, 0x5555);

    0
}

fn init_mm() -> ::core::ffi::c_int {
    ssd_os_sleep(1);
    println!("READ: INIT: MM STAGE");
    let mem_region = MemoryRegion::new_from_cpu(2);
    println!("{:?}", mem_region.free_start);
    println!("{:?}", mem_region.end);

    MM_ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());
    MM.set(MediaManager::new(&MM_ALLOC));

    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn l2p_read_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_sleep(1);
    println!("READ: L2P STAGE");
    // We just propagete the context here.

    let req : &mut Result<Request, RequestError> =  unsafe { context.cast::<Result<Request, RequestError>>().as_mut().unwrap() };
    
    if let Ok(request) = req {
        // println!("L2P_READ_STAGE: {:?}", request);
        // Modify the value behind the context pointer 
        request.physical_addr = Some(L2P_MAPPER.get_mut().lookup(request.logical_addr).unwrap());
    }
    context
}

fn mm_context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_sleep(1);

    // println!("READ: MM STAGE");
    // let req = context as *mut Result<Request, RequestError>;
    let req : &mut Result<Request, RequestError> =  unsafe { context.cast::<Result<Request, RequestError>>().as_mut().unwrap() };
    
    if let Ok(request) = req {
        // println!("L2P_READ_STAGE: {:?}", request);
        // Modify the value behind the context pointer 
        request.data = MM.get_mut().execute_request(request).unwrap();
    }

    // println!("REQUESTER TO L2P STAGE: {:?}", unsafe {*req});

    // We just propagete the context here.
    context
}
