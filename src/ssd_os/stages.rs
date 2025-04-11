use core::ffi::CStr;

use bindings::{MAGIC_STAGE, stage};

use crate::bindings;
impl stage {
    pub const fn new(
        name: &CStr,
        init: unsafe extern "C" fn() -> ::core::ffi::c_int,
        exit: unsafe extern "C" fn() -> ::core::ffi::c_int,
        stage_fn: unsafe extern "C" fn(
            context: *mut ::core::ffi::c_void,
        ) -> *mut ::core::ffi::c_void,
    ) -> Self {
        stage {
            magic: *MAGIC_STAGE,
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
            init_fn: Some(init),
            exit_fn: Some(exit),
            stage_fn: Some(stage_fn),
        }
    }
}

#[macro_export]
macro_rules! make_stage {
    (
        $name:expr,
        $init_fn:ident,
        $exit_fn:ident,
        $stage_fn:ident
    ) => {{
        {
            unsafe extern "C" fn wrapped_init() -> ::core::ffi::c_int {
                $init_fn()
            }

            unsafe extern "C" fn wrapped_exit() -> ::core::ffi::c_int {
                $exit_fn()
            }

            unsafe extern "C" fn wrapped_stage(
                context: *mut ::core::ffi::c_void,
            ) -> *mut ::core::ffi::c_void {
                $stage_fn(context)
            }

            $crate::bindings::stage::new($name, wrapped_init, wrapped_exit, wrapped_stage)
        }
    }};
}

#[macro_export]
macro_rules! make_stage_static {
    ($ident:ident, $init:ident, $exit:ident, $stage_fn:ident) => {
        #[unsafe(no_mangle)]
        pub static $ident: $crate::bindings::stage =
            $crate::make_stage!($crate::cstr!($ident), $init, $exit, $stage_fn);
    };
}
