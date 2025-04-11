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

#[macro_export]
macro_rules! make_connector {
    (
        $name:expr,
        $init_fn:ident,
        $exit_fn:ident,
        $conn_fn:ident,
        $ring_fn:ident
    ) => {{
        {
            unsafe extern "C" fn wrapped_init() -> ::core::ffi::c_int {
                $init_fn()
            }

            unsafe extern "C" fn wrapped_exit() -> ::core::ffi::c_int {
                $exit_fn()
            }

            unsafe extern "C" fn wrapped_conn(entry: *mut lring_entry) -> *mut pipeline {
                $conn_fn(entry)
            }

            unsafe extern "C" fn wrapped_ring(entry: *mut lring_entry) -> ::core::ffi::c_int {
                $ring_fn(entry)
            }

            $crate::bindings::connector::new(
                $name,
                wrapped_init,
                wrapped_exit,
                wrapped_conn,
                wrapped_ring,
            )
        }
    }};
}

#[macro_export]
macro_rules! make_connector_static {
    ($ident:ident, $init:ident, $exit:ident, $conn:ident, $ring:ident) => {
        #[unsafe(no_mangle)]
        pub static $ident: $crate::bindings::connector =
            $crate::make_connector!($crate::cstr!($ident), $init, $exit, $conn, $ring);
    };
}
