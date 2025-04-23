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
    println!("I AM BEING PRINTEDDDDDDAWDDSDASDSADASS!!!!!");
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
    // let mem_region = MemoryRegion::new_from_cpu(3);

    match unsafe { *req } {
        Ok(ref mut request) => {
            println!("L2P_MM_STAGE: {:?}", request);
            // println!("GEO: {:?}", GEO.get().n_of_ch);
            // TRANSFORM TO CMD THAT MM UNDERSTANDS
            // ALLOC.initialize(mem_region.free_start.cast(), mem_region.end.cast());

            let geo: MaybeUninit<nvm_mmgr_geometry> = MaybeUninit::uninit();

            unsafe { volt_get_geometry(geo.as_ptr().cast_mut()) };

            let geo = unsafe { geo.assume_init() };
            println!("GEO: {:?}", geo.n_of_ch);
            println!(
                "total entries: {:?}",
                geo.n_of_ch as u32
                    * geo.lun_per_ch as u32
                    * geo.blk_per_lun as u32
                    * geo.pg_per_blk as u32
                    * geo.sec_per_pg as u32
                    * geo.n_of_planes as u32
            );

            // TODO WHEN IVAN ANSWERS
            let ppa = nvm_ppa_addr {
                __bindgen_anon_1: nvm_ppa_addr__bindgen_ty_1 {
                    ppa: request.physical_addr.unwrap() as u64,
                },
            };

            let ch: *mut nvm_channel = null_mut();
            let dma_mem: *mut u8 = null_mut();
            let prp: [u64; 32] = [0; 32];

            let callback: nvm_callback = nvm_callback {
                cb_fn: None,
                opaque: null_mut(),
                ts: 0,
            };

            let cmd: nvm_mmgr_io_cmd = nvm_mmgr_io_cmd {
                nvm_io: null_mut(),
                ppa: ppa,
                ch: ch,
                callback: callback,
                prp: prp,
                md_prp: unsafe {
                    dma_mem.byte_add(((geo.pg_size / geo.sec_size) * geo.sec_size) as usize) as u64
                },
                status: NVM_IO_PROCESS as u8,
                cmdtype: 1,
                pg_index: 0,
                pg_sz: geo.pg_size,
                n_sectors: (geo.pg_size / geo.sec_size) as u16, // WARNING!
                sec_sz: geo.sec_size,
                md_sz: geo.pg_oob_sz,
                sec_offset: 0,
                force_sync_md: 1,
                force_sync_data: [0; 32],
                sync_count: 0,
                rsvd: [0; 128],
            };
            black_box(cmd);
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
