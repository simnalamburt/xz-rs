extern crate libc;

use libc::*;

//
// lzma/version.h
//
pub const LZMA_VERSION_STABILITY_ALPHA: c_int = 0;
pub const LZMA_VERSION_STABILITY_BETA: c_int = 1;
pub const LZMA_VERSION_STABILITY_STABLE: c_int = 2;

extern {
    pub fn lzma_version_number() -> uint32_t;
    pub fn lzma_version_string() -> *const c_char;
}

//
// lzma/base.h
//
#[repr(C)]
pub type lzma_bool = c_uchar;

// Note: https://github.com/rust-lang/rust/issues/10292
#[repr(C)]
pub type lzma_reserved_enum = c_int;
pub const LZMA_RESERVED_ENUM: lzma_reserved_enum = 0;

pub use lzma_ret::*;
#[repr(C)]
#[derive(Copy)]
pub enum lzma_ret {
    LZMA_OK                 = 0,
    LZMA_STREAM_END         = 1,
    LZMA_NO_CHECK           = 2,
    LZMA_UNSUPPORTED_CHECK  = 3,
    LZMA_GET_CHECK          = 4,
    LZMA_MEM_ERROR          = 5,
    LZMA_MEMLIMIT_ERROR     = 6,
    LZMA_FORMAT_ERROR       = 7,
    LZMA_OPTIONS_ERROR      = 8,
    LZMA_DATA_ERROR         = 9,
    LZMA_BUF_ERROR          = 10,
    LZMA_PROG_ERROR         = 11,
}

pub use lzma_action::*;
#[repr(C)]
#[derive(Copy)]
pub enum lzma_action {
    LZMA_RUN = 0,
    LZMA_SYNC_FLUSH = 1,
    LZMA_FULL_FLUSH = 2,
    LZMA_FULL_BARRIER = 4,
    LZMA_FINISH = 3
}

#[repr(C)]
pub struct lzma_allocator {
    pub alloc: *mut extern fn(opaque: *mut c_void, nmemb: size_t, size: size_t),
    pub free: extern fn(opaque: *mut c_void, ptr: *mut c_void),
    pub opaque: *mut c_void
}

#[repr(C)]
pub struct lzma_internal;

#[repr(C)]
pub struct lzma_stream {
    pub next_in: *const uint8_t,
    pub avail_in: size_t,
    pub total_in: uint64_t,

    pub next_out: *mut uint8_t,
    pub avail_out: size_t,
    pub total_out: uint64_t,

    pub allocator: *const lzma_allocator,

    pub internal: *mut lzma_internal,

    pub reserved_ptr1: *mut c_void,
    pub reserved_ptr2: *mut c_void,
    pub reserved_ptr3: *mut c_void,
    pub reserved_ptr4: *mut c_void,
    pub reserved_int1: *mut uint64_t,
    pub reserved_int2: *mut uint64_t,
    pub reserved_int3: *mut size_t,
    pub reserved_int4: *mut size_t,
    pub reserved_enum1: *mut lzma_reserved_enum,
    pub reserved_enum2: *mut lzma_reserved_enum,
}

extern {
    pub fn lzma_code(stream: *mut lzma_stream, action: lzma_action) -> lzma_ret;
    pub fn lzma_end(stream: *mut lzma_stream);
    pub fn lzma_get_progress(stream: *mut lzma_stream, progress_in: *mut uint64_t, progress_out: *mut uint64_t);
    pub fn lzma_memusage(stream: *const lzma_stream) -> uint64_t;
    pub fn lzma_memlimit_get(stream: *const lzma_stream) -> uint64_t;
    pub fn lzma_memlimit_set(stream: *mut lzma_stream, memlimit: uint64_t) -> lzma_ret;
}

//
// lzma/vli.h
//
pub const LZMA_VLI_MAX: uint64_t = std::u64::MAX / 2;
pub const LZMA_VLI_UNKNOWN: uint64_t = std::u64::MAX;
pub const LZMA_VLI_BYTES_MAX: c_int = 9;

#[repr(C)]
pub type lzma_vli = uint64_t;

#[inline]
pub fn lzma_vli_is_valid(vli: lzma_vli) -> bool {
    vli <= LZMA_VLI_MAX || vli == LZMA_VLI_UNKNOWN
}

extern {
    pub fn lzma_vli_encode(vli: lzma_vli, vli_pos: *mut size_t, out: *mut uint8_t, out_pos: *mut size_t, out_size: size_t) -> lzma_ret;
    pub fn lzma_vli_decode(vli: *mut lzma_vli, vli_pos: *mut size_t, in_: *const uint8_t, in_pos: *mut size_t, in_size: size_t) -> lzma_ret;
    pub fn lzma_vli_size(vli: lzma_vli) -> uint32_t;
}

//
// lzma/check.h
//
pub use lzma_check::*;
#[repr(C)]
#[derive(Copy)]
pub enum lzma_check {
    LZMA_CHECK_NONE     = 0,
    LZMA_CHECK_CRC32    = 1,
    LZMA_CHECK_CRC64    = 4,
    LZMA_CHECK_SHA256   = 10
}

pub const LZMA_CHECK_ID_MAX: c_int = 15;

extern {
    pub fn lzma_check_is_supported(check: lzma_check) -> lzma_bool;
    pub fn lzma_check_size(check: lzma_check) -> uint32_t;
}

pub const LZMA_CHECK_SIZE_MAX: c_int = 64;

extern {
    pub fn lzma_crc32(buf: *const uint8_t, size: size_t, crc: uint32_t) -> uint32_t;
    pub fn lzma_crc64(buf: *const uint8_t, size: size_t, crc: uint64_t) -> uint64_t;
    pub fn lzma_get_check(strm: *const lzma_stream) -> lzma_check;
}

//
// lzma/filter.h
//
pub const LZMA_FILTERS_MAX: c_int = 4;

#[repr(C)]
pub struct lzma_filter {
    pub id: lzma_vli,
    pub options: *mut c_void,
}

extern {
    pub fn lzma_filter_encoder_is_supported(id: lzma_vli) -> lzma_bool;
    pub fn lzma_filter_decoder_is_supported(id: lzma_vli) -> lzma_bool;
    pub fn lzma_filters_copy(src: *const lzma_filter, dest: *mut lzma_filter, allocator: *const lzma_allocator) -> lzma_ret;
    pub fn lzma_raw_encoder_memusage(filters: *const lzma_filter) -> uint64_t;
    pub fn lzma_raw_decoder_memusage(filters: *const lzma_filter) -> uint64_t;
    pub fn lzma_raw_encoder(strm: *mut lzma_stream, filters: *const lzma_filter) -> lzma_ret;
    pub fn lzma_raw_decoder(strm: *mut lzma_stream, filters: *const lzma_filter) -> lzma_ret;
    pub fn lzma_filters_update(strm: *mut lzma_stream, filters: *const lzma_filter) -> lzma_ret;
    pub fn lzma_raw_buffer_encode(filters: *const lzma_filter, allocator: *const lzma_allocator, in_: *const uint8_t, in_size: size_t, out: *mut uint8_t, out_pos: *mut size_t, out_size: size_t) -> lzma_ret;
    pub fn lzma_raw_buffer_decode(filters: *const lzma_filter, allocator: *const lzma_allocator, in_: *const uint8_t, in_pos: *mut size_t, in_size: size_t, out: *mut uint8_t, out_pos: *mut size_t, out_size: size_t) -> lzma_ret;
    pub fn lzma_properties_size(size: *mut uint32_t, filter: *const lzma_filter) -> lzma_ret;
    pub fn lzma_properties_encode(filter: *const lzma_filter, props: *mut uint8_t) -> lzma_ret;
    pub fn lzma_properties_decode(filter: *mut lzma_filter, allocator: *const lzma_allocator, props: *const uint8_t, props_size: size_t) -> lzma_ret;
    pub fn lzma_filter_flags_size(size: *mut uint32_t, filter: *const lzma_filter) -> lzma_ret;
    pub fn lzma_filter_flags_encode(filter: *const lzma_filter, out: *mut uint8_t, out_pos: *mut size_t, out_size: size_t) -> lzma_ret;
    pub fn lzma_filter_flags_decode(filter: *mut lzma_filter, allocator: *const lzma_allocator, in_: *const uint8_t, in_pos: *mut size_t, in_size: size_t) -> lzma_ret;
}

//
// lzma/container.h
//
pub const LZMA_CONCATENATED: uint32_t = 0x08;

extern {
    pub fn lzma_easy_encoder(stream: *mut lzma_stream, preset: uint32_t, check: lzma_check) -> lzma_ret;
    pub fn lzma_stream_decoder(stream: *mut lzma_stream, memlimit: uint64_t, flags: uint32_t) -> lzma_ret;
    pub fn lzma_auto_decoder(stream: *mut lzma_stream, memlimit: uint64_t, flags: uint32_t) -> lzma_ret;
}

//
// lzma/hardware.h
//
extern {
    pub fn lzma_physmem() -> uint64_t;
}
