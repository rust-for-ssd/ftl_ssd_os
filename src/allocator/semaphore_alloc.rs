use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

use crate::shared::semaphore::Semaphore;

use super::sdd_os_alloc::SimpleAllocator;

pub struct SemaphoreAllocator {
    semaphore: Semaphore<SimpleAllocator>,
}

impl SemaphoreAllocator {
    pub const fn new() -> Self {
        Self {
            semaphore: Semaphore::new(),
        }
    }
    pub fn init(&self, start: *mut u8, end: *mut u8) {
        self.semaphore.init(SimpleAllocator::new());
        let guard = self.semaphore.lock();
        guard.initialize(start, end);
    }
}

unsafe impl Allocator for SemaphoreAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let guard = self.semaphore.lock();
        guard.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let guard = self.semaphore.lock();
        unsafe { guard.deallocate(ptr, layout) }
    }
}
