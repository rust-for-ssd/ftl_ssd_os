use semihosting::println;

use crate::allocator::sdd_os_alloc::SimpleAllocator;
use crate::bindings::generated::nvm_mmgr_geometry;
use crate::media_manager::media_manager::Geometry;
use crate::provisioner::provisioner::{Block, BlockWithPageInfo, ProvisionError, Provisioner};
use crate::shared::addresses::{PhysicalBlockAddress, PhysicalPageAddress};
use crate::shared::core_local_cell::CoreLocalCell;

// const GEOMETRY: nvm_mmgr_geometry = {
//     let n_of_ch = 8;
//     let lun_per_ch = 12;
//     let blk_per_lun = 16;
//     let pg_per_blk = 16;
//     let sec_per_pg = 1;
//     let n_of_planes = 1;
//     let pg_size = 1;
//     let sec_oob_sz = 1;

//     nvm_mmgr_geometry {
//         n_of_ch,
//         lun_per_ch,
//         blk_per_lun,
//         pg_per_blk,
//         sec_per_pg,
//         n_of_planes,
//         pg_size,
//         sec_oob_sz,
//         sec_per_pl_pg: (sec_per_pg as u32 / n_of_planes as u32) as u32,
//         sec_per_blk: (blk_per_lun as u32 * pg_per_blk as u32 * sec_per_pg as u32),
//         sec_per_lun: (blk_per_lun as u32 * pg_per_blk as u32 * sec_per_pg as u32),
//         sec_per_ch: (lun_per_ch as u32
//             * blk_per_lun as u32
//             * pg_per_blk as u32
//             * sec_per_pg as u32),
//         pg_per_lun: (blk_per_lun as u32 * pg_per_blk as u32),
//         pg_per_ch: (lun_per_ch as u32 * blk_per_lun as u32 * pg_per_blk as u32),
//         blk_per_ch: (lun_per_ch as u32 * blk_per_lun as u32),
//         tot_sec: (n_of_ch as u64
//             * lun_per_ch as u64
//             * blk_per_lun as u64
//             * pg_per_blk as u64
//             * sec_per_pg as u64),
//         tot_pg: (n_of_ch as u64 * lun_per_ch as u64 * blk_per_lun as u64 * pg_per_blk as u64),
//         tot_blk: (n_of_ch as u32 * lun_per_ch as u32 * blk_per_lun as u32),
//         tot_lun: (n_of_ch as u32 * lun_per_ch as u32),
//         sec_size: 4096,
//         pl_pg_size: (pg_size / n_of_planes as u32),
//         blk_size: (pg_per_blk as u32 * pg_size),
//         lun_size: (blk_per_lun as u64 * pg_per_blk as u64 * pg_size as u64),
//         ch_size: (lun_per_ch as u64 * blk_per_lun as u64 * pg_per_blk as u64 * pg_size as u64),
//         tot_size: (n_of_ch as u64
//             * lun_per_ch as u64
//             * blk_per_lun as u64
//             * pg_per_blk as u64
//             * pg_size as u64),
//         pg_oob_sz: (sec_per_pg as u32 * sec_oob_sz),
//         pl_pg_oob_sz: ((sec_per_pg as u32 / n_of_planes as u32) as u32 * sec_oob_sz),
//         blk_oob_sz: (pg_per_blk as u32 * sec_per_pg as u32 * sec_oob_sz),
//         lun_oob_sz: (blk_per_lun as u32 * pg_per_blk as u32 * sec_per_pg as u32 * sec_oob_sz),
//         ch_oob_sz: (lun_per_ch as u64
//             * blk_per_lun as u64
//             * pg_per_blk as u64
//             * sec_per_pg as u64
//             * sec_oob_sz as u64),
//         tot_oob_sz: (n_of_ch as u64
//             * lun_per_ch as u64
//             * blk_per_lun as u64
//             * pg_per_blk as u64
//             * sec_per_pg as u64
//             * sec_oob_sz as u64),
//     }
// };

const GEOMETRY: Geometry = Geometry {
    n_pages: 100,
    n_of_ch: 8,
    n_of_planes: 2,
    lun_per_ch: 12,
    blk_per_lun: 16,
    pg_per_blk: 16,
};

fn alloc_init() -> &'static SimpleAllocator {
    static ALLOCATOR: CoreLocalCell<SimpleAllocator> = CoreLocalCell::new();
    let alloc = SimpleAllocator::new();
    let start = riscv_rt::heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    alloc.initialize(start, end);
    ALLOCATOR.set(alloc);
    return ALLOCATOR.get();
}

#[test_case]
fn init() {
    let allocator = alloc_init();
    let prov: Provisioner<SimpleAllocator> = Provisioner::new(&GEOMETRY, &allocator);

    assert_eq!(prov.channels.len(), GEOMETRY.n_of_ch as usize);
    assert_eq!(prov.channels[0].luns.len(), GEOMETRY.lun_per_ch as usize);
    assert_eq!(
        prov.channels[0].luns[0].free.capacity(),
        GEOMETRY.blk_per_lun as usize
    );
    assert_eq!(prov.channels[0].luns[0].free.len(), 0);
}

#[test_case]
pub fn provision_block() {
    let allocator = alloc_init();

    let mut prov: Provisioner<SimpleAllocator> = Provisioner::new(&GEOMETRY, &allocator);

    // No free blocks when creating new
    let res = prov.provision_block();
    assert_eq!(res, Err(ProvisionError::NoFreeBlock));

    let block = Block { id: 3, plane_id: 0 };
    prov.channels[0].luns[0].free.push_front(block);

    let size = prov.channels[0].luns[0].free.len();
    assert_eq!(size, 1);

    let res = prov.provision_block();
    assert_eq!(
        res,
        Ok(PhysicalBlockAddress {
            channel: 0,
            lun: 0,
            plane: 0,
            block: 3
        })
    );

    let size = prov.channels[0].luns[0].free.len();
    assert_eq!(size, 0);

    let res = prov.provision_block();
    assert_eq!(res, Err(ProvisionError::NoFreeBlock));
}

#[test_case]
pub fn provision_page() {
    let allocator = alloc_init();

    let mut prov: Provisioner<SimpleAllocator> = Provisioner::new(&GEOMETRY, &allocator);

    // No free blocks, meaning no free pages when creating new
    let res = prov.provision_page();
    assert_eq!(res, Err(ProvisionError::NoFreePage));

    let block = Block { id: 3, plane_id: 0 };
    prov.channels[0].luns[0].free.push_back(block);
    let size = prov.channels[0].luns[0].free.len();
    assert_eq!(size, 1);

    let res = prov.provision_page();
    assert_eq!(
        res,
        Ok(PhysicalPageAddress {
            channel: 0,
            lun: 0,
            plane: 0,
            block: 3,
            page: 0
        })
    );
    let size = prov.channels[0].luns[0].free.len();
    assert_eq!(size, 0);
    let size = prov.channels[0].luns[0].partially_used.len();
    assert_eq!(size, 1);
    let size = prov.channels[0].luns[0].used.len();
    assert_eq!(size, 0);

    for i in 1..GEOMETRY.pg_per_blk {
        let size = prov.channels[0].luns[0].partially_used.len();
        assert_eq!(size, 1);
        let res = prov.provision_page();
        assert_eq!(
            res,
            Ok(PhysicalPageAddress {
                channel: 0,
                lun: 0,
                plane: 0,
                block: 3,
                page: i as u64
            })
        );
    }
    let size = prov.channels[0].luns[0].partially_used.len();
    assert_eq!(size, 0);
    let size = prov.channels[0].luns[0].used.len();
    assert_eq!(size, 1);
    let res = prov.provision_page();
    assert_eq!(res, Err(ProvisionError::NoFreePage));
}

#[test_case]
pub fn provision_page_with_partially_used_blocks() {
    let allocator = alloc_init();

    let mut prov: Provisioner<SimpleAllocator> = Provisioner::new(&GEOMETRY, &allocator);

    // No free blocks, meaning no free pages when creating new
    let res = prov.provision_page();
    assert_eq!(res, Err(ProvisionError::NoFreePage));

    let block = BlockWithPageInfo {
        id: 3,
        plane_id: 0,
        pages_reserved: 0,
    };
    prov.channels[0].luns[0].partially_used.push_back(block);
    let size = prov.channels[0].luns[0].free.len();
    assert_eq!(size, 0);
    let size = prov.channels[0].luns[0].partially_used.len();
    assert_eq!(size, 1);

    let res = prov.provision_page();
    assert_eq!(
        res,
        Ok(PhysicalPageAddress {
            channel: 0,
            lun: 0,
            plane: 0,
            block: 3,
            page: 0
        })
    );
    let size = prov.channels[0].luns[0].free.len();
    assert_eq!(size, 0);
    let size = prov.channels[0].luns[0].partially_used.len();
    assert_eq!(size, 1);
    let size = prov.channels[0].luns[0].used.len();
    assert_eq!(size, 0);

    for i in 1..GEOMETRY.pg_per_blk {
        let size = prov.channels[0].luns[0].partially_used.len();
        assert_eq!(size, 1);
        let res = prov.provision_page();
        assert_eq!(
            res,
            Ok(PhysicalPageAddress {
                channel: 0,
                lun: 0,
                plane: 0,
                block: 3,
                page: i as u64
            })
        );
    }
    let size = prov.channels[0].luns[0].partially_used.len();
    assert_eq!(size, 0);
    println!("{:?}", prov.channels[0].luns[0]);
    let size = prov.channels[0].luns[0].used.len();
    assert_eq!(size, 1);
    let res = prov.provision_page();
    assert_eq!(res, Err(ProvisionError::NoFreePage));
}

#[test_case]
pub fn provision_block_from_different_channels() {
    let allocator = alloc_init();

    let mut prov: Provisioner<SimpleAllocator> = Provisioner::new(&GEOMETRY, &allocator);

    // No free blocks when creating new
    let res = prov.provision_block();
    assert_eq!(res, Err(ProvisionError::NoFreeBlock));

    let block = Block { id: 3, plane_id: 0 };
    prov.channels[0].luns[0].free.push_back(block);
    prov.channels[2].luns[3].free.push_back(block);

    let size = prov.channels[0].luns[0].free.len();
    assert_eq!(size, 1);

    let res = prov.provision_block();
    assert!(res.is_ok());

    let res = prov.provision_block();
    assert!(res.is_ok());

    let size = prov.channels[0].luns[0].free.len();
    assert_eq!(size, 0);

    let res = prov.provision_block();
    assert_eq!(res, Err(ProvisionError::NoFreeBlock));
}

#[test_case]
pub fn push_free_block() {
    let allocator = alloc_init();

    let mut prov: Provisioner<SimpleAllocator> = Provisioner::new(&GEOMETRY, &allocator);

    // No free blocks when creating new
    let res = prov.provision_block();
    assert_eq!(res, Err(ProvisionError::NoFreeBlock));

    let pba = PhysicalBlockAddress {
        channel: 0,
        lun: 2,
        plane: 0,
        block: 3,
    };
    prov.push_free_block(&pba);

    let size = prov.channels[0].luns[2].free.len();
    assert_eq!(size, 1);

    let res = prov.provision_block();
    assert!(res.is_ok());

    let size = prov.channels[0].luns[2].free.len();
    assert_eq!(size, 0);

    let res = prov.provision_block();
    assert_eq!(res, Err(ProvisionError::NoFreeBlock));
}

#[test_case]
pub fn multiple_push_free_block() {
    let allocator = alloc_init();

    let mut prov: Provisioner<SimpleAllocator> = Provisioner::new(&GEOMETRY, &allocator);

    // No free blocks when creating new
    let res = prov.provision_block();
    assert_eq!(res, Err(ProvisionError::NoFreeBlock));

    let pba = PhysicalBlockAddress {
        channel: 0,
        lun: 2,
        plane: 0,
        block: 3,
    };
    prov.push_free_block(&pba);

    let size = prov.channels[0].luns[2].free.len();
    assert_eq!(size, 1);

    let pba = PhysicalBlockAddress {
        channel: 2,
        lun: 2,
        plane: 0,
        block: 3,
    };
    prov.push_free_block(&pba);

    let size = prov.channels[2].luns[2].free.len();
    assert_eq!(size, 1);

    let res = prov.provision_block();
    assert!(res.is_ok());

    let res = prov.provision_block();
    assert!(res.is_ok());

    let res = prov.provision_block();
    assert_eq!(res, Err(ProvisionError::NoFreeBlock));
}
