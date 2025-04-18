use crate::bindings::safe::ssd_os_mem_get;
use core::alloc::{AllocError, Allocator, Layout};
use core::cell::{Cell, OnceCell};
use core::ptr::NonNull;
use core::{mem, ptr};

#[derive(Debug, PartialEq)]
pub struct SimpleAllocator {
    start: OnceCell<*mut u8>,
    end: OnceCell<*mut u8>,
    free_list_head: Cell<*mut FreeBlock>,
}

struct FreeBlock {
    // Allocates at least 8 bytes for any sizes
    size: usize,          // 4 bytes in 32 bit systems
    next: *mut FreeBlock, // 4 bytes in 32 bit systems
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
            panic!("Cannot set start region")
        };

        let Ok(()) = self.end.set(end) else {
            panic!("Cannot set end region")
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
        } else {
            panic!("Not enough space for a block")
        }
    }

    fn coalesce_blocks(&self) {
        let mut current = self.free_list_head.get();

        while !current.is_null() && unsafe { (*current).next } != ptr::null_mut() {
            let next_block = unsafe { (*current).next };

            // Check if blocks are adjacent
            if (current as usize) + unsafe { (*current).size } == next_block as usize {
                // Merge blocks
                unsafe {
                    // Increase size of current block to include next block
                    (*current).size += (*next_block).size;
                    // Skip the next block in the list
                    (*current).next = (*next_block).next;
                }
            } else {
                // Move to next block if no merge happened
                current = unsafe { (*current).next };
            }
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
        let lowest_addr = ssd_os_mem_get(0);
        assert!(lowest_addr <= ptr.as_ptr().cast());

        let Some(start) = self.start.get() else {
            panic!("Cannot get start region")
        };

        let Some(end) = self.end.get() else {
            panic!("Cannot end start region")
        };

        // Validate pointer is within our memory range
        if ptr.as_ptr() < *start {
            panic!("Deallocation attempted with pointer below allocator range");
        } else if ptr.as_ptr() >= *end {
            panic!("Deallocation attempted with pointer beyond allocator range");
        }

        // Calculate the size to free (adjust for alignment if needed)
        let size = layout.size().max(mem::size_of::<FreeBlock>());
        let align = layout.align().max(mem::align_of::<FreeBlock>());

        // Create a new free block
        let block_ptr = ptr.as_ptr() as *mut FreeBlock;

        // Find the right place to insert the block in the free list (keep it sorted by address)
        let mut current = self.free_list_head.get();
        let mut prev: *mut FreeBlock = ptr::null_mut();

        // Find where to insert the new free block (in address order)
        while !current.is_null() && current < block_ptr {
            prev = current;
            current = unsafe { (*current).next };
        }

        // Initialize the new free block
        unsafe {
            (*block_ptr).size = size;
            (*block_ptr).next = current;
        }

        // Link it to the free list
        if prev.is_null() {
            // Insert at the beginning
            self.free_list_head.set(block_ptr);
        } else {
            // Insert after prev
            unsafe {
                (*prev).next = block_ptr;
            }
        }

        // NOT TESTED YET!!
        self.coalesce_blocks();
    }
}
