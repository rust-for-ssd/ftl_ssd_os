macro_rules! cstr {
    ($ident:ident) => {
        unsafe {
            ::core::ffi::CStr::from_bytes_with_nul_unchecked(
                concat!(stringify!($ident), "\0").as_bytes(),
            )
        }
    };
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
        $crate::bindings::safe::ssd_os_print_lock();
        let _ = writeln!(printer, "{}", $arg);
        $crate::bindings::safe::ssd_os_print_unlock();
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
        $crate::bindings::safe::ssd_os_print_lock();
        let _ = writeln!(printer, $fmt, $($args)+);
        $crate::bindings::safe::ssd_os_print_unlock();
        }
    }};
}

macro_rules! dbg_println {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug")]
        $crate::shared::macros::println!($($arg)*);
    };
}

macro_rules! dbg {
    () => {
        if cfg!(feature = "debug") {
            $crate::shared::macros::println!("[{}:{}]", file!(), line!());
        }
    };
    ($val:expr) => {
        if cfg!(feature = "debug") {
            match &$val {
                tmp => {
                    $crate::shared::macros::println!(
                        "[{}:{}] {} = {:?}",
                        file!(),
                        line!(),
                        stringify!($val),
                        tmp
                    );
                    tmp
                }
            }
        } else {
            &$val
        }
    };
}
pub(crate) use cstr;
pub(crate) use dbg;
pub(crate) use dbg_println;
pub(crate) use println;
