#[macro_export]
macro_rules! cstr {
    ($ident:ident) => {
        unsafe { CStr::from_ptr(concat!(stringify!($ident), "\0").as_ptr()) }
    };
}
