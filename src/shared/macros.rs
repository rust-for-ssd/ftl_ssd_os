macro_rules! cstr {
    ($ident:ident) => {
        unsafe {
            ::core::ffi::CStr::from_bytes_with_nul_unchecked(
                concat!(stringify!($ident), "\0").as_bytes(),
            )
        }
    };
}

macro_rules! ensure_unique {
    () => {
        {
            static DUMMY: u8 = 0;
            unsafe { core::ptr::read_volatile(&DUMMY) };
        }
    };
}

pub(crate) use cstr;
pub(crate) use ensure_unique;
