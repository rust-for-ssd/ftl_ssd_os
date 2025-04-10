use core::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

/// A simple memory allocator for embedded systems with a single memory region.
pub struct SimpleAllocator {
    start: usize,
    end: usize,
    initialized: AtomicBool,
    free_list_head: AtomicPtr<FreeBlock>,
}

/// Represents a free block of memory.
#[repr(C)]
struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

unsafe impl Send for SimpleAllocator {}
unsafe impl Sync for SimpleAllocator {}

impl SimpleAllocator {
    /// Creates a new allocator with the given memory region.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided memory region is valid and available for use.
    pub const unsafe fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            initialized: AtomicBool::new(false),
            free_list_head: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    /// Initializes the allocator by setting up the initial free block.
    fn initialize(&self) {
        if self.initialized.load(Ordering::Relaxed) {
            return;
        }

        let size = self.end - self.start;

        // Only initialize if the region has enough space for a block
        if size >= mem::size_of::<FreeBlock>() {
            let block_ptr = self.start as *mut FreeBlock;

            // Safety: We're writing to memory we own
            unsafe {
                (*block_ptr).size = size;
                (*block_ptr).next = core::ptr::null_mut();
                self.free_list_head.store(block_ptr, Ordering::Relaxed);
            }
        }

        self.initialized.store(true, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Ensure the allocator is initialized
        if !self.initialized.load(Ordering::Relaxed) {
            self.initialize();
        }

        let size = layout.size().max(mem::size_of::<FreeBlock>());
        let align = layout.align().max(mem::align_of::<FreeBlock>());

        let mut current_ptr = self.free_list_head.load(Ordering::Acquire);
        let mut prev_ptr: *mut FreeBlock = core::ptr::null_mut();

        while !current_ptr.is_null() {
            let current_block = &*current_ptr;
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
                    (*new_block_ptr).size = remaining_size;
                    (*new_block_ptr).next = current_block.next;

                    if prev_ptr.is_null() {
                        self.free_list_head.store(new_block_ptr, Ordering::Release);
                    } else {
                        (*prev_ptr).next = new_block_ptr;
                    }
                } else {
                    // Use the entire block
                    if prev_ptr.is_null() {
                        self.free_list_head.store(current_block.next, Ordering::Release);
                    } else {
                        (*prev_ptr).next = current_block.next;
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
        return;
        let ptr_addr = ptr as usize;

        // Ensure the pointer is within our memory region
        if ptr_addr < self.start || ptr_addr >= self.end {
            #[cfg(debug_assertions)]
            panic!("Attempted to free memory not managed by this allocator");
            return;
        }

        let size = layout.size().max(mem::size_of::<FreeBlock>());
        let block_ptr = ptr as *mut FreeBlock;

        (*block_ptr).size = size;

        // Insert the block at the beginning of the free list
        loop {
            let head = self.free_list_head.load(Ordering::Acquire);
            (*block_ptr).next = head;

            if self
                .free_list_head
                .compare_exchange(head, block_ptr, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        // Note: A production allocator would merge adjacent free blocks here
    }
}

