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
    pub fn ssd_os_mem_size(key: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn ssd_os_mem_cpy(
        dest: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        n: u32,
    ) -> *mut ::core::ffi::c_void;
}

// ---------- Volt Manager (MEDIA MANAGER) ---------
unsafe extern "C" {
    pub fn volt_get_geometry(g: *mut nvm_mmgr_geometry) -> ::core::ffi::c_int;
    pub fn volt_get_ch_info(ch_info: *mut volt_info) -> ::core::ffi::c_int;
    pub fn volt_get_last_address(ptr: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
}

// -- Structs etc from volt.h
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct volt_info {
    pub channel_array: *mut nvm_channel,
    pub n_channels: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct nvm_channel {
    pub ch_id: u16,
    pub ch_mmgr_id: u16,
    pub ns_pgs: u64,
    pub slba: u64,
    pub elba: u64,
    pub tot_bytes: u64,
    pub mmgr_rsv: u16,
    pub ftl_rsv: u16,
    pub mmgr: *mut nvm_mmgr,
    pub ftl: *mut nvm_ftl,
    pub geometry: *mut nvm_mmgr_geometry,
    pub mmgr_rsv_list: [nvm_ppa_addr; 16usize],
    pub ftl_rsv_list: [nvm_ppa_addr; 16usize],
    pub __bindgen_anon_1: nvm_channel__bindgen_ty_1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union nvm_ppa_addr {
    pub ppa: u64,
    pub g: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union nvm_ppa_addr__bindgen_ty_1 {
    pub g: u64,
    pub ppa: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union nvm_channel__bindgen_ty_1 {
    pub i: u64,
    pub nvm_info: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvm_ftl {
    pub _address: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvm_mmgr_geometry {
    pub n_of_ch: u8,
    pub lun_per_ch: u8,
    pub blk_per_lun: u16,
    pub pg_per_blk: u16,
    pub sec_per_pg: u16,
    pub n_of_planes: u8,
    pub pg_size: u32,
    pub sec_oob_sz: u32,
    pub sec_per_pl_pg: u32,
    pub sec_per_blk: u32,
    pub sec_per_lun: u32,
    pub sec_per_ch: u32,
    pub pg_per_lun: u32,
    pub pg_per_ch: u32,
    pub blk_per_ch: u32,
    pub tot_sec: u64,
    pub tot_pg: u64,
    pub tot_blk: u32,
    pub tot_lun: u32,
    pub sec_size: u32,
    pub pl_pg_size: u32,
    pub blk_size: u32,
    pub lun_size: u64,
    pub ch_size: u64,
    pub tot_size: u64,
    pub pg_oob_sz: u32,
    pub pl_pg_oob_sz: u32,
    pub blk_oob_sz: u32,
    pub lun_oob_sz: u32,
    pub ch_oob_sz: u64,
    pub tot_oob_sz: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvm_mmgr {
    pub name: *const ::core::ffi::c_char,
    pub ops: *mut nvm_mmgr_ops,
    pub geometry: *mut nvm_mmgr_geometry,
    pub ch_info: *mut nvm_channel,
    pub flags: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvm_mmgr_ops {
    pub read_pg: nvm_mmgr_read_pg,
    pub write_pg: nvm_mmgr_write_pg,
    pub erase_blk: nvm_mmgr_erase_blk,
    pub exit: nvm_mmgr_exit,
    pub get_ch_info: nvm_mmgr_get_ch_info,
    pub set_ch_info: nvm_mmgr_set_ch_info,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvm_io_status {
    pub status: u8,
    pub nvme_status: u16,
    pub pg_errors: u32,
    pub total_pgs: u32,
    pub pgs_p: u16,
    pub pgs_s: u16,
    pub ret_t: u16,
    pub pg_map: [u8; 8usize],
}
pub type nvm_callback_fn =
    ::core::option::Option<unsafe extern "C" fn(arg: *mut ::core::ffi::c_void)>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvm_callback {
    pub cb_fn: nvm_callback_fn,
    pub opaque: *mut ::core::ffi::c_void,
    pub ts: u64,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct nvm_io_cmd {
    pub cid: u64,
    pub channel: [*mut nvm_channel; 64usize],
    pub ppalist: [nvm_ppa_addr; 256usize],
    pub status: nvm_io_status,
    pub mmgr_io: [nvm_mmgr_io_cmd; 64usize],
    pub callback: nvm_callback,
    pub req: *mut ::core::ffi::c_void,
    pub mq_req: *mut ::core::ffi::c_void,
    pub opaque: *mut ::core::ffi::c_void,
    pub prp: [u64; 256usize],
    pub md_prp: [u64; 256usize],
    pub sec_sz: u32,
    pub md_sz: u32,
    pub n_sec: u32,
    pub slba: u64,
    pub cmdtype: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct nvm_mmgr_io_cmd {
    pub nvm_io: *mut nvm_io_cmd,
    pub ppa: nvm_ppa_addr,
    pub ch: *mut nvm_channel,
    pub callback: nvm_callback,
    pub prp: [u64; 32usize],
    pub md_prp: u64,
    pub status: u8,
    pub cmdtype: u8,
    pub pg_index: u32,
    pub pg_sz: u32,
    pub n_sectors: u16,
    pub sec_sz: u32,
    pub md_sz: u32,
    pub sec_offset: u16,
    pub force_sync_md: u8,
    pub force_sync_data: [u8; 32usize],
    pub sync_count: u32,
    pub rsvd: [u8; 128usize],
}
pub type nvm_mmgr_read_pg =
    ::core::option::Option<unsafe extern "C" fn(arg1: *mut nvm_mmgr_io_cmd) -> ::core::ffi::c_int>;
pub type nvm_mmgr_write_pg =
    ::core::option::Option<unsafe extern "C" fn(arg1: *mut nvm_mmgr_io_cmd) -> ::core::ffi::c_int>;
pub type nvm_mmgr_erase_blk =
    ::core::option::Option<unsafe extern "C" fn(arg1: *mut nvm_mmgr_io_cmd) -> ::core::ffi::c_int>;
pub type nvm_mmgr_get_ch_info = ::core::option::Option<
    unsafe extern "C" fn(arg1: *mut nvm_channel, arg2: u16) -> ::core::ffi::c_int,
>;
pub type nvm_mmgr_set_ch_info = ::core::option::Option<
    unsafe extern "C" fn(arg1: *mut nvm_channel, arg2: u16) -> ::core::ffi::c_int,
>;
pub type nvm_mmgr_exit = ::core::option::Option<unsafe extern "C" fn(arg1: *mut nvm_mmgr)>;
