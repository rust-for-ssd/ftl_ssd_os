use crate::allocator::linked_list_alloc::LinkedListAllocator;
use crate::l2p::l2p::L2pMapper;
use crate::shared::core_local_cell::CoreLocalCell;

fn alloc_init() -> &'static LinkedListAllocator {
    static ALLOCATOR: CoreLocalCell<LinkedListAllocator> = CoreLocalCell::new();
    let alloc = LinkedListAllocator::new();
    let start = riscv_rt::heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    alloc.initialize(start, end);
    ALLOCATOR.set(alloc);
    return ALLOCATOR.get();
}

#[test_case]
pub fn test_l2p_mapping() {
    let allocator = alloc_init();
    let mut mapper: L2pMapper<10_000, LinkedListAllocator> = L2pMapper::new(allocator);

    let physical_addr1 = 0x1234;
    let physical_addr2 = 0x5678;

    mapper.map(0x100, physical_addr1);
    mapper.map(0x200, physical_addr2);

    assert!(
        mapper.lookup(0x100).is_some(),
        "Logical address 0x100 should be mapped"
    );
    assert!(
        mapper.lookup(0x200).is_some(),
        "Logical address 0x200 should be mapped"
    );
}

#[test_case]
pub fn test_l2p_lookup() {
    let allocator = alloc_init();
    let mut mapper: L2pMapper<10_000, LinkedListAllocator> = L2pMapper::new(allocator);

    let physical_addr1 = 0x1234;
    let physical_addr2 = 0x5678;

    mapper.map(0x100, physical_addr1);
    mapper.map(0x200, physical_addr2);

    if let Some(ppa1) = mapper.lookup(0x100) {
        assert_eq!(ppa1, 0x1234);
    } else {
        panic!("Failed to find mapping for logical address 0x100");
    }

    if let Some(ppa2) = mapper.lookup(0x200) {
        assert_eq!(ppa2, 0x5678);
    } else {
        panic!("Failed to find mapping for logical address 0x200");
    }
}

#[test_case]
pub fn test_l2p_unmapping() {
    let allocator = alloc_init();
    let mut mapper: L2pMapper<10_000, LinkedListAllocator> = L2pMapper::new(allocator);

    let physical_addr1 = 0x1234;
    mapper.map(0x100, physical_addr1);

    let removed_ppa = mapper
        .unmap(0x100)
        .expect("Failed to unmap logical address 0x100");
    assert_eq!(removed_ppa, 0x1234);

    assert_eq!(mapper.lookup(0x100), None);
}

#[test_case]
pub fn test_l2p_clear() {
    let allocator = alloc_init();
    let mut mapper: L2pMapper<10_000, LinkedListAllocator> = L2pMapper::new(allocator);

    let physical_addr1 = 0x1234;
    let physical_addr2 = 0x5678;

    mapper.map(0x100, physical_addr1);
    mapper.map(0x200, physical_addr2);

    mapper.clear();
    assert!(mapper.is_empty());
    assert_eq!(mapper.len(), 0);
}
