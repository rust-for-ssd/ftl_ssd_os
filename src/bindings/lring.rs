use core::ffi::c_void;
use core::{cell::OnceCell, ffi::CStr};

use crate::bindings::generated::lring_entry;
use crate::bindings::generated::{lring, ssd_os_lring_create, ssd_os_lring_dequeue};

use super::generated::ssd_os_lring_enqueue;

pub enum LRingErr {
    Enqueue(i32),
    Dequeue(i32),
    AlreadyInit(i32),
    NoCtx,
}

pub struct LRing<const Capacity: usize> {
    lring_ptr: OnceCell<*mut lring>,
}

// TODO: Should this really be Sync???
unsafe impl<const Capacity: usize> Sync for LRing<Capacity> {}

impl<const Capacity: usize> LRing<Capacity> {
    pub const fn new() -> Self {
        LRing {
            lring_ptr: OnceCell::new(),
        }
    }

    pub fn init(&self, name: &CStr, dst: *mut c_void, flags: i32) -> Result<(), LRingErr> {
        self.lring_ptr
            .set(unsafe {
                ssd_os_lring_create(name.as_ptr().cast_mut(), Capacity as i32, dst, flags)
            })
            .map_err(|_| LRingErr::AlreadyInit(-1))
    }

    pub fn get_ptr(&self) -> Option<*mut lring> {
        self.lring_ptr.get().copied()
    }

    pub fn get_lring(&self) -> Option<&lring> {
        // SAFETY: as_ref() returns None if the ptr is Null
        unsafe { self.get_ptr()?.as_ref() }
    }

    pub fn enqueue(&self, entry: *mut lring_entry) -> Result<(), LRingErr> {
        let Some(lring) = self.get_ptr() else {
            return Err(LRingErr::Enqueue(-1));
        };

        // SAFETY: returns 0 if dequed, otherwise err code
        match unsafe { ssd_os_lring_enqueue(lring, entry) } {
            0 => Ok(()),
            e => Err(LRingErr::Enqueue(e)),
        }
    }

    pub fn dequeue(&self, entry: *mut lring_entry) -> Result<*mut lring_entry, LRingErr> {
        let Some(lring) = self.get_ptr() else {
            return Err(LRingErr::Dequeue(-1));
        };

        // SAFETY: returns 0 if dequed, otherwise err code
        match unsafe { ssd_os_lring_dequeue(lring, entry) } {
            0 => Ok(entry),
            e => Err(LRingErr::Dequeue(e)),
        }
    }

    pub fn dequeue_as_ref(&self, entry: *mut lring_entry) -> Result<&lring_entry, LRingErr> {
        // SAFETY: as_ref() returns None if the ptr is Null
        unsafe {
            self.dequeue(entry)?
                .as_ref()
                .map_or(Err(LRingErr::Dequeue(-1)), |lring_entry| Ok(lring_entry))
        }
    }

    pub fn dequeue_as_mut(&self, entry: *mut lring_entry) -> Result<&mut lring_entry, LRingErr> {
        // SAFETY: as_ref() returns None if the ptr is Null
        unsafe {
            self.dequeue(entry)?
                .as_mut()
                .map_or(Err(LRingErr::Dequeue(-1)), |lring_entry| Ok(lring_entry))
        }
    }

    pub fn dequeue_as_mut_ctx<T>(&self, entry: *mut lring_entry) -> Result<&mut T, LRingErr> {
        let entry = self.dequeue_as_mut(entry)?;
        let Some(ctx) = entry.get_ctx_as_mut() else {
            return Err(LRingErr::NoCtx);
        };
        Ok(ctx)
    }
}

impl lring_entry {
    pub fn new<'r>(entry: *mut lring_entry) -> Option<&'r mut Self> {
        // SAFETY: as_mut() returns None if the ptr is Null
        unsafe { entry.as_mut() }
    }

    pub fn get_ctx_as_ref<T>(&self) -> Option<&T> {
        // SAFETY: as_ref() returns None if the ptr is Null
        unsafe { self.ctx.cast::<T>().as_ref() }
    }

    pub fn get_ctx_as_mut<T>(&mut self) -> Option<&mut T> {
        // SAFETY: as_ref() returns None if the ptr is Null
        unsafe { self.ctx.cast::<T>().as_mut() }
    }

    pub fn set_ctx<T>(&mut self, new_ctx: &T) {
        let new_ctx_ptr: *const T = new_ctx;
        self.ctx = new_ctx_ptr.cast_mut().cast();
    }

    pub fn get_mut_ctx_raw<'r, T>(entry: *mut Self) -> Option<&'r mut T> {
        let entry: &mut Self = unsafe { entry.as_mut()? };
        unsafe { entry.ctx.cast::<T>().as_mut() }
    }
}
