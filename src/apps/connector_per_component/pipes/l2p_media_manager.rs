use core::{hint::black_box, mem::MaybeUninit, ptr::null_mut};

use alloc::boxed::Box;

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator,
    bindings::{
        generated::{
            NVM_IO_PROCESS, nvm_callback, nvm_channel, nvm_mmgr_geometry, nvm_mmgr_io_cmd,
            nvm_ppa_addr, nvm_ppa_addr__bindgen_ty_1, volt_get_geometry,
        },
        mem::MemoryRegion,
    },
    make_stage_static, println,
    requester::requester::{Request, RequestError},
    shared::core_local_cell::CoreLocalCell,
};

static CMD: CoreLocalCell<Box<nvm_mmgr_io_cmd, &SimpleAllocator>> = CoreLocalCell::new();
// static ALLOC: SimpleAllocator = SimpleAllocator::new();

make_stage_static!(l2p_media_manager_stage, init, exit, context_handler);

// static GEO: CoreLocalCell<nvm_mmgr_geometry> = CoreLocalCell::new();

fn init() -> ::core::ffi::c_int {
    // println!("I AM BEING PRINTEDDDDDDAWDDSDASDSADASS!!!!!");
    // let geo: MaybeUninit<nvm_mmgr_geometry> = MaybeUninit::uninit();
    // unsafe { volt_get_geometry(geo.as_ptr().cast_mut()) };
    // let geo = unsafe { geo.assume_init() };
    // GEO.set(geo);
    // println!("GEO: {:?}", GEO.get().n_of_ch);
    // println!("GEO: {:p}", &GEO as *const CoreLocalCell<_>);

    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("L2P_MM_STAGE");

    let req = context as *mut Result<Request, RequestError>;
    unsafe {
        println!("l2p_mm:{:?}", req.as_ref().unwrap());
    }
    // let mem_region = MemoryRegion::new_from_cpu(3);
    return context;

    match unsafe { *req } {
        Ok(ref mut request) => {
            println!("L2P_MM_STAGE: {:?}", request);
            // println!("GEO: {:?}", GEO.get().n_of_ch);
            // TRANSFORM TO CMD THAT MM UNDERSTANDS
            // ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());

            // let geo: MaybeUninit<nvm_mmgr_geometry> = MaybeUninit::uninit();

            // unsafe { volt_get_geometry(geo.as_ptr().cast_mut()) };

            println!("Just propagates for now....");

            context
        }
        Err(ref err) => {
            println!("L2P_MM_STAGE ERROR: {:?}", err);
            null_mut()
        }
    }
}
