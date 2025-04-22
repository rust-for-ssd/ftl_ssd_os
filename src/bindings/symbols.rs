use super::generated;

#[cfg(not(feature = "test"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
    dest: *mut ::core::ffi::c_void,
    src: *const ::core::ffi::c_void,
    n: u32,
) -> *mut ::core::ffi::c_void {
    let alignment = ::core::mem::size_of::<usize>();
    let dest_usize = dest as usize;
    let src_usize = src as usize;

    if dest_usize % alignment != 0 || src_usize % alignment != 0 {
        // Fallback: byte-by-byte safe copy
        let dest_u8 = dest as *mut u8;
        let src_u8 = src as *const u8;
        for i in 0..n {
            unsafe { *dest_u8.add(i as usize) = *src_u8.add(i as usize) };
        }
        return dest;
    }

    // Safe to use optimized C function
    unsafe { generated::ssd_os_mem_cpy(dest, src, n) }
}

#[cfg(not(feature = "test"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(
    dst: *mut ::core::ffi::c_void,
    src: *const ::core::ffi::c_void,
    size: usize,
) -> *mut ::core::ffi::c_void {
    use core::ptr;

    if dst == src.cast_mut() || size == 0 {
        return dst;
    }

    if (dst as usize) < (src as usize) || (dst as usize) >= (src as usize + size) {
        // No overlap or safe to copy forward
        unsafe { memcpy(dst, src, size as u32) };
    } else {
        // Overlap: copy backwards
        for i in (0..size).rev() {
            unsafe { ptr::write(dst.add(i), ptr::read(src.add(i))) };
        }
    }

    dst
}

#[cfg(not(feature = "test"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(
    dst: *mut ::core::ffi::c_void,
    value: ::core::ffi::c_int,
    size: usize,
) -> *mut ::core::ffi::c_void {
    use core::ptr;

    let mut ptr = dst as *mut u8;
    let byte = value as u8;

    for _ in 0..size {
        unsafe {
            ptr::write(ptr, byte);
            ptr = ptr.add(1);
        }
    }

    dst
}
