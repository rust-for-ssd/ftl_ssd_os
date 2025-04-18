use core::alloc::{AllocError, Allocator, Layout};
use core::cell::{Cell, OnceCell};
use core::ptr::NonNull;
use core::{mem, ptr};

use crate::bindings::safe::ssd_os_mem_get;
// ISSUES: Dealloc does not work.
// Dynamic allocator

pub struct SimpleAllocator {
    start: OnceCell<*mut u8>,
    end: OnceCell<*mut u8>,
    free_list_head: Cell<*mut FreeBlock>,
}

struct FreeBlock {         // Allocates at least 8 bytes for any sizes
    size: usize,           // 4 bytes in 32 bit systems
    next: *mut FreeBlock,  // 4 bytes in 32 bit systems 
}

unsafe impl Send for SimpleAllocator {}
unsafe impl Sync for SimpleAllocator {}

impl SimpleAllocator {
    pub const fn new() -> Self {
        Self {
            start: OnceCell::new(),
            end: OnceCell::new(),
            free_list_head: Cell::new(ptr::null_mut()),
        }
    }

    /// Initializes the allocator by setting up the initial free block.
    pub fn initialize(&self, start: *mut u8, end: *mut u8) {
        let lowest_addr = ssd_os_mem_get(0);
        assert!(lowest_addr <= start.cast());
        let Ok(()) = self.start.set(start) else {
            return;
        };
        let Ok(()) = self.end.set(end) else {
            return;
        };

        let size = end.addr() - start.addr();

        // Only initialize if the region has enough space for a block
        if size >= mem::size_of::<FreeBlock>() {
            let block_ptr = *self.start.get().unwrap() as *mut FreeBlock;

            // SAFETY: We're writing to memory we own
            unsafe {
                (*block_ptr).size = size;
                (*block_ptr).next = core::ptr::null_mut();
            }
            self.free_list_head.set(block_ptr);
        }
    }
}

unsafe impl Allocator for SimpleAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
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

                let ptr: NonNull<[u8]> = unsafe {
                    core::slice::from_raw_parts_mut(aligned_addr as *mut u8, size).into()
                };
                let lowest_addr = ssd_os_mem_get(0);
                assert!(lowest_addr <= ptr.as_ptr().cast());
                return Ok(ptr);
            }

            prev_ptr = current_ptr;
            current_ptr = current_block.next;
        }

        // No suitable block found
        Err(AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // let lowest_addr = ssd_os_mem_get(0);
        // assert!(lowest_addr <= ptr.as_ptr().cast());
        // crate::println_s!(c"DEALLOC!");
        return;
        let Some(start) = self.start.get() else {
            return;
        };

        let Some(end) = self.end.get() else {
            return;
        };

        if ptr.as_mut() < start.as_mut().unwrap() {
            return;
        } else if ptr.as_mut() >= end.as_mut().unwrap() {
            return;
        }

        let ptr_addr = ptr.addr();

        let size = layout.size().max(mem::size_of::<FreeBlock>());

        let _adjusted_ptr = if layout.align() > mem::align_of::<FreeBlock>() {
            // Ensure pointer is aligned for both the layout and FreeBlock
            align_up(ptr.addr().into(), layout.align()) as *mut u8
        } else {
            ptr.as_mut()
        };
        // let block_ptr = adjusted_ptr as *mut FreeBlock;

        let block_ptr: *mut FreeBlock = ptr.cast().as_mut();

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
