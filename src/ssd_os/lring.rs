use core::{cell::OnceCell, ffi::CStr, mem::MaybeUninit};

use crate::bindings::{lring, lring_entry, ssd_os_lring_create, ssd_os_lring_dequeue};

pub struct LRing<const Capacity: usize> {
    ssd_os_lring: OnceCell<*mut lring>,
    buffer: [MaybeUninit<u8>; Capacity],
}

// TODO: Should this really be Sync???
unsafe impl<const Capacity: usize> Sync for LRing<Capacity> {}

impl<const Capacity: usize> LRing<Capacity> {
    pub fn dequeue(&self, entry: *mut lring_entry) -> Option<*mut lring_entry> {
        if let 0 = unsafe { ssd_os_lring_dequeue(*self.ssd_os_lring.get()?, entry) } {
            Some(entry)
        } else {
            None
        }
    }

    pub fn init(&self, name: &CStr, flags: i32) {
        let _ = self.ssd_os_lring.set(unsafe {
            ssd_os_lring_create(
                name.as_ptr().cast_mut(),
                Capacity as i32,
                self.buffer.as_ptr().cast_mut().cast(),
                flags,
            )
        });
    }

    pub const fn new() -> Self {
        LRing {
            ssd_os_lring: OnceCell::new(),
            buffer: [MaybeUninit::uninit(); Capacity],
        }
    }
}
