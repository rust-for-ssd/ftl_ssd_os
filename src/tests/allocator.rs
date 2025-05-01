use crate::allocator::linked_list_alloc::LinkedListAllocator;
use crate::bindings::safe::ssd_os_mem_get;
use riscv_rt::heap_start;

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;

#[test_case]
pub fn we_can_allocate_two_boxes() {
    let allocator = LinkedListAllocator::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    allocator.initialize(start, end);
    let one: Box<u32, &LinkedListAllocator> = Box::new_in(1, &allocator);
    let two: Box<u32, &LinkedListAllocator> = Box::new_in(2, &allocator);

    let three = *one + *two;

    assert_eq!(3, three)
}

#[test_case]
pub fn we_can_mutate_boxes() {
    let allocator = LinkedListAllocator::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    allocator.initialize(start, end);

    let one: Box<u32, &LinkedListAllocator> = Box::new_in(1, &allocator);
    let mut two: Box<u32, &LinkedListAllocator> = Box::new_in(1, &allocator);
    *two = 2;

    let three = *one + *two;

    assert_eq!(3, three)
}

#[test_case]
pub fn boxes_gets_allocated_32bit_alligned() {
    let allocator = LinkedListAllocator::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    allocator.initialize(start, end);

    let one: Box<u32, &LinkedListAllocator> = Box::new_in(1, &allocator);
    let two: Box<u8, &LinkedListAllocator> = Box::new_in(1, &allocator);
    let three: Box<u8, &LinkedListAllocator> = Box::new_in(1, &allocator);
    let four: Box<u8, &LinkedListAllocator> = Box::new_in(1, &allocator);

    let base = start as u32;

    // Get integer addresses
    let addr_one = Box::as_ptr(&one) as u32;
    let addr_two = Box::as_ptr(&two) as u32;
    let addr_three = Box::as_ptr(&three) as u32;
    let addr_four = Box::as_ptr(&four) as u32;

    // Assert expected memory layout (e.g., 32-byte aligned)
    assert_eq!(addr_one, base);
    assert_eq!(addr_two, base + 8);
    assert_eq!(addr_three, base + 16);
    assert_eq!(addr_four, base + 24);
}

#[test_case]
pub fn we_can_allocate_structs() {
    let allocator = LinkedListAllocator::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    allocator.initialize(start, end);

    #[derive(Debug, PartialEq, Clone)]
    struct MyStruct<'a> {
        id: u32,
        name: &'a str,
    }

    let instance = MyStruct {
        id: 1234,
        name: "hi from struct",
    };

    let instance_clone = instance.clone();

    let one: Box<MyStruct, &LinkedListAllocator> = Box::new_in(instance, &allocator);

    assert_eq!(1234, one.id);
    assert_eq!("hi from struct", one.name);
    assert_eq!(instance_clone, *one);
}

#[test_case]
pub fn we_cannot_allocate_above_the_region() {
    let allocator = LinkedListAllocator::new();
    let start: *mut u8 = ssd_os_mem_get(0).cast();
    let end = unsafe { start.add(8) };
    allocator.initialize(start, end);

    let one: Box<u32, &LinkedListAllocator> = Box::new_in(1, &allocator);
    let should_fail = Box::try_new_in(1, &allocator);
    match should_fail {
        Ok(b) => {
            assert!(false);
        }
        Err(_) => {
            assert!(true);
        }
    }
}

#[test_case]
pub fn we_can_allocate_huge_things() {
    let allocator = LinkedListAllocator::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    allocator.initialize(start, end);

    const SIZE: usize = 1024 * 256;
    // Create a large vector using the custom allocator
    let mut vec: Vec<usize, &LinkedListAllocator> = Vec::with_capacity_in(SIZE, &allocator); // ~4 MiB

    for i in 0..SIZE {
        // usize (4) * 1024 * 1024 bytes ~4 MiB
        vec.push(i);
    }
    // let size_in_bytes = vec.len() * core::mem::size_of::<usize>();
    assert_eq!(vec.len(), SIZE)
}

#[test_case]
pub fn deallocation_works() {
    let allocator = LinkedListAllocator::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    allocator.initialize(start, end);

    // allocate a value
    let first_ptr = {
        let one: Box<u32, &LinkedListAllocator> = Box::new_in(42, &allocator);
        let ptr = Box::into_raw(one); // extract the raw pointer
        unsafe {
            drop(Box::from_raw_in(ptr, &allocator));
        }
        ptr
    };

    // allocate again, should reuse same memory if dealloc worked
    let two: Box<u32, &LinkedListAllocator> = Box::new_in(99, &allocator);
    let second_ptr = Box::into_raw(two);

    assert_eq!(
        first_ptr, second_ptr,
        "Allocator did not reuse deallocated memory"
    );
}

#[test_case]
pub fn coalescing_works() {
    let allocator = LinkedListAllocator::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    allocator.initialize(start, end);

    // Allocate adjacent blocks (e.g., 8 bytes each)
    let a: Box<u32, &LinkedListAllocator> = Box::new_in(1, &allocator);
    let b: Box<u32, &LinkedListAllocator> = Box::new_in(2, &allocator);
    let c: Box<u32, &LinkedListAllocator> = Box::new_in(3, &allocator);

    let a_ptr = Box::into_raw(a);
    let b_ptr = Box::into_raw(b);
    let c_ptr = Box::into_raw(c);

    // Ensure they're adjacent
    assert_eq!(unsafe { b_ptr.offset_from(a_ptr) }, 2); // two usize apart (1 for data, one for pointer)
    assert_eq!(unsafe { c_ptr.offset_from(b_ptr) }, 2); // two usize apart (1 for data, one for pointer)
    assert_eq!(unsafe { c_ptr.offset_from(a_ptr) }, 4); // two usize apart (1 for data, one for pointer)

    // Drop all (deallocate)
    unsafe {
        drop(Box::from_raw_in(a_ptr, &allocator));
        drop(Box::from_raw_in(b_ptr, &allocator));
        drop(Box::from_raw_in(c_ptr, &allocator));
    }

    // Now try allocating a larger block that would only fit if coalesced
    let large: Box<[u32; 10], &LinkedListAllocator> = Box::new_in([0; 10], &allocator);
    let large_ptr = Box::into_raw(large) as *mut u32;

    assert_eq!(
        large_ptr, a_ptr,
        "Coalescing failed: expected allocation at start of freed region"
    );
}

#[test_case]
pub fn coalescing_works_in_middle() {
    let allocator = LinkedListAllocator::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    allocator.initialize(start, end);

    // Allocate adjacent blocks (e.g., 8 bytes each)
    let a: Box<u32, &LinkedListAllocator> = Box::new_in(1, &allocator);
    let b: Box<u32, &LinkedListAllocator> = Box::new_in(2, &allocator); // goal: coalese this
    let c: Box<u32, &LinkedListAllocator> = Box::new_in(3, &allocator); // and this
    let d: Box<u32, &LinkedListAllocator> = Box::new_in(4, &allocator);

    let a_ptr = Box::into_raw(a);
    let b_ptr = Box::into_raw(b);
    let c_ptr = Box::into_raw(c);
    let d_ptr = Box::into_raw(d);

    // Ensure they're adjacent
    assert_eq!(unsafe { b_ptr.offset_from(a_ptr) }, 2); // two usize apart (1 for data, one for pointer)
    assert_eq!(unsafe { c_ptr.offset_from(b_ptr) }, 2); // two usize apart (1 for data, one for pointer)
    assert_eq!(unsafe { c_ptr.offset_from(a_ptr) }, 4); // two usize apart (1 for data, one for pointer)
    assert_eq!(unsafe { d_ptr.offset_from(a_ptr) }, 6); // two usize apart (1 for data, one for pointer)

    // Only drop two in the middle
    unsafe {
        drop(Box::from_raw_in(b_ptr, &allocator));
        drop(Box::from_raw_in(c_ptr, &allocator));
    }

    // Now try allocating a larger block that would only fit if coalesced
    let large: Box<[u32; 3], &LinkedListAllocator> = Box::new_in([0; 3], &allocator);
    let large_ptr = Box::into_raw(large) as *mut u32;

    assert_eq!(
        large_ptr, b_ptr,
        "Coalescing failed: expected allocation at start of freed region"
    );
}

#[test_case]
pub fn large_allocation_cannot_get_small_coalesed_block_in_middle() {
    let allocator = LinkedListAllocator::new();
    let start = heap_start() as *mut u8;
    let end = unsafe { start.add(&crate::_heap_size as *const u8 as usize) };
    allocator.initialize(start, end);

    // Allocate two adjacent blocks (e.g., 8 bytes each)
    let a: Box<u32, &LinkedListAllocator> = Box::new_in(1, &allocator);
    let b: Box<u32, &LinkedListAllocator> = Box::new_in(2, &allocator); // goal: coalese this
    let c: Box<u32, &LinkedListAllocator> = Box::new_in(3, &allocator); // and this
    let d: Box<u32, &LinkedListAllocator> = Box::new_in(4, &allocator);

    let a_ptr = Box::into_raw(a);
    let b_ptr = Box::into_raw(b);
    let c_ptr = Box::into_raw(c);
    let d_ptr = Box::into_raw(d);

    // Ensure they're adjacent
    assert_eq!(unsafe { b_ptr.offset_from(a_ptr) }, 2); // two usize apart (1 for data, one for pointer)
    assert_eq!(unsafe { c_ptr.offset_from(b_ptr) }, 2); // two usize apart (1 for data, one for pointer)
    assert_eq!(unsafe { c_ptr.offset_from(a_ptr) }, 4); // two usize apart (1 for data, one for pointer)
    assert_eq!(unsafe { d_ptr.offset_from(a_ptr) }, 6); // two usize apart (1 for data, one for pointer)

    // Only drop two in the middle
    unsafe {
        drop(Box::from_raw_in(b_ptr, &allocator));
        drop(Box::from_raw_in(c_ptr, &allocator));
    }

    // Now try allocating a larger block that doesnt fit in the coalesed space in the middle
    let large: Box<[u32; 100], &LinkedListAllocator> = Box::new_in([0; 100], &allocator);
    let large_ptr = Box::into_raw(large) as *mut u32;
    let should_be_here_ptr = unsafe { d_ptr.add(2) }; // 2x usize is the size of a free block (data + next pointer)

    assert_eq!(large_ptr, should_be_here_ptr);
}
