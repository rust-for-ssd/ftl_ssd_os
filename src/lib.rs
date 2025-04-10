#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(static_mut_refs)]
mod bindings;
mod my_alloc;
mod safe_bindings;
mod shared;
mod bbt;

extern crate alloc;

use ::core::ffi::CStr;
use core::{cell::{OnceCell, UnsafeCell}, mem::{self, MaybeUninit}};
use alloc::{boxed::Box, vec::Vec};
use bbt::BadBlockTable;
use bindings::{
    connector, lring_entry, nvm_mmgr_geometry, pipeline, ssd_os_ctrl_fn, ssd_os_stage_fn, stage, volt_get_geometry, MAGIC_CONNECTOR, MAGIC_STAGE
};
use my_alloc::SimpleAllocator;
use safe_bindings::{
    ssd_os_get_connection, ssd_os_mem_get, ssd_os_mem_size, ssd_os_print_lock, ssd_os_print_s,
    ssd_os_print_ss, ssd_os_print_unlock, ssd_os_sleep, ssd_os_this_cpu,
};
use shared::addresses::PhysicalBlockAddress;


pub struct Tester {
    pub elem: Vec<u32>,
}

impl Tester {
    pub fn new() -> Self {
        println_s!(c"Tester init!");
        Tester {
            elem: Vec::with_capacity(3),
        }
    }
}

impl Tester {
    pub fn push(&mut self, val: u32) {
        println_s!(c"Tester push!");

        self.elem.push(val);
    }
}

impl Drop for Tester {
    fn drop(&mut self) {
        println_s!(c"Tester dropped");
    }
}


#[inline(never)]
fn panic_printer(info: &core::panic::PanicInfo) {
    const BUFFER_SIZE: usize = 128;
    static mut buffer: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];

    if let Some(localtion) = info.location() {
        let file = localtion.file();
        let line = localtion.line();
        let col = localtion.column();
        for i in 0..(BUFFER_SIZE - 1).min(file.len()) {
            unsafe {
                buffer[i + 1] = 0;
                buffer[i] = file.as_bytes()[i];
            }
        }
        unsafe {
            println_s!(CStr::from_bytes_with_nul_unchecked(&buffer));
        }
        println_s!(c"line: ");
        println_i!(line);
        println_s!(c"column: ");
        println_i!(col);
    }
    if let Some(msg) = info.message().as_str() {
        for i in 0..(BUFFER_SIZE - 1).min(msg.len()) {
            unsafe {
                buffer[i + 1] = 0;
                buffer[i] = msg.as_bytes()[i];
            }
        }
        unsafe {
            println_s!(CStr::from_bytes_with_nul_unchecked(&buffer));
        }
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println_s!(c"PANIC!");
    panic_printer(info);
    loop {}
}

#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator::new();


static mut my_int: u64 = 0;
const hello: [u8; 32] = *b"hello world\0....................";

impl stage {
    const fn new(
        name: &[u8],
        init: ssd_os_ctrl_fn,
        exit: ssd_os_ctrl_fn,
        stage_fn: ssd_os_stage_fn,
    ) -> Self {
        stage {
            magic: *MAGIC_STAGE,
            name: {
                let mut buf = [0u8; 32];
                let mut i = 0;
                while i < name.len() {
                    buf[i] = name[i];
                    i += 1;
                }
                buf
            },
            init_fn: init,
            exit_fn: exit,
            stage_fn,
        }
    }
}

#[unsafe(no_mangle)]
pub static bbt_stage: stage = stage::new(
    b"bbt_stage",
    Some(s1_init),
    Some(s1_init),
    Some(bbt_stage_fn),
);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn s1_init() -> ::core::ffi::c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn s1_exit() -> ::core::ffi::c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_stage_fn(
    context: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    ssd_os_print_lock();
    ssd_os_print_ss(
        unsafe { CStr::from_ptr(context as *const u8) },
        c"BBT_STAGE\n",
    );
    ssd_os_print_unlock();
    context
}

#[unsafe(no_mangle)]
pub static bbt_conn: connector =
    connector::new(c"bbt_conn", bbt_init, bbt_exit, bbt_conn_fn, bbt_ring);

impl connector {
    const fn new(
        name: &CStr,
        init_fn: unsafe extern "C" fn() -> i32,
        exit_fn: unsafe extern "C" fn() -> i32,
        conn_fn: unsafe extern "C" fn(*mut lring_entry) -> *mut pipeline,
        ring_fn: unsafe extern "C" fn(*mut lring_entry) -> i32,
    ) -> Self {
        Self {
            magic: *MAGIC_CONNECTOR,
            name: {
                let mut buf = [0u8; 32];
                let s = name.to_bytes();
                let mut i = 0;
                while i < s.len() {
                    buf[i] = s[i];
                    i += 1;
                }
                buf
            },
            init_fn: Some(init_fn),
            exit_fn: Some(exit_fn),
            conn_fn: Some(conn_fn),
            ring_fn: Some(ring_fn),
        }
    }
    fn get_name(&self) -> &CStr {
        let Ok(s) = CStr::from_bytes_until_nul(&self.name) else {
            println_s!(c"ERROR!");
            return c"";
        };
        s
    }
}

static BBT : BadBlockTable = BadBlockTable::new();

// static mut GEO : nvm_mmgr_geometry = nvm_mmgr_geometry { n_of_ch: 10, lun_per_ch: 2, blk_per_lun: 2, pg_per_blk: 2, sec_per_pg: 2, n_of_planes: 2, pg_size: 2, sec_oob_sz: 2, sec_per_pl_pg: 2, sec_per_blk: 2, sec_per_lun: 2, sec_per_ch: 2, pg_per_lun: 2, pg_per_ch: 2, blk_per_ch: 2, tot_sec: 2, tot_pg: 2, tot_blk: 2, tot_lun: 2, sec_size: 2, pl_pg_size: 2, blk_size: 2, lun_size: 2, ch_size: 2, tot_size: 2, pg_oob_sz: 2, pl_pg_oob_sz: 2, blk_oob_sz: 2, lun_oob_sz: 2, ch_oob_sz: 2, tot_oob_sz: 2 };


#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_init() -> ::core::ffi::c_int {
    println_s!(c"init start:");
    let cpu_id = ssd_os_this_cpu(bbt_conn.get_name());
    let memory_region = ssd_os_mem_get(cpu_id) as usize;
    let memory_size = ssd_os_mem_size(cpu_id) as usize;
    println_s!(c"yo1:");
    let mut geo : nvm_mmgr_geometry = nvm_mmgr_geometry { n_of_ch: 10, lun_per_ch: 2, blk_per_lun: 2, pg_per_blk: 2, sec_per_pg: 2, n_of_planes: 2, pg_size: 2, sec_oob_sz: 2, sec_per_pl_pg: 2, sec_per_blk: 2, sec_per_lun: 2, sec_per_ch: 2, pg_per_lun: 2, pg_per_ch: 2, blk_per_ch: 2, tot_sec: 2, tot_pg: 2, tot_blk: 2, tot_lun: 2, sec_size: 2, pl_pg_size: 2, blk_size: 2, lun_size: 2, ch_size: 2, tot_size: 2, pg_oob_sz: 2, pl_pg_oob_sz: 2, blk_oob_sz: 2, lun_oob_sz: 2, ch_oob_sz: 2, tot_oob_sz: 2 };

    // let mut geo = MaybeUninit::<nvm_mmgr_geometry>::uninit();
    // static GEO : MaybeUninit::<nvm_mmgr_geometry> = MaybeUninit::<nvm_mmgr_geometry>::uninit();
    println_s!(c"yo2:");
    println_i!((&mut geo as *mut nvm_mmgr_geometry)as u32);
    unsafe { volt_get_geometry(&mut geo as *mut nvm_mmgr_geometry) };
    println_s!(c"yo3:");


    assert_eq!(
        (&ALLOCATOR as *const _ as usize) % core::mem::align_of::<usize>(),
        0
    );
    ALLOCATOR.initialize(memory_region, memory_region + memory_size);

    // panic!("info");
    //
    println_s!(c"yoyo:");
    
    BBT.init(&geo);


    println_s!(c"Channel len");
    println_i!(BBT.channels.borrow().len() as u32);

    ssd_os_sleep(10);

    let pba : PhysicalBlockAddress = PhysicalBlockAddress {
        channel: 0,
        lun: 0,
        plane: 0,
        block: 0,
    };

    let pba_bad_check : PhysicalBlockAddress = PhysicalBlockAddress {
        channel: 5,
        lun: 0,
        plane: 0,
        block: 0,
    };

    for i in 0..10 {
        let pba2 : PhysicalBlockAddress = PhysicalBlockAddress {
            channel: i,
            lun: 0,
            plane: 0,
            block: 0,
        };
        BBT.set_bad_block(&pba2);
    }


    println_s!(c"Bad block");
    println_i!(BBT.get_block_status(&pba) as u32);

    println_s!(c"Another bad block");
    println_i!(BBT.get_block_status(&pba_bad_check) as u32);

    // let mut test2: Vec<u32> = alloc::vec::Vec::new();

    // test2.push(33);
    //
    println_s!(c"Size of bbt");
    // println_i!(mem::size_of_val(&BBT) as u32);
    // println_s!(c"");



    let mut heap_val1: alloc::vec::Vec<u32> = alloc::vec::Vec::with_capacity(3);

    heap_val1.push(42);
    println_i!(heap_val1[0]);
    heap_val1[0] = 69;
    println_i!(heap_val1[0]);
    heap_val1.push(3);
    // heap_val1.push(3);
    // heap_val1.push(3);
    // println_i!(heap_val1[3]);

    let b1 = Box::new(41u32);
    let b2 = Box::new(42u32);
    let b3 = Box::new(43u32);
    println_i!(*b1);
    println_i!(*b2);
    println_i!(*b3);

    // ssd_os_sleep(10);

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_exit() -> ::core::ffi::c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_ring(ring: *mut lring_entry) -> ::core::ffi::c_int {
    ssd_os_print_lock();
    ssd_os_print_ss(
        unsafe { CStr::from_ptr(ring.as_ref().unwrap().ctx as *const u8) },
        c"END\n",
    );
    ssd_os_print_unlock();
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbt_conn_fn(entry: *mut lring_entry) -> *mut pipeline {
    let pipe = ssd_os_get_connection(
        c"bbt_conn".as_ptr().cast_mut(),
        c"bbt_pipe".as_ptr().cast_mut(),
    );
    if !pipe.is_null() {
        unsafe { entry.as_mut().unwrap() }.ctx = hello.as_ptr() as *mut ::core::ffi::c_void;
        ssd_os_print_ss(
            unsafe { CStr::from_ptr(entry.as_ref().unwrap().ctx as *const u8) },
            c"START\n",
        );
        println_s!(c"Accessing BBT from conn function");
        let pba_bad : PhysicalBlockAddress = PhysicalBlockAddress {
            channel: 0,
            lun: 0,
            plane: 0,
            block: 0,
        };

        let pba_good : PhysicalBlockAddress = PhysicalBlockAddress {
            channel: 0,
            lun: 0,
            plane: 1,
            block: 0,
        };
        println_s!(c"BAD: (SHOULD BE 0)");
        println_i!(BBT.get_block_status(&pba_bad) as u32);

        println_s!(c"GOD: (SHOULD BE 1)");
        println_i!(BBT.get_block_status(&pba_good) as u32);

        println_s!(c"MUTATING BAD BLOCK TABLE!!");
        BBT.set_bad_block(&pba_good);

        println_s!(c"SHOULD NOW BE SET TO BAD (0)");
        println_i!(BBT.get_block_status(&pba_good) as u32);



        ssd_os_sleep(10);
        return pipe;
    } else {
        return ::core::ptr::null_mut();
    }
}
