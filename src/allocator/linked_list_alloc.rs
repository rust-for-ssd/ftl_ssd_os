use crate::bindings::safe::ssd_os_mem_get;
use core::alloc::{AllocError, Allocator, Layout};
use core::cell::{Cell, OnceCell};
use core::ptr::NonNull;
use core::slice::from_raw_parts_mut;
use core::{mem, ptr};

// Linked list allocator
#[derive(Debug, PartialEq)]
pub struct LinkedListAllocator {
    start: OnceCell<*mut u8>,
    end: OnceCell<*mut u8>,
    free_list_head: Cell<*mut FreeBlock>,
}

struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            start: OnceCell::new(),
            end: OnceCell::new(),
            free_list_head: Cell::new(ptr::null_mut()),
        }
    }

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

        if size >= mem::size_of::<FreeBlock>() {
            let block_ptr = *self.start.get().unwrap() as *mut FreeBlock;

            // SAFETY: We're writing to memory we own,
            // if not then this is unsafe.
            unsafe {
                (*block_ptr).size = size;
                (*block_ptr).next = core::ptr::null_mut();
            }
            self.free_list_head.set(block_ptr);
        } else {
            panic!("Not enough space for a block")
        }
    }

    fn coalesce(&self) {
        let mut current = self.free_list_head.get();

        // go though the linked list and merge free blocks
        while !current.is_null() && unsafe { (*current).next } != ptr::null_mut() {
            let next_block = unsafe { (*current).next };

            // if they are connected, then merge
            // otherwise update current
            if (current as usize) + unsafe { (*current).size } == next_block as usize {
                unsafe {
                    (*current).size += (*next_block).size;
                    (*current).next = (*next_block).next;
                }
            } else {
                current = unsafe { (*current).next };
            }
        }
    }
}

unsafe impl Allocator for LinkedListAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let size = layout.size().max(mem::size_of::<FreeBlock>());
        let align = layout.align().max(mem::align_of::<FreeBlock>());

        let mut current_ptr = self.free_list_head.get();
        let mut prev_ptr: *mut FreeBlock = core::ptr::null_mut();

        // walk the linked list until end or found free block >= size
        while !current_ptr.is_null() {
            let current_block = unsafe { &*current_ptr };
            let block_addr = current_ptr.addr();

            let align_offset = (align - (block_addr % align)) % align;
            let aligned_addr = block_addr + align_offset;

            // check if block can contain the layout
            if aligned_addr + size <= block_addr + current_block.size {
                let remaining_size = (block_addr + current_block.size) - (aligned_addr + size);

                // if block - layout >= size(free block)
                // then create new block from remaining.
                // otherwise allocate full block
                if remaining_size >= mem::size_of::<FreeBlock>() {
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
                    if prev_ptr.is_null() {
                        self.free_list_head.set(current_block.next);
                    } else {
                        unsafe {
                            (*prev_ptr).next = current_block.next;
                        }
                    }
                }

                let ptr: NonNull<[u8]> =
                    unsafe { from_raw_parts_mut(aligned_addr as *mut u8, size).into() };
                // TODO: debug asserting
                let lowest_addr = ssd_os_mem_get(0);
                assert!(lowest_addr <= ptr.as_ptr().cast());
                return Ok(ptr);
            }

            prev_ptr = current_ptr;
            current_ptr = current_block.next;
        }

        // if there are no free block with the size
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

        if ptr.as_ptr() < *start {
            panic!(
                "Deallocation attempted with pointer below allocator range: {:p} < {:p}",
                ptr.as_ptr(),
                *start
            );
        } else if ptr.as_ptr() >= *end {
            panic!(
                "Deallocation attempted with pointer beyond allocator range: {:p} >= {:p}",
                ptr.as_ptr(),
                *end
            );
        }

        let size = layout.size().max(mem::size_of::<FreeBlock>());
        let block_ptr = ptr.as_ptr() as *mut FreeBlock;
        let mut current = self.free_list_head.get();
        let mut prev: *mut FreeBlock = ptr::null_mut();

        // Find the node where to insert the free block
        while !current.is_null() && current < block_ptr {
            prev = current;
            current = unsafe { (*current).next };
        }

        unsafe {
            (*block_ptr).size = size;
            (*block_ptr).next = current;
        }

        if prev.is_null() {
            self.free_list_head.set(block_ptr);
        } else {
            unsafe {
                (*prev).next = block_ptr;
            }
        }

        self.coalesce();
    }
}
