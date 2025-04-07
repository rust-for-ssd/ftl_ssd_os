pub const MAGIC_STAGE: &[u8; 4] = b"stg\0";
pub const MAGIC_CONNECTOR: &[u8; 4] = b"con\0";

pub type ssd_os_ctrl_fn = ::core::option::Option<unsafe extern "C" fn() -> ::core::ffi::c_int>;
pub type ssd_os_stage_fn = ::core::option::Option<
    unsafe extern "C" fn(context: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void,
>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct lring_entry {
    pub function: *mut ::core::ffi::c_void,
    pub ctx: *mut ::core::ffi::c_void,
}

pub type ssd_os_conn_fn =
    ::core::option::Option<unsafe extern "C" fn(entry: *mut lring_entry) -> *mut pipeline>;
pub type ssd_os_conn_ring_fn =
    ::core::option::Option<unsafe extern "C" fn(entry: *mut lring_entry) -> ::core::ffi::c_int>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct stage {
    pub magic: [::core::ffi::c_char; 4usize],
    pub name: [::core::ffi::c_char; 32usize],
    pub init_fn: ssd_os_ctrl_fn,
    pub exit_fn: ssd_os_ctrl_fn,
    pub stage_fn: ssd_os_stage_fn,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pipeline {
    pub name: [::core::ffi::c_char; 32usize],
    pub internal: [*mut ::core::ffi::c_void; 2usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct connector {
    pub magic: [::core::ffi::c_char; 4usize],
    pub name: [::core::ffi::c_char; 32usize],
    pub init_fn: ssd_os_ctrl_fn,
    pub exit_fn: ssd_os_ctrl_fn,
    pub conn_fn: ssd_os_conn_fn,
    pub ring_fn: ssd_os_conn_ring_fn,
}

unsafe extern "C" {
    pub fn ssd_os_sleep(sec: ::core::ffi::c_int);
    pub fn ssd_os_this_cpu(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn ssd_os_print_i(x: ::core::ffi::c_ulong);
    pub fn ssd_os_get_connection(
        connector_name: *mut ::core::ffi::c_char,
        pipe_name: *mut ::core::ffi::c_char,
    ) -> *mut pipeline;
    pub fn ssd_os_print_s(s: *const ::core::ffi::c_char);
    pub fn ssd_os_print_lock();
    pub fn ssd_os_print_unlock();
    pub fn ssd_os_print_ss(s1: *const ::core::ffi::c_char, s2: *const ::core::ffi::c_char);
    pub fn ssd_os_mem_get(key: ::core::ffi::c_int) -> *mut ::core::ffi::c_void;
}
