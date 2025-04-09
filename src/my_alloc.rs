use core::alloc::{GlobalAlloc, Layout};
use core::cell::{Cell};
use core::mem;

use crate::safe_bindings::{ssd_os_print_i, ssd_os_print_lock, ssd_os_print_s, ssd_os_print_ss, ssd_os_print_unlock};
// use core::panicking::panic;



// ISSUES: Dealloc does not work.
// Dynamic allocator

/// A simple memory allocator for embedded systems with a single memory region.
pub struct SimpleAllocator {
    start: Cell<usize>,
    end: Cell<usize>,
    initialized: Cell<bool>,
    free_list_head: Cell<*mut FreeBlock>,
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
    pub const unsafe fn new() -> Self {
        let test = Self {
            start: Cell::new(0),
            end: Cell::new(0),
            initialized: Cell::new(false),
            free_list_head: Cell::new(core::ptr::null_mut()),
        };
        test
    }

    /// Initializes the allocator by setting up the initial free block.
    pub fn initialize(&self, start: usize, end: usize) {
        ssd_os_print_lock();
        ssd_os_print_s(c"YOOO\n");
        ssd_os_print_unlock();
        
        
        ssd_os_print_lock();
        ssd_os_print_s(c"START ADDR: \n");
        ssd_os_print_unlock();
        
        ssd_os_print_lock();
        crate::safe_bindings::ssd_os_print_i(start as u32);
        ssd_os_print_unlock();
        
        ssd_os_print_lock();
        ssd_os_print_s(c"\n");
        ssd_os_print_unlock();

        ssd_os_print_lock();
        ssd_os_print_s(c"END ADDR: \n");
        ssd_os_print_unlock();
        
        ssd_os_print_lock();
        crate::safe_bindings::ssd_os_print_i(end as u32);
        ssd_os_print_unlock();
        
        let is_init : bool = self.initialized.get();
        assert!(is_init as u32 == 1);
        // assert!(is_init as u32 == 111);

                
        if is_init {
                    ssd_os_print_lock();
            ssd_os_print_s(c"WTF??????\n");
            ssd_os_print_i(self.initialized.get() as u32);
            ssd_os_print_s(c"WTF??????\n");
               ssd_os_print_unlock();

            return;
        }
        
                ssd_os_print_s(c"LOL: \n");
        
        
        self.start.set(start);
        self.end.set(end);
        
        crate::safe_bindings::ssd_os_print_i(self.start.get() as u32);
        crate::safe_bindings::ssd_os_print_i(self.end.get() as u32);

        

        let size = self.end.get() - self.start.get();

        // Only initialize if the region has enough space for a block
        if size >= mem::size_of::<FreeBlock>() {
            let block_ptr = self.start.get() as *mut FreeBlock;

            // Safety: We're writing to memory we own
            unsafe {
                (*block_ptr).size = size;
                (*block_ptr).next = core::ptr::null_mut();
                self.free_list_head.set(block_ptr);
            }
        }
        
        self.initialized.set(true); 
    }
}

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Ensure the allocator is initialized
        // if !self.initialized.get() {
        //     panic!("Allocator not initialized!")
        // }

        let size = layout.size().max(mem::size_of::<FreeBlock>());
        let align = layout.align().max(mem::align_of::<FreeBlock>());

        let mut current_ptr = self.free_list_head.get();
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
                        self.free_list_head.set(new_block_ptr);
                    } else {
                        (*prev_ptr).next = new_block_ptr;
                    }
                } else {
                    // Use the entire block
                    if prev_ptr.is_null() {
                        self.free_list_head.set(current_block.next);
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
        if ptr_addr < self.start.get() || ptr_addr >= self.end.get() {
            #[cfg(debug_assertions)]
            panic!("Attempted to free memory not managed by this allocator");
            return;
        }

        let size = layout.size().max(mem::size_of::<FreeBlock>());
        
        let adjusted_ptr = if layout.align() > mem::align_of::<FreeBlock>() {
            // Ensure pointer is aligned for both the layout and FreeBlock
            align_up(ptr as usize, layout.align()) as *mut u8
        } else {
            ptr
        };
        let block_ptr = adjusted_ptr as *mut FreeBlock;
        
        let block_ptr = ptr as *mut FreeBlock;

        (*block_ptr).size = size;

        // Insert the block at the beginning of the free list
        loop {
            let head = self.free_list_head.get();
            (*block_ptr).next = head;

            // if self
            //     .free_list_head
            //     .compare_exchange(head, block_ptr, Ordering::Release, Ordering::Relaxed)
            //     .is_ok()
            // {
            //     break;
            // }
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