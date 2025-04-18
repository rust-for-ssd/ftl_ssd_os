use core::alloc::Allocator;
use ftl_ssd_os::{allocator::sdd_os_alloc::SimpleAllocator, bindings::safe::ssd_os_mem_get};
use semihosting::{print, println};

extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

// #[test_case]
// pub fn we_can_allocate_two_boxes() {
//     let allocator = SimpleAllocator::new();
//     let start = 0x80000000 as *mut u8;    
//     let end = unsafe { start.add(10000) };
//     allocator.initialize(start, end);

//     let one: Box<u32, &SimpleAllocator> = Box::new_in(1, &allocator);
//     let two: Box<u32, &SimpleAllocator> = Box::new_in(2, &allocator);

//     let three = *one + *two;

//     assert_eq!(3, three)
// }

// #[test_case]
// pub fn we_can_mutate_boxes() {
//     let allocator = SimpleAllocator::new();
//     let start = 0x80000000 as *mut u8;
//     let end = unsafe { start.add(10000) };
//     allocator.initialize(start, end);

//     let one: Box<u32, &SimpleAllocator> = Box::new_in(1, &allocator);
//     let mut two: Box<u32, &SimpleAllocator> = Box::new_in(1, &allocator);
//     *two = 2;

//     let three = *one + *two;

//     assert_eq!(3, three)
// }

// #[test_case]
// pub fn boxes_gets_allocated_32bit_alligned() {
//     let allocator = SimpleAllocator::new();
//     let start = 0x80000000 as *mut u8;
//     let end = unsafe { start.add(10000) };
//     allocator.initialize(start, end);

//     let one: Box<u32, &SimpleAllocator> = Box::new_in(1, &allocator);
//     let two: Box<u8, &SimpleAllocator> = Box::new_in(1, &allocator);
//     let three: Box<u8, &SimpleAllocator> = Box::new_in(1, &allocator);
//     let four: Box<u8, &SimpleAllocator> = Box::new_in(1, &allocator);

//     let base = 0x80000000 as u32;

//     // Get integer addresses
//     let addr_one = Box::as_ptr(&one) as u32;
//     let addr_two = Box::as_ptr(&two) as u32;
//     let addr_three = Box::as_ptr(&three) as u32;
//     let addr_four = Box::as_ptr(&four) as u32;

//     // Assert expected memory layout (e.g., 32-byte aligned)
//     assert_eq!(addr_one, base);
//     assert_eq!(addr_two, base + 8);
//     assert_eq!(addr_three, base + 16);
//     assert_eq!(addr_four, base + 24);
// }

// #[test_case]
// pub fn we_can_allocate_structs() {
//     let allocator = SimpleAllocator::new();
//     let start = 0x80000000 as *mut u8;
//     let end = unsafe { start.add(10000) };
//     allocator.initialize(start, end);

    
//     #[derive(Debug, PartialEq, Clone)]
//     struct MyStruct<'a> {
//         id: u32,
//         name:  &'a str,
//     }

//     let instance = MyStruct {
//         id: 1234,
//         name: "hi from struct",
//     };
    
//     let instance_clone = instance.clone();
    
//     let one: Box<MyStruct, &SimpleAllocator> = Box::new_in(instance, &allocator);
    
//     assert_eq!(1234, one.id);
//     assert_eq!("hi from struct", one.name);
//     assert_eq!(instance_clone, *one);
// }

// #[test_case]
// pub fn we_cannot_allocate_above_the_region() {
//     let allocator = SimpleAllocator::new();
//     let start = 0x80000000 as *mut u8;
//     let end = unsafe { start.add(8) };
//     allocator.initialize(start, end);

//     let one: Box<u32, &SimpleAllocator> = Box::new_in(1, &allocator);
//     let should_fail = Box::try_new_in(1, &allocator);
//     match should_fail {
//         Ok(b) => { assert!(false); }
//         Err(_) => { assert!(true); }
//     }
// }

#[test_case]
pub fn we_can_allocate_huge_things() {
    let allocator = SimpleAllocator::new();
    let start = 0x80000000 as *mut u8;
    let end = unsafe { start.add(1024 * 1024 * 100) }; // 100 MB
    allocator.initialize(start, end);

    // Create a large vector using the custom allocator
    let mut vec: Vec<usize, &SimpleAllocator> = Vec::with_capacity_in(1024 * 1024 * 20, &allocator); // ~40 MiB

    for i in 0..(1024 * 1024 * 20) { // usize (4) * 1024 * 1024 * 5 bytes // 20MB
        vec.push(i); 
    }
    
    let size_in_bytes = vec.len() * core::mem::size_of::<usize>();
    
    println!("Vector size: {:?}", size_in_bytes);

    for i in (0..vec.len()).rev() {
        assert_eq!(i, vec.pop().unwrap());
    }
}


// #[test_case]
// pub fn deallocation_works() {
//     let allocator = SimpleAllocator::new();
//     let start = 0x80000000 as *mut u8;
//     let end = unsafe { start.add(8) };
//     allocator.initialize(start, end);

//     // allocate a value
//     let first_ptr = {
//         let one: Box<u32, &SimpleAllocator> = Box::new_in(42, &allocator);
//         let ptr = Box::into_raw(one); // extract the raw pointer
//         unsafe {
//             // SAFETY: we still own the memory, we immediately reconstruct and drop it
//             drop(Box::from_raw_in(ptr, &allocator));
//         }
//         ptr
//     };

//     // allocate again, should reuse same memory if dealloc worked
//     let two: Box<u32, &SimpleAllocator> = Box::new_in(99, &allocator);
//     let second_ptr = Box::into_raw(two);

//     assert_eq!(first_ptr, second_ptr, "Allocator did not reuse deallocated memory");
// }

// #[test_case]
// pub fn coalescing_works() {
//     let allocator = SimpleAllocator::new();
//     let start = 0x80000000 as *mut u8;
//     let end = unsafe { start.add(32) };
//     allocator.initialize(start, end);

//     // Allocate two adjacent blocks (e.g., 8 bytes each)
//     let a: Box<u32, &SimpleAllocator> = Box::new_in(1, &allocator);
//     let b: Box<u32, &SimpleAllocator> = Box::new_in(2, &allocator);

//     let a_ptr = Box::into_raw(a);
//     let b_ptr = Box::into_raw(b);

//     // Ensure they're adjacent
//     assert_eq!(unsafe { b_ptr.offset_from(a_ptr) }, 1);

//     // Drop both (deallocate)
//     unsafe {
//         drop(Box::from_raw_in(a_ptr, &allocator));
//         drop(Box::from_raw_in(b_ptr, &allocator));
//     }

//     // Now try allocating a larger block that would only fit if coalesced
//     let large: Box<[u32; 3], &SimpleAllocator> = Box::new_in([0; 3], &allocator);
//     let large_ptr = Box::into_raw(large) as *mut u32;

//     assert_eq!(
//         large_ptr, a_ptr,
//         "Coalescing failed: expected allocation at start of freed region"
//     );
// }

