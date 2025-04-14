macro_rules! cstr {
    ($ident:ident) => {
        unsafe {
            ::core::ffi::CStr::from_bytes_with_nul_unchecked(
                concat!(stringify!($ident), "\0").as_bytes(),
            )
        }
    };
}

pub(crate) use cstr;
