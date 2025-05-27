use core::ffi::CStr;

use bindings::generated::MAGIC_STAGE;
pub use bindings::generated::stage;

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
            magic: *MAGIC_STAGE.first_chunk::<4>().unwrap(),
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
    pub const fn get_name(&self) -> &CStr {
        let Ok(s) = CStr::from_bytes_until_nul(&self.name) else {
            panic!("ERR");
        };
        s
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
                // TODO: replace this with a proc macro.
                // For now, Rust optimizes the code to use the same function ptr
                // if the stage_fn body is the same as another.
                // ssd_os need unique fn ptrs for stages.
                // The following is a trick to avoid sharing fn ptrs.
                // It can be avoided by not wrapping the fn or
                // by creating a proc macro which changes the type.
                #[used]
                static UNIQUE: [u8; 0] = [];
                unsafe { core::ptr::read_volatile(&UNIQUE) };
                $stage_fn(context)
            }

            $crate::bindings::stages::stage::new($name, wrapped_init, wrapped_exit, wrapped_stage)
        }
    }};
}

#[macro_export]
macro_rules! make_stage_static {
    ($ident:ident, $init:ident, $exit:ident, $stage_fn:ident) => {
        #[unsafe(no_mangle)]
        pub static $ident: $crate::bindings::stages::stage = $crate::make_stage!(
            $crate::shared::macros::cstr!($ident),
            $init,
            $exit,
            $stage_fn
        );
    };
}
