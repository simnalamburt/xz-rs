extern crate libc;

//
// lzma/version.h
//
extern {
    pub fn lzma_version_number() -> libc::uint32_t;
    pub fn lzma_version_string() -> *const libc::c_char;
}

//
// lzma/base.h
//
#[repr(C)]
pub type lzma_boot = libc::c_uchar;

#[repr(C)]
pub type lzma_reserved_enum = libc::c_int;
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
    pub alloc: *mut extern fn(opaque: *mut libc::c_void, nmemb: libc::size_t, size: libc::size_t),
    pub free: extern fn(opaque: *mut libc::c_void, ptr: *mut libc::c_void),
    pub opaque: *mut libc::c_void
}

#[repr(C)]
pub struct lzma_internal;

#[repr(C)]
pub struct lzma_stream {
    pub next_in: *const libc::uint8_t,
    pub avail_in: libc::size_t,
    pub total_in: libc::uint64_t,

    pub next_out: *mut libc::uint8_t,
    pub avail_out: libc::size_t,
    pub total_out: libc::uint64_t,

    pub allocator: *const lzma_allocator,

    pub internal: *mut lzma_internal,

    pub reserved_ptr1: *mut libc::c_void,
    pub reserved_ptr2: *mut libc::c_void,
    pub reserved_ptr3: *mut libc::c_void,
    pub reserved_ptr4: *mut libc::c_void,
    pub reserved_int1: *mut libc::uint64_t,
    pub reserved_int2: *mut libc::uint64_t,
    pub reserved_int3: *mut libc::size_t,
    pub reserved_int4: *mut libc::size_t,
    pub reserved_enum1: *mut lzma_reserved_enum,
    pub reserved_enum2: *mut lzma_reserved_enum,
}

extern {
    pub fn lzma_code(stream: *mut lzma_stream, action: lzma_action) -> lzma_ret;
    pub fn lzma_end(stream: *mut lzma_stream);
    pub fn lzma_get_progress(stream: *mut lzma_stream, progress_in: *mut libc::uint64_t, progress_out: *mut libc::uint64_t);
    pub fn lzma_memusage(stream: *const lzma_stream) -> libc::uint64_t;
    pub fn lzma_memlimit_get(stream: *const lzma_stream) -> libc::uint64_t;
    pub fn lzma_memlimit_set(stream: *mut lzma_stream, memlimit: libc::uint64_t) -> lzma_ret;
}

//
// lzma/container.h
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

extern {
    pub fn lzma_easy_encoder(stream: *mut lzma_stream, preset: libc::uint32_t, check: lzma_check) -> lzma_ret;
}
