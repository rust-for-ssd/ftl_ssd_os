use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use crate::bindings::generated::{
    ssd_os_semaphore_init, ssd_os_semaphore_lock, ssd_os_semaphore_unlock,
};

pub struct Semaphore<T> {
    inner: UnsafeCell<MaybeUninit<T>>,
    ssd_os_semaphore: UnsafeCell<::core::ffi::c_int>,
}

unsafe impl<T: Send> Send for Semaphore<T> {}
unsafe impl<T: Sync> Sync for Semaphore<T> {}

impl<T> Semaphore<T> {
    pub const fn new() -> Self {
        let res = Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
            ssd_os_semaphore: UnsafeCell::new(0),
        };
        return res;
    }

    pub fn init(&self, value: T) {
        unsafe {
            let ptr = (*self.inner.get()).as_mut_ptr();
            ptr.write(value);
            ssd_os_semaphore_init(self.ssd_os_semaphore.get());
        }
    }
    pub fn lock(&self) -> Guard<'_, T> {
        unsafe {
            ssd_os_semaphore_lock(self.ssd_os_semaphore.get());
        }
        Guard { sem: self }
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
    unsafe fn inner_mut(&self) -> *mut T {
        unsafe { (*self.inner.get()).as_mut_ptr() }
    }
}

pub struct Guard<'s, T> {
    sem: &'s Semaphore<T>,
}

impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.sem.inner_mut() }
    }
}

impl<'a, T> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.sem.inner_mut() }
    }
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            ssd_os_semaphore_unlock(self.sem.ssd_os_semaphore.get());
        }
    }
}
