// WARNING: This is only needed because we dont want the global allocator,
// however it is needed for the alloc crate to compile...
// To overcome this, we make fake symbols, which should never appear in the binary.
// It is adviced to use a script that check for the symbols when compiling.

struct FakeAllocator;

unsafe impl core::alloc::GlobalAlloc for FakeAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe extern "Rust" {
            fn fake_alloc_this_doesnt_exist(layout: core::alloc::Layout) -> *mut u8;
        }
        unsafe { fake_alloc_this_doesnt_exist(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe extern "Rust" {
            fn fake_dealloc_this_doesnt_exist(ptr: *mut u8, layout: core::alloc::Layout);
        }
        unsafe { fake_dealloc_this_doesnt_exist(ptr, layout) }
    }
}

#[global_allocator]
static FAKE_ALLOCATOR: FakeAllocator = FakeAllocator;
