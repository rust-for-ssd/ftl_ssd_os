use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use crate::bindings::generated::{ssd_os_semaphore_lock, ssd_os_semaphore_unlock};

pub struct Semaphore<T> {
    inner: UnsafeCell<T>,
    ssd_os_semaphore: UnsafeCell<::core::ffi::c_int>,
}

unsafe impl<T: Send> Send for Semaphore<T> {}
unsafe impl<T: Sync> Sync for Semaphore<T> {}

impl<T> Semaphore<T> {
    pub fn lock(&self) -> Guard<'_, T> {
        unsafe {
            ssd_os_semaphore_lock(self.ssd_os_semaphore.get());
        }
        Guard { sem: self }
    }

    pub const fn new(inner: T) -> Self {
        let res = Self {
            inner: UnsafeCell::new(inner),
            ssd_os_semaphore: UnsafeCell::new(0),
        };
        // WARNING: it seems to be correct to not init, but we should check
        // unsafe { ssd_os_semaphore_init(res.ssd_os_semaphore.get()) };
        return res;
    }

    pub fn with_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.lock();
        let ret = f(&mut *guard);
        // dropping guard unlocks
        ret
    }
}

pub struct Guard<'s, T> {
    sem: &'s Semaphore<T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.sem.inner.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.sem.inner.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            ssd_os_semaphore_unlock(self.sem.ssd_os_semaphore.get());
        }
    }
}
