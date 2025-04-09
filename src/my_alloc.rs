use core::alloc::{GlobalAlloc, Layout};
use core::cell::{Cell, OnceCell};
use core::mem;

use crate::println_s;
use crate::safe_bindings::ssd_os_sleep;

// ISSUES: Dealloc does not work.
// Dynamic allocator

pub struct SimpleAllocator {
    start: OnceCell<usize>,
    end: OnceCell<usize>,
    free_list_head: Cell<*mut FreeBlock>,
}

struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

unsafe impl Send for SimpleAllocator {}
unsafe impl Sync for SimpleAllocator {}

impl SimpleAllocator {
    pub const fn new() -> Self {
        Self {
            start: OnceCell::new(),
            end: OnceCell::new(),
            free_list_head: Cell::new(core::ptr::null_mut()),
        }
    }

    /// Initializes the allocator by setting up the initial free block.
    pub fn initialize(&self, start: usize, end: usize) {
        if self.start.get().is_some() {
            return;
        }

        let _ = self.start.set(start);
        let _ = self.end.set(end);

        let size = end - start;

        // Only initialize if the region has enough space for a block
        if size >= mem::size_of::<FreeBlock>() {
            let block_ptr = start as *mut FreeBlock;

            // SAFETY: We're writing to memory we own
            unsafe {
                (*block_ptr).size = size;
                (*block_ptr).next = core::ptr::null_mut();
            }
            self.free_list_head.set(block_ptr);
        }
    }
}

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(mem::size_of::<FreeBlock>());
        let align = layout.align().max(mem::align_of::<FreeBlock>());

        let mut current_ptr = self.free_list_head.get();
        let mut prev_ptr: *mut FreeBlock = core::ptr::null_mut();

        while !current_ptr.is_null() {
            let current_block = unsafe { &*current_ptr };
            let block_addr = current_ptr as usize;

            // Calculate aligned address
            let align_offset = (align - (block_addr % align)) % align;
            let aligned_addr = block_addr + align_offset;

            // Check if the block is large enough
            if aligned_addr + size <= block_addr + current_block.size {
                // Remove or adjust the free block
                let remaining_size = (block_addr + current_block.size) - (aligned_addr + size);

                if remaining_size >= mem::size_of::<FreeBlock>() {
                    // Create a new free block with the remaining space
                    let new_block_ptr = (aligned_addr + size) as *mut FreeBlock;

                    unsafe {
                        (*new_block_ptr).size = remaining_size;
                        (*new_block_ptr).next = current_block.next;
                    }

                    if prev_ptr.is_null() {
                        self.free_list_head.set(new_block_ptr);
                    } else {
                        unsafe {
                            (*prev_ptr).next = new_block_ptr;
                        }
                    }
                } else {
                    // Use the entire block
                    if prev_ptr.is_null() {
                        self.free_list_head.set(current_block.next);
                    } else {
                        unsafe {
                            (*prev_ptr).next = current_block.next;
                        }
                    }
                }

                return aligned_addr as *mut u8;
            }

            prev_ptr = current_ptr;
            current_ptr = current_block.next;
        }

        // No suitable block found
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ssd_os_sleep(3);
        println_s!(c"DEALLOC!");
        return;
        println_s!(c"DEALLOC!");
        return;
        let Some(start) = self.start.get() else {
            return;
        };

        let end = self.end.get().unwrap();

        if (ptr as usize) < *start {
            return;
        } else if (ptr as usize) >= *end {
            return;
        }

        let ptr_addr = ptr as usize;

        // Ensure the pointer is within our memory region
        return;
        let b = ptr_addr < *self.start.get().unwrap();
        let a = ptr_addr >= *self.end.get().unwrap();
        if b || a {
            return;
            // panic!("Attempted to free memory not managed by this allocator");
        }

        return;
        let size = layout.size().max(mem::size_of::<FreeBlock>());

        let _adjusted_ptr = if layout.align() > mem::align_of::<FreeBlock>() {
            // Ensure pointer is aligned for both the layout and FreeBlock
            align_up(ptr as usize, layout.align()) as *mut u8
        } else {
            ptr
        };
        // let block_ptr = adjusted_ptr as *mut FreeBlock;

        let block_ptr = ptr as *mut FreeBlock;

        unsafe {
            (*block_ptr).size = size;
        }

        // Insert the block at the beginning of the free list
        loop {
            let head = self.free_list_head.get();
            unsafe {
                (*block_ptr).next = head;
            }

            if head == block_ptr {
                self.free_list_head.set(block_ptr);
                break;
            };
        }

        // Note: A production allocator would merge adjacent free blocks here
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    // Check if align is a power of 2
    debug_assert!(align.is_power_of_two());
    // Calculate the alignment mask
    let mask = align - 1;
    // Align the address upward
    (addr + mask) & !mask
}
