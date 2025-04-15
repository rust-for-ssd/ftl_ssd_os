use crate::bindings::lring_entry;
use crate::bindings::nvm_mmgr_geometry;
use crate::bindings::pipeline;
use crate::bindings::ssd_os_lring_dequeue;
use crate::bindings::ssd_os_lring_enqueue;
use crate::bindings::volt_get_geometry;
use crate::sdd_os_alloc::SimpleAllocator;
use crate::ssd_os::lring::LRing;
use crate::{make_connector_static, make_stage_static, safe_bindings, shared};
use ::core::ffi::CStr;
use alloc::boxed::Box;
use safe_bindings::{
    ssd_os_get_connection, ssd_os_mem_get, ssd_os_mem_size, ssd_os_print_lock, ssd_os_print_ss,
    ssd_os_print_unlock, ssd_os_sleep, ssd_os_this_cpu,
};
use shared::addresses::PhysicalBlockAddress;

use crate::{println_i, println_s};

use super::bbt::BadBlockTable;

// static BBT_ALLOCATOR: SimpleAllocator = SimpleAllocator::new();
static BBT_ALLOCATOR: SimpleAllocator = SimpleAllocator::new();

fn s1_init() -> ::core::ffi::c_int {
    0
}

fn s1_exit() -> ::core::ffi::c_int {
    0
}

make_connector_static!(bbt_conn, bbt_init, bbt_exit, bbt_conn_fn, bbt_ring);
make_stage_static!(bbt_cstage, s1_init, s1_init, bbt_cstage_fn);


fn bbt_cstage_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    println_s!(c"BBT COMPLETION STAGE");
    context
}

static BBT: BadBlockTable<SimpleAllocator> = BadBlockTable::new();

fn bbt_init() -> ::core::ffi::c_int {
    println_s!(c"BBT CONNECTOR: INIT START");
    let cpu_id = ssd_os_this_cpu(bbt_conn.get_name());
    let memory_region = ssd_os_mem_get(cpu_id);
    let memory_size = ssd_os_mem_size(cpu_id);
    // println_s!(c"yo1:");
    let mut geo: nvm_mmgr_geometry = nvm_mmgr_geometry {
        n_of_ch: 10,
        lun_per_ch: 2,
        blk_per_lun: 2,
        pg_per_blk: 2,
        sec_per_pg: 2,
        n_of_planes: 2,
        pg_size: 2,
        sec_oob_sz: 2,
        sec_per_pl_pg: 2,
        sec_per_blk: 2,
        sec_per_lun: 2,
        sec_per_ch: 2,
        pg_per_lun: 2,
        pg_per_ch: 2,
        blk_per_ch: 2,
        tot_sec: 2,
        tot_pg: 2,
        tot_blk: 2,
        tot_lun: 2,
        sec_size: 2,
        pl_pg_size: 2,
        blk_size: 2,
        lun_size: 2,
        ch_size: 2,
        tot_size: 2,
        pg_oob_sz: 2,
        pl_pg_oob_sz: 2,
        blk_oob_sz: 2,
        lun_oob_sz: 2,
        ch_oob_sz: 2,
        tot_oob_sz: 2,
    };

    // println_s!(c"yo2:");
    // println_i!((&mut geo as *mut nvm_mmgr_geometry) as u32);
    unsafe { volt_get_geometry(&mut geo as *mut nvm_mmgr_geometry) };
    // println_s!(c"yo3:");

    assert_eq!(
        (&BBT_ALLOCATOR as *const _ as usize) % core::mem::align_of::<usize>(),
        0
    );
    unsafe {
        BBT_ALLOCATOR.initialize(
            memory_region.cast(),
            memory_region.add(memory_size as usize).cast(),
        )
    };

    // println_s!(c"alloc location bbt:");
    // println_i!(&BBT_ALLOCATOR as *const _ as u32);

    // println_s!(c"yoyo:");

    let _ = BBT.init(&geo, &BBT_ALLOCATOR);

    // println_s!(c"init ring");
    println_s!(c"BBT CONNECTOR: INIT RING");

    bbt_lring.init(c"BBT_LRING", 0);

    // println_s!(c"Channel len");
    // println_i!(BBT.channels.borrow().len() as u32);

    ssd_os_sleep(1);

    let pba: PhysicalBlockAddress = PhysicalBlockAddress {
        channel: 0,
        lun: 0,
        plane: 0,
        block: 0,
    };

    let pba_bad_check: PhysicalBlockAddress = PhysicalBlockAddress {
        channel: 0,
        lun: 0,
        plane: 0,
        block: 5,
    };

    for i in 0..10 {
        let pba2: PhysicalBlockAddress = PhysicalBlockAddress {
            channel: 0,
            lun: 0,
            plane: 0,
            block: i,
        };
        BBT.set_bad_block(&pba2);
    }

    // println_s!(c"Bad block");
    // println_i!(BBT.get_block_status(&pba) as u32);

    // println_s!(c"Another bad block");
    // println_i!(BBT.get_block_status(&pba_bad_check) as u32);

    // println_s!(c"Size of bbt");

    let mut heap_val1: alloc::vec::Vec<u32, &SimpleAllocator> =
        alloc::vec::Vec::with_capacity_in(3, &BBT_ALLOCATOR);

    heap_val1.push(42);
    println_i!(heap_val1[0]);
    heap_val1[0] = 69;
    println_i!(heap_val1[0]);
    heap_val1.push(3);

    let b1 = Box::new_in(41u32, &BBT_ALLOCATOR);
    let b2 = Box::new_in(42u32, &BBT_ALLOCATOR);
    let b3 = Box::new_in(43u32, &BBT_ALLOCATOR);
    // println_i!(*b1);
    // println_i!(*b2);
    // println_i!(*b3);
    println_s!(c"BBT CONNECTOR: BBT ALLOCATED");

    println_s!(c"BBT CONNECTOR: INIT END");

    0
}

fn bbt_exit() -> ::core::ffi::c_int {
    0
}

fn bbt_ring(ring: *mut lring_entry) -> ::core::ffi::c_int {
    // println_i!(unsafe {ring.as_ref().unwrap().ctx as u32});
    unsafe { ssd_os_lring_enqueue(*bbt_lring.ssd_os_lring.get().unwrap() , ring)};
    0
}

static bbt_lring: LRing<128> = LRing::new();

fn bbt_conn_fn(entry: *mut lring_entry) -> *mut pipeline {
    println_s!(c"BBT COMPLETION PIPE");
    ssd_os_sleep(1);

    // let Some(_entry) = bbt_lring.dequeue(entry) else {
    //     return ::core::ptr::null_mut();
    // };
    // 
    let res = unsafe { ssd_os_lring_dequeue(*bbt_lring.ssd_os_lring.get().unwrap(), entry)};
    println_i!(res as u32);
    
    if entry.is_null(){
        println_s!(c"HERE");

        return ::core::ptr::null_mut();
    }

    println_s!(c"I'm NOT CRSHED");

    println_i!(unsafe{entry.as_ref().unwrap().ctx as u32});
    
    let pipe = ssd_os_get_connection(
        c"bbt_conn".as_ptr().cast_mut(),
        c"bbt_cpipe".as_ptr().cast_mut(),
    );
    if !pipe.is_null() {
        println_s!(c"CREATED COMPLETION PIPE");

        return pipe;
}
    
    return ::core::ptr::null_mut();
}
