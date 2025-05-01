use crate::allocator::sdd_os_alloc::SimpleAllocator;
use crate::bbt::bbt::BadBlockTable;
use crate::media_manager::media_manager::Geometry;
use crate::shared::core_local_cell::CoreLocalCell;
use riscv_rt::heap_start;

extern crate alloc;

const GEOMETRY: Geometry = Geometry {
    n_pages: 100,
    n_of_ch: 8,
    n_of_planes: 2,
    lun_per_ch: 12,
    blk_per_lun: 16,
    pg_per_blk: 16,
};

#[test_case]
pub fn new() {
    static ALLOCATOR: CoreLocalCell<SimpleAllocator> = CoreLocalCell::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };

    ALLOCATOR.set(SimpleAllocator::new());
    ALLOCATOR.get().initialize(start, end);

    let bbt: BadBlockTable<SimpleAllocator> = BadBlockTable::new(&GEOMETRY, ALLOCATOR.get());

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
