use core::{mem::MaybeUninit, ptr::null_mut};

use alloc::boxed::Box;

use crate::{
    allocator::sdd_os_alloc::SimpleAllocator, apps::connector_per_component::connectors::requester::{Request, RequestError}, bindings::{generated::{nvm_mmgr_geometry, nvm_mmgr_io_cmd, nvm_ppa_addr, nvm_ppa_addr__bindgen_ty_1, volt_get_geometry}, mem::MemoryRegion}, make_stage_static, println, shared::core_local_cell::CoreLocalCell
};

static CMD: CoreLocalCell<Box<nvm_mmgr_io_cmd, &SimpleAllocator>> = CoreLocalCell::new();
static ALLOC: SimpleAllocator = SimpleAllocator::new();


make_stage_static!(l2p_media_manager_stage, init, exit, context_handler);

fn init() -> ::core::ffi::c_int {
    0
}

fn exit() -> ::core::ffi::c_int {
    0
}

fn context_handler(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println!("L2P_MM_STAGE");

    let req = context as *mut Result<Request, RequestError>;
    let mem_region = MemoryRegion::new_from_cpu(3);


    match unsafe { *req } {
        Ok(ref mut request) => {
            println!("L2P_MM_STAGE: {:?}", request);
            // TRANSFORM TO CMD THAT MM UNDERSTANDS
            // ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());
            
            // let geo: MaybeUninit<nvm_mmgr_geometry> = MaybeUninit::uninit();
           
            // unsafe { volt_get_geometry(geo.as_ptr().cast_mut()) };
           
            // let geo = unsafe { geo.assume_init() };
            
            // println!("GEO: {:?}", geo);
            
            // TODO WHEN IVAN ANSWERS 
            // let b = Box::new_in(nvm_mmgr_io_cmd { 
            //     nvm_io: null_mut(), 
            //     ppa: nvm_ppa_addr{ __bindgen_anon_1: nvm_ppa_addr__bindgen_ty_1{ppa: request.physical_addr.unwrap()}  },
            //     ch: todo!(), 
            //     callback: todo!(),
            //     prp: todo!(), 
            //     md_prp: todo!(),
            //     status: 0x2,
            //     cmdtype: 1, //todo
            //     pg_index: 0, 
            //     pg_sz: geo.pg_size, n_sectors: (geo.pg_size / geo.sec_size) as u16, 
            //     sec_sz: geo.sec_size, md_sz: geo.pg_oob_sz, 
            //     sec_offset: 0, 
            //     force_sync_md: todo!(), 
            //     force_sync_data: todo!(), 
            //     sync_count: todo!(), 
            //     rsvd: todo!() }, &ALLOC);
            
            // CMD.set(b);
            // let ptr = Box::into_raw(*CMD.get()) as *mut ::core::ffi::c_void;
            // ptr
            println!("Just propagates for now....");

            context
        }
        Err(ref err) => {
            println!("L2P_MM_STAGE ERROR: {:?}", err);
            null_mut()
        }
    }

}
