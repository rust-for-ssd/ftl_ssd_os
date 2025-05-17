use ::core::ffi::c_void;
use core::{
    ffi::c_int,
    ptr::{null, null_mut},
};

use crate::{
    bindings::{
        generated::{
            TICKS_SEC, lring_entry, pipeline, ssd_os_lring_dequeue, ssd_os_sleep,
            ssd_os_timer_interrupt_on, ssd_os_usleep,
        },
        lring::LRing,
        mem::MemoryRegion,
        safe::ssd_os_get_connection,
    },
    make_connector_static, make_stage_static,
    shared::macros::{ensure_unique, println},
};

make_connector_static!(
    round_trip_conn1,
    conn1_init,
    conn1_exit,
    conn1_fn,
    conn1_ring_fn,
    0
);
make_connector_static!(
    round_trip_conn2,
    conn2_init,
    conn2_exit,
    conn2_fn,
    conn2_ring_fn,
    0
);

make_stage_static!(stage1_1, stage_init_fn, stage_exit_fn, stage1_1_fn);
make_stage_static!(stage1_2, stage_init_fn, stage_exit_fn, stage1_2_fn);
make_stage_static!(stage1_3, stage_init_fn, stage_exit_fn, stage1_3_fn);

make_stage_static!(stage2_1, stage_init_fn, stage_exit_fn, stage2_1_fn);
make_stage_static!(stage2_2, stage_init_fn, stage_exit_fn, stage2_2_fn);
make_stage_static!(stage2_3, stage_init_fn, stage_exit_fn, stage2_3_fn);

static mut AMOUNT: u32 = 0;
static mut COUNT: u32 = 0;
static mut SUBMITTED: u32 = 0;
static mut LAST_COUNT: u32 = 0;

static mut PIPE1: *mut pipeline = core::ptr::null_mut();
static mut PIPE2: *mut pipeline = core::ptr::null_mut();
static POOL_SIZE: usize = 10000;
static RING_SIZE: usize = 128;
static LRING: LRing<RING_SIZE> = LRing::new();

static mut MESSAGE_POOL: [Numbers; POOL_SIZE] = [Numbers::ZERO; POOL_SIZE];
static mut MSG_USAGE_BITMAP: [bool; POOL_SIZE] = [false; POOL_SIZE];

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Numbers {
    value: u8,
    add: u8,
    id: u16,
}

impl Numbers {
    const ZERO: Self = Numbers {
        value: 0,
        add: 0,
        id: 0,
    };

    fn default() -> Numbers {
        Numbers {
            value: 0,
            add: 0,
            id: 0,
        }
    }

    fn reset(&mut self) {
        self.value = 0;
        self.add = 0;
        self.id = 0;
    }
}

fn get_free_message_index() -> Option<usize> {
    unsafe {
        for i in 0..POOL_SIZE {
            if !MSG_USAGE_BITMAP[i] {
                MSG_USAGE_BITMAP[i] = true;
                return Some(i);
            }
        }
    }
    None
}

// Release a message back to the pool
fn release_message(index: usize) {
    if index < POOL_SIZE {
        unsafe {
            MSG_USAGE_BITMAP[index] = false;
            MESSAGE_POOL[index].reset();
        }
    }
}

// Helper to get a pointer to a message from the pool
fn get_message_ptr(index: usize) -> *mut Numbers {
    if index < POOL_SIZE {
        unsafe { &mut MESSAGE_POOL[index] as *mut Numbers }
    } else {
        null_mut()
    }
}

// Helper to get index from a pointer
fn get_index_from_ptr(ptr: *const Numbers) -> Option<usize> {
    if ptr.is_null() {
        return None;
    }

    unsafe {
        let base_addr = &MESSAGE_POOL[0] as *const Numbers;
        let offset = (ptr as usize - base_addr as usize) / core::mem::size_of::<Numbers>();

        if offset < POOL_SIZE {
            Some(offset)
        } else {
            None
        }
    }
}

fn timer_fn() {
    unsafe {
        let cur = COUNT;
        let diff = cur - LAST_COUNT;
        LAST_COUNT = cur;

        // println!("op/sec       : {:?}", diff);
        // println!("stages/sec   : {:?}", 6 * diff); // we have 6 stages
        println!("{:?}", 6 * diff); // for benchmark
        // println!("in the rings : {:?}", AMOUNT);
        // println!("total        : {:?}", COUNT);
        // println!("submitted    : {:?}", SUBMITTED);
    }
}

extern "C" fn timer_callback() {
    timer_fn();
}

// ------- Connection functions --------
fn conn1_init() -> c_int {
    unsafe { ssd_os_timer_interrupt_on(TICKS_SEC as i32, timer_callback as *mut c_void) };

    0
}

fn conn2_init() -> c_int {
    let mut mem_region = MemoryRegion::new_from_cpu(0);
    let Ok(()) = LRING.init(c"CONN2_LRING", mem_region.free_start, 0) else {
        panic!("LRING WAS ALREADY INITIALIZED!");
    };
    let ring = LRING.get_lring().unwrap();
    mem_region.reserve(ring.alloc_mem as usize);

    0
}

fn conn1_exit() -> c_int {
    0
}

fn conn2_exit() -> c_int {
    0
}

fn conn1_fn(entry: *mut lring_entry) -> *mut pipeline {
    unsafe {
        // Only allocate a new message if we're below capacity
        if AMOUNT < RING_SIZE as u32 {
            if let Some(idx) = get_free_message_index() {
                let msg_ptr = get_message_ptr(idx);

                (*msg_ptr).value = 1;
                (*msg_ptr).add = 1;
                (*msg_ptr).id = SUBMITTED as u16;
                SUBMITTED += 1;

                (*entry).ctx = msg_ptr as *mut c_void;

                AMOUNT += 1;
            }
        }

        if PIPE1.is_null() {
            PIPE1 = ssd_os_get_connection(c"round_trip_conn1", c"round_trip_pipe1");
        }

        PIPE1
    }
}

fn conn2_fn(entry: *mut lring_entry) -> *mut pipeline {
    ensure_unique!();
    let _ = LRING.dequeue(entry);

    unsafe {
        if PIPE2.is_null() {
            PIPE2 = ssd_os_get_connection(c"round_trip_conn2", c"round_trip_pipe2");
        }
        return PIPE2;
    }
}

fn conn1_ring_fn(entry: *mut lring_entry) -> c_int {
    let Some(entry_ref) = lring_entry::new(entry) else {
        println!("NULL PTR!");
        return 1;
    };

    let n = match entry_ref.get_ctx_as_mut::<Numbers>() {
        Some(existing) => existing,
        None => {
            println!("Failed to get context as Numbers");
            return 1;
        }
    };

    unsafe {
        COUNT += 1;
        AMOUNT -= 1;
    }

    if n.value != 7 {
        println!("conn1_ring: Value is wrong: {:?}", n.value);
        println!("ID: {:?}", n.id);
    }

    // Release the message back to the pool
    if let Some(idx) = get_index_from_ptr(n) {
        release_message(idx);
    }

    0
}

fn conn2_ring_fn(entry: *mut lring_entry) -> c_int {
    ensure_unique!();

    match LRING.enqueue(entry) {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

// ------- Stage functions --------

fn stage_init_fn() -> c_int {
    ensure_unique!();
    0
}

fn stage_exit_fn() -> c_int {
    ensure_unique!();
    0
}

// -- Helper ---
fn add_fn(ctx: *mut c_void) -> *mut c_void {
    if !ctx.is_null() {
        let n = ctx as *mut Numbers;
        unsafe {
            (*n).value = (*n).value.wrapping_add((*n).add);
        }
    }
    ctx
}

fn stage1_1_fn(context: *mut c_void) -> *mut c_void {
    ensure_unique!();

    if context.is_null() {
        return context;
    }

    add_fn(context)
}

fn stage1_2_fn(context: *mut c_void) -> *mut c_void {
    ensure_unique!();

    if context.is_null() {
        return context;
    }

    add_fn(context)
}

fn stage1_3_fn(context: *mut c_void) -> *mut c_void {
    ensure_unique!();
    if context.is_null() {
        return context;
    }

    add_fn(context)
}

fn stage2_1_fn(context: *mut c_void) -> *mut c_void {
    ensure_unique!();

    if context.is_null() {
        return context;
    }

    add_fn(context)
}

fn stage2_2_fn(context: *mut c_void) -> *mut c_void {
    ensure_unique!();

    if context.is_null() {
        return context;
    }
    add_fn(context)
}

fn stage2_3_fn(context: *mut c_void) -> *mut c_void {
    ensure_unique!();

    if context.is_null() {
        return context;
    }
    add_fn(context)
}
