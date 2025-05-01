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
    () => {{
        static DUMMY: u8 = 0;
        unsafe { core::ptr::read_volatile(&DUMMY) };
    }};
}

macro_rules! println {
    // Case where there's only one argument and it is a literal format string
        ($arg:expr) => {{
        #[cfg(feature = "test")]
        {
            semihosting::println!("{}", $arg);
        }

        #[cfg(not(feature = "test"))]
        {
        use core::fmt::Write;
        let mut printer = $crate::bindings::safe::SSD_OS_Printer {};
        let _ = writeln!(printer, "{}", $arg);
        }
    }};
    // Case where there are multiple arguments, including format string and parameters
    ($fmt:expr, $($args:tt)+) => {{
        #[cfg(feature = "test")]
        {
            semihosting::println!($fmt, $($args)+);
        }
        #[cfg(not(feature = "test"))]
        {
        use core::fmt::Write;
        let mut printer = $crate::bindings::safe::SSD_OS_Printer {};
        let _ = writeln!(printer, $fmt, $($args)+);
        }
    }};
}

pub(crate) use cstr;
pub(crate) use ensure_unique;
pub(crate) use println;
