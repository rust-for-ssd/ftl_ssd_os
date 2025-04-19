use core::alloc::Allocator;
use ftl_ssd_os::bbt::bbt::BadBlockTable;
use ftl_ssd_os::bindings::generated::nvm_mmgr_geometry;
use ftl_ssd_os::provisioner::provisioner::Provisioner;
use ftl_ssd_os::{allocator::sdd_os_alloc::SimpleAllocator, bindings::safe::ssd_os_mem_get};
use riscv_rt::heap_start;
use semihosting::{print, println};

extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;

const GEOMETRY: nvm_mmgr_geometry = {
    let n_of_ch = 1;
    let lun_per_ch = 1;
    let blk_per_lun = 1;
    let pg_per_blk = 1;
    let sec_per_pg = 1;
    let n_of_planes = 1;
    let pg_size = 1;
    let sec_oob_sz = 1;

    nvm_mmgr_geometry {
        n_of_ch,
        lun_per_ch,
        blk_per_lun,
        pg_per_blk,
        sec_per_pg,
        n_of_planes,
        pg_size,
        sec_oob_sz,
        sec_per_pl_pg: (sec_per_pg as u32 / n_of_planes as u32) as u32,
        sec_per_blk: (blk_per_lun as u32 * pg_per_blk as u32 * sec_per_pg as u32),
        sec_per_lun: (blk_per_lun as u32 * pg_per_blk as u32 * sec_per_pg as u32),
        sec_per_ch: (lun_per_ch as u32
            * blk_per_lun as u32
            * pg_per_blk as u32
            * sec_per_pg as u32),
        pg_per_lun: (blk_per_lun as u32 * pg_per_blk as u32),
        pg_per_ch: (lun_per_ch as u32 * blk_per_lun as u32 * pg_per_blk as u32),
        blk_per_ch: (lun_per_ch as u32 * blk_per_lun as u32),
        tot_sec: (n_of_ch as u64
            * lun_per_ch as u64
            * blk_per_lun as u64
            * pg_per_blk as u64
            * sec_per_pg as u64),
        tot_pg: (n_of_ch as u64 * lun_per_ch as u64 * blk_per_lun as u64 * pg_per_blk as u64),
        tot_blk: (n_of_ch as u32 * lun_per_ch as u32 * blk_per_lun as u32),
        tot_lun: (n_of_ch as u32 * lun_per_ch as u32),
        sec_size: 4096,
        pl_pg_size: (pg_size / n_of_planes as u32),
        blk_size: (pg_per_blk as u32 * pg_size),
        lun_size: (blk_per_lun as u64 * pg_per_blk as u64 * pg_size as u64),
        ch_size: (lun_per_ch as u64 * blk_per_lun as u64 * pg_per_blk as u64 * pg_size as u64),
        tot_size: (n_of_ch as u64
            * lun_per_ch as u64
            * blk_per_lun as u64
            * pg_per_blk as u64
            * pg_size as u64),
        pg_oob_sz: (sec_per_pg as u32 * sec_oob_sz),
        pl_pg_oob_sz: ((sec_per_pg as u32 / n_of_planes as u32) as u32 * sec_oob_sz),
        blk_oob_sz: (pg_per_blk as u32 * sec_per_pg as u32 * sec_oob_sz),
        lun_oob_sz: (blk_per_lun as u32 * pg_per_blk as u32 * sec_per_pg as u32 * sec_oob_sz),
        ch_oob_sz: (lun_per_ch as u64
            * blk_per_lun as u64
            * pg_per_blk as u64
            * sec_per_pg as u64
            * sec_oob_sz as u64),
        tot_oob_sz: (n_of_ch as u64
            * lun_per_ch as u64
            * blk_per_lun as u64
            * pg_per_blk as u64
            * sec_per_pg as u64
            * sec_oob_sz as u64),
    }
};

#[test_case]
pub fn new() {
    static ALLOCATOR: SimpleAllocator = SimpleAllocator::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    ALLOCATOR.initialize(start, end);

    let bbt: BadBlockTable<SimpleAllocator> = BadBlockTable::new(&GEOMETRY, &ALLOCATOR);

    assert_eq!(bbt.channels.len(), GEOMETRY.n_of_ch as usize);
    assert_eq!(bbt.channels[0].luns.len(), GEOMETRY.lun_per_ch as usize);
    assert_eq!(
        bbt.channels[0].luns[0].planes.len(),
        GEOMETRY.n_of_planes as usize
    );
    assert_eq!(
        bbt.channels[0].luns[0].planes[0].blocks.len(),
        GEOMETRY.blk_per_lun as usize
    );
}
