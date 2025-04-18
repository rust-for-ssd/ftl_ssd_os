use core::ffi::{CStr, c_void};

use super::generated::{ssd_os_mem_get, ssd_os_mem_size, ssd_os_this_cpu};

pub struct MemoryRegion {
    start: *mut c_void,
    pub end: *mut c_void,
    pub free_start: *mut c_void,
}

impl MemoryRegion {
    pub fn new(owner: &CStr) -> Self {
        let cpu = unsafe { ssd_os_this_cpu(owner.as_ptr().cast_mut()) };
        let start = unsafe { ssd_os_mem_get(cpu) };
        let end = unsafe { start.byte_add(ssd_os_mem_size(cpu) as usize) };
        Self {
            start,
            end,
            free_start: start,
        }
    }

    pub fn reserve(&mut self, size: usize) -> Option<*mut c_void> {
        if self.free_start.is_null() {
            return None;
        }

        let res = self.free_start;

        let mut free_start = unsafe { self.free_start.byte_add(size) };
        free_start = unsafe { free_start.byte_offset(8) };

        if free_start > self.end {
            return None;
        }

        self.free_start = free_start;
        Some(res)
    }
}
