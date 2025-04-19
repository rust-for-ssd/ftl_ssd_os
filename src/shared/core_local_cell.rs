use core::cell::UnsafeCell;

pub struct CoreLocalCell<T> {
    inner: UnsafeCell<Option<T>>,
}

// ASSUMTION: this is only accesed by a single core.
unsafe impl<T> Sync for CoreLocalCell<T> {}

impl<T> CoreLocalCell<T> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    pub fn set(&self, v: T) {
        let inner = unsafe { self.inner.get().as_mut().expect("Inner was Null") };
        inner.replace(v);
    }

    pub fn get(&self) -> &T {
        let inner = unsafe { self.inner.get().as_ref().expect("Inner was Null") };
        let Some(v) = inner else {
            panic!("Get before initialization of cell!");
        };
        v
    }

    pub fn get_mut(&self) -> &mut T {
        let inner = unsafe { self.inner.get().as_mut().expect("Inner was Null") };
        let Some(v) = inner else {
            panic!("Get before initialization of cell!");
        };
        v
    }
}
