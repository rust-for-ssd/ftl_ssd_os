use super::generated;

#[cfg(not(feature = "test"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
    dest: *mut ::core::ffi::c_void,
    src: *const ::core::ffi::c_void,
    n: u32,
) -> *mut ::core::ffi::c_void {
    let alignment = ::core::mem::size_of::<usize>();

    if dest as usize % alignment != 0 || src as usize % alignment != 0 {
        let dest_byte_ptr = dest as *mut u8;
        let src_byte_ptr = src as *const u8;
        for i in 0..n {
            unsafe { *dest_byte_ptr.add(i as usize) = *src_byte_ptr.add(i as usize) };
        }
        return dest;
    }

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
        unsafe { memcpy(dst, src, size as u32) };
    } else {
        // reverse cpy
        for i in (0..size).rev() {
            unsafe {
                let dst_i = dst.add(i);
                let src_i = src.add(i);
                let data = ptr::read(src_i);
                ptr::write(dst_i, data);
            };
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
