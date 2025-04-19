use crate::allocator::sdd_os_alloc::SimpleAllocator;
use crate::l2p::l2p::L2pMapper;
use crate::shared::core_local_cell::CoreLocalCell;



fn alloc_init() -> &'static SimpleAllocator {
    static ALLOCATOR: CoreLocalCell<SimpleAllocator> = CoreLocalCell::new();
    let alloc = SimpleAllocator::new();
    let start = riscv_rt::heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    alloc.initialize(start, end);
    ALLOCATOR.set(alloc);
    return ALLOCATOR.get();
}

// Test case for the simple L2pMapper with custom allocator
#[test_case]
pub fn test_l2p_mapping() {
    // Initialize the allocator
    let allocator = alloc_init();
    
    // Create a new mapper with the custom allocator
    let mut mapper = L2pMapper::new(allocator);
    
    // Create some test physical addresses
    let physical_addr1 = 0x1234;
    let physical_addr2 = 0x5678;
    
    // Map logical addresses to physical addresses
    mapper.map(0x100, physical_addr1);
    mapper.map(0x200, physical_addr2);
    
    // Test lookup functionality
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
    
    // Test unmapping
    let removed_ppa = mapper.unmap(0x100).expect("Failed to unmap logical address 0x100");
    assert_eq!(removed_ppa, 0x1234);
    
    // Verify the mapping was removed
    assert_eq!(mapper.lookup(0x100), None);
    
    // Verify other mappings still exist
    assert!(mapper.is_mapped(0x200));
    
    // Test length functionality
    assert_eq!(mapper.len(), 1);
    
    // Test clear functionality
    mapper.clear();
    assert!(mapper.is_empty());
    assert_eq!(mapper.len(), 0);
}