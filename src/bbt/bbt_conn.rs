use crate::bbt::bbt::BadBlockTable;
use crate::sdd_os_alloc::SimpleAllocator;
use crate::ssd_os::lring::LRing;
use crate::{bindings, make_connector_static, make_stage_static, safe_bindings, shared};
use ::core::ffi::CStr;
use alloc::boxed::Box;
use alloc::vec::Vec;
use bindings::{lring_entry, nvm_mmgr_geometry, pipeline, volt_get_geometry};
use safe_bindings::{
    ssd_os_get_connection, ssd_os_mem_get, ssd_os_mem_size, ssd_os_print_lock, ssd_os_print_ss,
    ssd_os_print_unlock, ssd_os_sleep, ssd_os_this_cpu,
};
use shared::addresses::PhysicalBlockAddress;

use crate::{println_i, println_s};

#[global_allocator]
pub static ALLOCATOR: SimpleAllocator = SimpleAllocator::new();

const hello: [u8; 32] = *b"hello world\0....................";

make_stage_static!(bbt_stage, s1_init, s1_init, bbt_stage_fn);

fn s1_init() -> ::core::ffi::c_int {
    0
}

fn s1_exit() -> ::core::ffi::c_int {
    0
}

fn bbt_stage_fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    ssd_os_print_lock();
    ssd_os_print_ss(
        unsafe { CStr::from_ptr(context as *const u8) },
        c"BBT_STAGE\n",
    );
    ssd_os_print_unlock();
    context
}

make_connector_static!(bbt_conn, bbt_init, bbt_exit, bbt_conn_fn, bbt_ring);

static BBT: BadBlockTable = BadBlockTable::new();

fn bbt_init() -> ::core::ffi::c_int {
    println_s!(c"init start:");
    let cpu_id = ssd_os_this_cpu(bbt_conn.get_name());
    let memory_region = ssd_os_mem_get(cpu_id) as usize;
    let memory_size = ssd_os_mem_size(cpu_id) as usize;
    println_s!(c"yo1:");
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

    println_s!(c"yo2:");
    println_i!((&mut geo as *mut nvm_mmgr_geometry) as u32);
    unsafe { volt_get_geometry(&mut geo as *mut nvm_mmgr_geometry) };
    println_s!(c"yo3:");

    assert_eq!(
        (&ALLOCATOR as *const _ as usize) % core::mem::align_of::<usize>(),
        0
    );
    ALLOCATOR.initialize(memory_region, memory_region + memory_size);

    println_s!(c"alloc location bbt:");
    println_i!(&ALLOCATOR as *const _ as u32);

    println_s!(c"yoyo:");

    BBT.init(&geo);

    println_s!(c"init ring");
    // bbt_lring.init(c"BBT_LRING", 0);

    println_s!(c"Channel len");
    println_i!(BBT.channels.borrow().len() as u32);

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

    println_s!(c"Bad block");
    println_i!(BBT.get_block_status(&pba) as u32);

    println_s!(c"Another bad block");
    println_i!(BBT.get_block_status(&pba_bad_check) as u32);

    println_s!(c"Size of bbt");

    let mut heap_val1: alloc::vec::Vec<u32> = alloc::vec::Vec::with_capacity(3);

    heap_val1.push(42);
    println_i!(heap_val1[0]);
    heap_val1[0] = 69;
    println_i!(heap_val1[0]);
    heap_val1.push(3);

    let b1 = Box::new(41u32);
    let b2 = Box::new(42u32);
    let b3 = Box::new(43u32);
    println_i!(*b1);
    println_i!(*b2);
    println_i!(*b3);

    0
}

fn bbt_exit() -> ::core::ffi::c_int {
    0
}

fn bbt_ring(ring: *mut lring_entry) -> ::core::ffi::c_int {
    ssd_os_print_lock();
    ssd_os_print_ss(
        unsafe { CStr::from_ptr(ring.as_ref().unwrap().ctx as *const u8) },
        c"END\n",
    );
    ssd_os_print_unlock();
    0
}

static bbt_lring: LRing<128> = LRing::new();

fn bbt_conn_fn(entry: *mut lring_entry) -> *mut pipeline {
    let Some(_entry) = bbt_lring.dequeue(entry) else {
        let pipe = ssd_os_get_connection(
            c"bbt_conn".as_ptr().cast_mut(),
            c"gc_pipe".as_ptr().cast_mut(),
        );
        if !pipe.is_null() {
            unsafe { entry.as_mut().unwrap() }.ctx = hello.as_ptr() as *mut ::core::ffi::c_void;
            ssd_os_print_ss(
                unsafe { CStr::from_ptr(entry.as_ref().unwrap().ctx as *const u8) },
                c"START\n",
            );
            println_s!(c"Accessing BBT from conn function");
            let pba_bad: PhysicalBlockAddress = PhysicalBlockAddress {
                channel: 0,
                lun: 0,
                plane: 0,
                block: 0,
            };

            let pba_good: PhysicalBlockAddress = PhysicalBlockAddress {
                channel: 0,
                lun: 0,
                plane: 0,
                block: 1,
            };
            println_s!(c"BAD: (SHOULD BE 0)");
            println_i!(BBT.get_block_status(&pba_bad) as u32);

            println_s!(c"GOD: (SHOULD BE 1)");
            println_i!(BBT.get_block_status(&pba_good) as u32);

            println_s!(c"MUTATING BAD BLOCK TABLE!!");
            BBT.set_bad_block(&pba_good);

            println_s!(c"SHOULD NOW BE SET TO BAD (0)");
            println_i!(BBT.get_block_status(&pba_good) as u32);

            ssd_os_sleep(1);
            return pipe;
        } else {
            return ::core::ptr::null_mut();
        }
    };
    return ::core::ptr::null_mut();
}
