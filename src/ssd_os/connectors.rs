use core::ffi::CStr;

use bindings::{MAGIC_CONNECTOR, connector, lring_entry, pipeline};

use crate::{bindings, println_s};
impl connector {
    pub const fn new(
        name: &CStr,
        init_fn: unsafe extern "C" fn() -> i32,
        exit_fn: unsafe extern "C" fn() -> i32,
        conn_fn: unsafe extern "C" fn(*mut lring_entry) -> *mut pipeline,
        ring_fn: unsafe extern "C" fn(*mut lring_entry) -> i32,
    ) -> Self {
        Self {
            magic: *MAGIC_CONNECTOR,
            name: {
                let mut buf = [0u8; 32];
                let s = name.to_bytes();
                let mut i = 0;
                while i < s.len() {
                    buf[i] = s[i];
                    i += 1;
                }
                buf
            },
            init_fn: Some(init_fn),
            exit_fn: Some(exit_fn),
            conn_fn: Some(conn_fn),
            ring_fn: Some(ring_fn),
        }
    }
    pub fn get_name(&self) -> &CStr {
        let Ok(s) = CStr::from_bytes_until_nul(&self.name) else {
            println_s!(c"ERROR!");
            return c"";
        };
        s
    }
}
