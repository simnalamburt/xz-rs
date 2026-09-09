//! liblzma-sys compatible API layer backed by pure Rust xz-core
//!
//! Re-exports symbols from xz-core with the same names and signatures
//! as liblzma-sys, enabling drop-in replacement.
//!
#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unsafe_op_in_unsafe_fn,
    unused_imports,
    clippy::all
)]

//! Because c2rust generates per-file type definitions, this layer provides
//! canonical types and thin wrapper functions that cast between structurally
//! identical `#[repr(C)]` types.

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
use libc::size_t;
#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
use libc::{c_char, c_int, c_uchar, c_uint, c_void};
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_void};
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
type wasm_size_t = usize;
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
use self::wasm_size_t as size_t;

/************************
 * Canonical type aliases
 ************************/

/*
 * Match liblzma-sys' platform-dependent C enum ABI directly.
 */
#[cfg(target_env = "msvc")]
type c_enum = c_int;
#[cfg(not(target_env = "msvc"))]
type c_enum = c_uint;

pub type lzma_ret = c_enum;
pub type lzma_action = c_enum;
pub type lzma_check = c_enum;
pub type lzma_mode = c_enum;
pub type lzma_match_finder = c_enum;
pub type lzma_bool = c_uchar;
pub type lzma_vli = u64;
pub type lzma_delta_type = c_uint;

/****************************
 * Canonical struct re-exports
 ****************************/
pub use xz_core::common::index_hash::lzma_index_hash;
pub use xz_core::types::lzma_allocator;
pub use xz_core::types::lzma_block;
pub use xz_core::types::lzma_filter;
pub use xz_core::types::lzma_index;
pub use xz_core::types::lzma_index_iter;
pub use xz_core::types::lzma_index_iter_mode;
pub use xz_core::types::lzma_mt;
pub use xz_core::types::lzma_options_lzma;
pub use xz_core::types::lzma_stream;
pub use xz_core::types::lzma_stream_flags;
pub use xz_core::types::{
    LZMA_INDEX_ITER_ANY, LZMA_INDEX_ITER_BLOCK, LZMA_INDEX_ITER_NONEMPTY_BLOCK,
    LZMA_INDEX_ITER_STREAM,
};

fn normalize_c_allocator(allocator: *const lzma_allocator) -> *const lzma_allocator {
    xz_core::alloc::allocator_or_c(allocator.cast()).cast()
}

unsafe fn normalize_c_stream_allocator(strm: *mut lzma_stream) {
    if !strm.is_null() && (*strm).allocator.is_null() {
        (*strm).allocator = xz_core::alloc::c_allocator_ptr().cast();
    }
}

#[repr(C)]
pub struct lzma_options_bcj {
    pub start_offset: u32,
}

#[repr(C)]
pub struct lzma_options_delta {
    pub r#type: lzma_delta_type,
    pub dist: u32,
    pub reserved_int1: u32,
    pub reserved_int2: u32,
    pub reserved_int3: u32,
    pub reserved_int4: u32,
    pub reserved_ptr1: *mut c_void,
    pub reserved_ptr2: *mut c_void,
}

pub use xz_core::common::common_types::lzma_internal_s as lzma_internal;

/******************
 * Basic Features *
 ******************/

/* `lzma/version.h`: compile-time version constants */
pub const LZMA_VERSION_MAJOR: u32 = xz_core::common::common::LZMA_VERSION_MAJOR as u32;
pub const LZMA_VERSION_MINOR: u32 = xz_core::common::common::LZMA_VERSION_MINOR as u32;
pub const LZMA_VERSION_PATCH: u32 = xz_core::common::common::LZMA_VERSION_PATCH as u32;
pub const LZMA_VERSION: u32 = xz_core::common::common::LZMA_VERSION as u32;

/* `lzma/base.h`: return codes */
pub use xz_core::types::{
    LZMA_BUF_ERROR, LZMA_DATA_ERROR, LZMA_FORMAT_ERROR, LZMA_GET_CHECK, LZMA_MEM_ERROR,
    LZMA_MEMLIMIT_ERROR, LZMA_NO_CHECK, LZMA_OK, LZMA_OPTIONS_ERROR, LZMA_PROG_ERROR,
    LZMA_SEEK_NEEDED, LZMA_STREAM_END, LZMA_UNSUPPORTED_CHECK,
};

/* `lzma/base.h`: actions */
pub use xz_core::types::{
    LZMA_FINISH, LZMA_FULL_BARRIER, LZMA_FULL_FLUSH, LZMA_RUN, LZMA_SYNC_FLUSH,
};

/* `lzma/vli.h`: backward size / VLI */
pub const LZMA_BACKWARD_SIZE_MIN: lzma_vli = xz_core::types::LZMA_BACKWARD_SIZE_MIN as lzma_vli;
pub const LZMA_BACKWARD_SIZE_MAX: lzma_vli = xz_core::types::LZMA_BACKWARD_SIZE_MAX as lzma_vli;
pub use xz_core::types::{LZMA_VLI_MAX, LZMA_VLI_UNKNOWN};
pub const LZMA_VLI_BYTES_MAX: usize = xz_core::types::LZMA_VLI_BYTES_MAX as usize;

/* `lzma/check.h`: check types */
pub use xz_core::types::{LZMA_CHECK_CRC32, LZMA_CHECK_CRC64, LZMA_CHECK_NONE, LZMA_CHECK_SHA256};
pub const LZMA_CHECK_ID_MAX: u32 = xz_core::types::LZMA_CHECK_ID_MAX as u32;
pub const LZMA_CHECK_SIZE_MAX: u32 = xz_core::types::LZMA_CHECK_SIZE_MAX as u32;

/***********
 * Filters *
 ***********/

/* `lzma/lzma12.h`: modes / match finders */
pub use xz_core::types::{
    LZMA_MF_BT2, LZMA_MF_BT3, LZMA_MF_BT4, LZMA_MF_HC3, LZMA_MF_HC4, LZMA_MODE_FAST,
    LZMA_MODE_NORMAL,
};

/* `lzma/filter.h`: filter IDs */
pub use xz_core::types::{
    LZMA_FILTER_ARM, LZMA_FILTER_ARM64, LZMA_FILTER_ARMTHUMB, LZMA_FILTER_DELTA, LZMA_FILTER_IA64,
    LZMA_FILTER_LZMA1, LZMA_FILTER_LZMA2, LZMA_FILTER_POWERPC, LZMA_FILTER_RISCV,
    LZMA_FILTER_SPARC, LZMA_FILTER_X86,
};
pub const LZMA_FILTERS_MAX: u32 = xz_core::types::LZMA_FILTERS_MAX as u32;
pub const LZMA_DELTA_DIST_MIN: u32 = xz_core::types::LZMA_DELTA_DIST_MIN as u32;
pub const LZMA_DELTA_DIST_MAX: u32 = xz_core::types::LZMA_DELTA_DIST_MAX as u32;
pub const LZMA_DELTA_TYPE_BYTE: lzma_delta_type =
    xz_core::types::LZMA_DELTA_TYPE_BYTE as lzma_delta_type;

/*********************
 * Container Formats *
 *********************/

/* `lzma/container.h`: decoder flags */
pub use xz_core::types::{
    LZMA_CONCATENATED, LZMA_IGNORE_CHECK, LZMA_TELL_ANY_CHECK, LZMA_TELL_NO_CHECK,
    LZMA_TELL_UNSUPPORTED_CHECK,
};

/* `lzma/container.h`: presets / option limits */
pub const LZMA_PRESET_DEFAULT: u32 = xz_core::common::string_conversion::LZMA_PRESET_DEFAULT as u32;
pub const LZMA_PRESET_LEVEL_MASK: u32 =
    xz_core::lzma::lzma_encoder_presets::LZMA_PRESET_LEVEL_MASK as u32;
pub const LZMA_PRESET_EXTREME: u32 = xz_core::types::LZMA_PRESET_EXTREME as u32;
pub const LZMA_DICT_SIZE_MIN: u32 = xz_core::types::LZMA_DICT_SIZE_MIN as u32;
pub const LZMA_DICT_SIZE_DEFAULT: u32 =
    xz_core::common::string_conversion::LZMA_DICT_SIZE_DEFAULT as u32;
pub const LZMA_LCLP_MIN: u32 = xz_core::common::string_conversion::LZMA_LCLP_MIN as u32;
pub const LZMA_LCLP_MAX: u32 = xz_core::types::LZMA_LCLP_MAX as u32;
pub const LZMA_LC_DEFAULT: u32 = xz_core::lzma::lzma_encoder_presets::LZMA_LC_DEFAULT as u32;
pub const LZMA_LP_DEFAULT: u32 = xz_core::lzma::lzma_encoder_presets::LZMA_LP_DEFAULT as u32;
pub const LZMA_PB_MIN: u32 = xz_core::common::string_conversion::LZMA_PB_MIN as u32;
pub const LZMA_PB_MAX: u32 = xz_core::types::LZMA_PB_MAX as u32;
pub const LZMA_PB_DEFAULT: u32 = xz_core::lzma::lzma_encoder_presets::LZMA_PB_DEFAULT as u32;

/*********************
 * Advanced Features *
 *********************/

/* `lzma/stream_flags.h`: stream header size */
pub const LZMA_STREAM_HEADER_SIZE: u32 = xz_core::types::LZMA_STREAM_HEADER_SIZE as u32;
pub const LZMA_BLOCK_HEADER_SIZE_MIN: u32 =
    xz_core::common::block_util::LZMA_BLOCK_HEADER_SIZE_MIN as u32;
pub const LZMA_BLOCK_HEADER_SIZE_MAX: u32 = xz_core::types::LZMA_BLOCK_HEADER_SIZE_MAX as u32;

/*******************
 * Function Wrappers
 *******************/

/*
 * Wrappers follow `lzma.h`'s public subheader layout where practical.
 * They cast to canonical Rust implementation types and forward the call.
 */

/******************
 * Basic Features *
 ******************/

/* `lzma/version.h` */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_version_number() -> u32 {
    xz_core::common::common::lzma_version_number()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_version_string() -> *const c_char {
    xz_core::common::common::lzma_version_string()
}

/* `lzma/base.h`: common stream API */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_code(strm: *mut lzma_stream, action: lzma_action) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::common::lzma_code(strm.cast(), action)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_end(strm: *mut lzma_stream) {
    normalize_c_stream_allocator(strm);
    xz_core::common::common::lzma_end(strm.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_memlimit_get(strm: *const lzma_stream) -> u64 {
    // Read-only accessor: never allocates, so the allocator is left as-is.
    xz_core::common::common::lzma_memlimit_get(strm.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_memlimit_set(strm: *mut lzma_stream, new_memlimit: u64) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::common::lzma_memlimit_set(strm.cast(), new_memlimit)
}

/* `lzma/base.h`: allocation helpers */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_alloc(size: size_t, allocator: *const lzma_allocator) -> *mut c_void {
    xz_core::common::common::lzma_alloc(size, normalize_c_allocator(allocator))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_alloc_zero(
    size: size_t,
    allocator: *const lzma_allocator,
) -> *mut c_void {
    xz_core::common::common::lzma_alloc_zero(size, normalize_c_allocator(allocator))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_free(ptr: *mut c_void, allocator: *const lzma_allocator) {
    xz_core::common::common::lzma_free(ptr, normalize_c_allocator(allocator))
}

/* `lzma/base.h`: progress / memusage */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_get_progress(
    strm: *mut lzma_stream,
    progress_in: *mut u64,
    progress_out: *mut u64,
) {
    normalize_c_stream_allocator(strm);
    xz_core::common::common::lzma_get_progress(strm.cast(), &mut *progress_in, &mut *progress_out)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_memusage(strm: *const lzma_stream) -> u64 {
    // Read-only accessor: never allocates, so the allocator is left as-is.
    xz_core::common::common::lzma_memusage(strm.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_get_check(strm: *const lzma_stream) -> lzma_check {
    // Read-only accessor: never allocates, so the allocator is left as-is.
    xz_core::common::common::lzma_get_check(strm.cast())
}

/* `lzma/vli.h` */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_vli_encode(
    vli: lzma_vli,
    vli_pos: *mut size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::vli_encoder::lzma_vli_encode(
        vli,
        vli_pos.as_mut(),
        c_slice_mut(out, out_size),
        &mut *out_pos,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_vli_decode(
    vli: *mut lzma_vli,
    vli_pos: *mut size_t,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
) -> lzma_ret {
    xz_core::common::vli_decoder::lzma_vli_decode(
        &mut *vli,
        vli_pos.as_mut(),
        c_slice(input, in_size),
        &mut *in_pos,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_vli_size(vli: lzma_vli) -> u32 {
    xz_core::common::vli_size::lzma_vli_size(vli)
}

/* `lzma/check.h` */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_check_is_supported(check: lzma_check) -> lzma_bool {
    xz_core::check::check::lzma_check_is_supported(check)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_check_size(check: lzma_check) -> u32 {
    xz_core::check::check::lzma_check_size(check)
}

/// xz-core takes buffers as slices. C hands over a pointer and a size, and
/// allows the pointer to be NULL when the size is zero, which `from_raw_parts`
/// does not. This is the boundary where that contract stops being checkable,
/// so it is reconstructed here rather than carried into xz-core.
///
/// # Safety
/// `buf` must be readable for `size` bytes, or `size` must be zero.
unsafe fn c_slice<'a>(buf: *const u8, size: size_t) -> &'a [u8] {
    if size == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(buf, size)
    }
}

/// Mutable form of [`c_slice`].
///
/// # Safety
/// `buf` must be writable for `size` bytes, or `size` must be zero.
unsafe fn c_slice_mut<'a>(buf: *mut u8, size: size_t) -> &'a mut [u8] {
    if size == 0 {
        &mut []
    } else {
        core::slice::from_raw_parts_mut(buf, size)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_crc32(buf: *const u8, size: size_t, crc: u32) -> u32 {
    xz_core::check::crc32_fast::crc32(c_slice(buf, size), crc)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_crc64(buf: *const u8, size: size_t, crc: u64) -> u64 {
    xz_core::check::crc64_fast::crc64(c_slice(buf, size), crc)
}

/*********************
 * Container Formats *
 *********************/

/* `lzma/container.h`: easy encoder */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_easy_encoder_memusage(preset: u32) -> u64 {
    xz_core::common::easy_encoder_memusage::lzma_easy_encoder_memusage(preset)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_easy_decoder_memusage(preset: u32) -> u64 {
    xz_core::common::easy_decoder_memusage::lzma_easy_decoder_memusage(preset)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_easy_encoder(
    strm: *mut lzma_stream,
    preset: u32,
    check: lzma_check,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::easy_encoder::lzma_easy_encoder(strm.cast(), preset, check)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_easy_buffer_encode(
    preset: u32,
    check: lzma_check,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::easy_buffer_encoder::lzma_easy_buffer_encode(
        preset,
        check,
        normalize_c_allocator(allocator).cast(),
        input,
        in_size,
        out,
        out_pos,
        out_size,
    )
}

/* `lzma/container.h`: stream encoder / decoder */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_encoder(
    strm: *mut lzma_stream,
    filters: *const lzma_filter,
    check: lzma_check,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::stream_encoder::lzma_stream_encoder(strm.cast(), filters.cast(), check)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_decoder(
    strm: *mut lzma_stream,
    memlimit: u64,
    flags: u32,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::stream_decoder::lzma_stream_decoder(strm.cast(), memlimit, flags)
}

/* `lzma/container.h`: auto decoder */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_auto_decoder(
    strm: *mut lzma_stream,
    memlimit: u64,
    flags: u32,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::auto_decoder::lzma_auto_decoder(strm.cast(), memlimit, flags)
}

/* `lzma/container.h`: alone encoder / decoder */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_alone_encoder(
    strm: *mut lzma_stream,
    options: *const lzma_options_lzma,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::alone_encoder::lzma_alone_encoder(strm.cast(), options.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_alone_decoder(strm: *mut lzma_stream, memlimit: u64) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::alone_decoder::lzma_alone_decoder(strm.cast(), memlimit)
}

/* `lzma/container.h`: lzip decoder */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_lzip_decoder(
    strm: *mut lzma_stream,
    memlimit: u64,
    flags: u32,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::lzip_decoder::lzma_lzip_decoder(strm.cast(), memlimit, flags)
}

/* `lzma/container.h`: stream buffer */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_buffer_bound(uncompressed_size: size_t) -> size_t {
    xz_core::common::stream_buffer_encoder::lzma_stream_buffer_bound(uncompressed_size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_buffer_encode(
    filters: *mut lzma_filter,
    check: lzma_check,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::stream_buffer_encoder::lzma_stream_buffer_encode(
        filters.cast(),
        check,
        normalize_c_allocator(allocator).cast(),
        input,
        in_size,
        out,
        out_pos,
        out_size,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_buffer_decode(
    memlimit: *mut u64,
    flags: u32,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::stream_buffer_decoder::lzma_stream_buffer_decode(
        memlimit,
        flags,
        normalize_c_allocator(allocator).cast(),
        input,
        in_pos,
        in_size,
        out,
        out_pos,
        out_size,
    )
}

/***********
 * Filters *
 ***********/

/* `lzma/filter.h` */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_filter_encoder_is_supported(id: lzma_vli) -> lzma_bool {
    xz_core::common::filter_encoder::lzma_filter_encoder_is_supported(id)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_filter_decoder_is_supported(id: lzma_vli) -> lzma_bool {
    xz_core::common::filter_decoder::lzma_filter_decoder_is_supported(id)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_filters_copy(
    src: *const lzma_filter,
    dest: *mut lzma_filter,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    xz_core::common::filter_common::lzma_filters_copy(
        src.cast(),
        dest.cast(),
        normalize_c_allocator(allocator).cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_raw_encoder_memusage(filters: *const lzma_filter) -> u64 {
    xz_core::common::filter_encoder::lzma_raw_encoder_memusage(filters.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_raw_decoder_memusage(filters: *const lzma_filter) -> u64 {
    xz_core::common::filter_decoder::lzma_raw_decoder_memusage(filters.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_raw_encoder(
    strm: *mut lzma_stream,
    filters: *const lzma_filter,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::filter_encoder::lzma_raw_encoder(strm.cast(), filters.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_raw_decoder(
    strm: *mut lzma_stream,
    filters: *const lzma_filter,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::filter_decoder::lzma_raw_decoder(strm.cast(), filters.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_filters_update(
    strm: *mut lzma_stream,
    filters: *const lzma_filter,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::filter_encoder::lzma_filters_update(strm.cast(), filters.cast())
}

/* `lzma/filter.h`: raw buffer */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_raw_buffer_encode(
    filters: *const lzma_filter,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::filter_buffer_encoder::lzma_raw_buffer_encode(
        filters.cast(),
        normalize_c_allocator(allocator).cast(),
        input,
        in_size,
        out,
        out_pos,
        out_size,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_raw_buffer_decode(
    filters: *const lzma_filter,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::filter_buffer_decoder::lzma_raw_buffer_decode(
        filters.cast(),
        normalize_c_allocator(allocator).cast(),
        input,
        in_pos,
        in_size,
        out,
        out_pos,
        out_size,
    )
}

/* `lzma/filter.h`: properties */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_properties_size(
    size: *mut u32,
    filter: *const lzma_filter,
) -> lzma_ret {
    xz_core::common::filter_encoder::lzma_properties_size(&mut *size, &*filter.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_properties_encode(
    filter: *const lzma_filter,
    props: *mut u8,
) -> lzma_ret {
    // C gives `props` no length; the caller is required to have sized it with
    // lzma_properties_size, so that is what reconstructs the slice here.
    let filter = &*filter.cast();
    let mut props_size: u32 = 0;
    let ret = xz_core::common::filter_encoder::lzma_properties_size(&mut props_size, filter);
    if ret != LZMA_OK {
        return ret;
    }
    xz_core::common::filter_encoder::lzma_properties_encode(
        filter,
        c_slice_mut(props, props_size as size_t),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_properties_decode(
    filter: *mut lzma_filter,
    allocator: *const lzma_allocator,
    props: *const u8,
    props_size: size_t,
) -> lzma_ret {
    xz_core::common::filter_decoder::lzma_properties_decode(
        &mut *filter.cast(),
        normalize_c_allocator(allocator).cast(),
        c_slice(props, props_size),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_mt_block_size(filters: *const lzma_filter) -> u64 {
    xz_core::common::filter_encoder::lzma_mt_block_size(filters.cast())
}

/* `lzma/lzma12.h` */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_lzma_preset(
    options: *mut lzma_options_lzma,
    preset: u32,
) -> lzma_bool {
    xz_core::lzma::lzma_encoder_presets::lzma_lzma_preset(&mut *options.cast(), preset)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_mf_is_supported(mf: lzma_match_finder) -> lzma_bool {
    xz_core::lz::lz_encoder::lzma_mf_is_supported(mf)
}

#[unsafe(no_mangle)]
pub extern "C" fn lzma_mode_is_supported(mode: lzma_mode) -> lzma_bool {
    xz_core::lzma::lzma_encoder::lzma_mode_is_supported(mode)
}

/* `lzma/bcj.h` */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_bcj_arm64_encode(
    start_offset: u32,
    buf: *mut u8,
    size: size_t,
) -> size_t {
    xz_core::simple::arm64::lzma_bcj_arm64_encode(start_offset, buf, size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_bcj_arm64_decode(
    start_offset: u32,
    buf: *mut u8,
    size: size_t,
) -> size_t {
    xz_core::simple::arm64::lzma_bcj_arm64_decode(start_offset, buf, size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_bcj_riscv_encode(
    start_offset: u32,
    buf: *mut u8,
    size: size_t,
) -> size_t {
    xz_core::simple::riscv::lzma_bcj_riscv_encode(start_offset, buf, size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_bcj_riscv_decode(
    start_offset: u32,
    buf: *mut u8,
    size: size_t,
) -> size_t {
    xz_core::simple::riscv::lzma_bcj_riscv_decode(start_offset, buf, size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_bcj_x86_encode(
    start_offset: u32,
    buf: *mut u8,
    size: size_t,
) -> size_t {
    xz_core::simple::x86::lzma_bcj_x86_encode(start_offset, buf, size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_bcj_x86_decode(
    start_offset: u32,
    buf: *mut u8,
    size: size_t,
) -> size_t {
    xz_core::simple::x86::lzma_bcj_x86_decode(start_offset, buf, size)
}

/*********************
 * Advanced Features *
 *********************/

/* `lzma/stream_flags.h` */

#[unsafe(no_mangle)]
// The four Stream Header/Footer entry points read or write exactly
// LZMA_STREAM_HEADER_SIZE bytes. xz-core states that in the type; at this ABI
// boundary it is the caller's promise, as it is in C.
pub unsafe extern "C" fn lzma_stream_header_encode(
    options: *const lzma_stream_flags,
    out: *mut u8,
) -> lzma_ret {
    xz_core::common::stream_flags_encoder::lzma_stream_header_encode(
        &*options.cast(),
        &mut *out.cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_footer_encode(
    options: *const lzma_stream_flags,
    out: *mut u8,
) -> lzma_ret {
    xz_core::common::stream_flags_encoder::lzma_stream_footer_encode(
        &*options.cast(),
        &mut *out.cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_header_decode(
    options: *mut lzma_stream_flags,
    input: *const u8,
) -> lzma_ret {
    xz_core::common::stream_flags_decoder::lzma_stream_header_decode(
        &mut *options.cast(),
        &*input.cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_footer_decode(
    options: *mut lzma_stream_flags,
    input: *const u8,
) -> lzma_ret {
    xz_core::common::stream_flags_decoder::lzma_stream_footer_decode(
        &mut *options.cast(),
        &*input.cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_flags_compare(
    a: *const lzma_stream_flags,
    b: *const lzma_stream_flags,
) -> lzma_ret {
    xz_core::common::stream_flags_common::lzma_stream_flags_compare(&*a.cast(), &*b.cast())
}

/* `lzma/hardware.h` */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_physmem() -> u64 {
    xz_core::common::hardware_physmem::lzma_physmem()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_cputhreads() -> u32 {
    xz_core::common::hardware_cputhreads::lzma_cputhreads()
}

/* `lzma/index.h`: core index operations */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_buffer_decode(
    i: *mut *mut lzma_index,
    memlimit: *mut u64,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
) -> lzma_ret {
    xz_core::common::index_decoder::lzma_index_buffer_decode(
        i.cast(),
        memlimit,
        normalize_c_allocator(allocator).cast(),
        input,
        in_pos,
        in_size,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_uncompressed_size(i: *const lzma_index) -> lzma_vli {
    xz_core::common::index::lzma_index_uncompressed_size(&*i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_end(i: *mut lzma_index, allocator: *const lzma_allocator) {
    xz_core::common::index::lzma_index_end(i.cast(), normalize_c_allocator(allocator).cast())
}

/* `lzma/block.h` */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_header_size(block: *mut lzma_block) -> lzma_ret {
    xz_core::common::block_header_encoder::lzma_block_header_size(&mut *block.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_header_encode(
    block: *const lzma_block,
    out: *mut u8,
) -> lzma_ret {
    // C gives `out` no length; the Block Header is header_size bytes.
    let block: &xz_core::types::lzma_block = &*block.cast();
    xz_core::common::block_header_encoder::lzma_block_header_encode(
        block,
        c_slice_mut(out, block.header_size as size_t),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_header_decode(
    block: *mut lzma_block,
    allocator: *const lzma_allocator,
    input: *const u8,
) -> lzma_ret {
    // C gives `input` no length; the Block Header is header_size bytes. The
    // null tests stay here because a C caller can still pass NULL.
    if block.is_null() || input.is_null() {
        return LZMA_PROG_ERROR;
    }
    let block: &mut xz_core::types::lzma_block = &mut *block.cast();
    let header_size = block.header_size as size_t;
    xz_core::common::block_header_decoder::lzma_block_header_decode(
        block,
        normalize_c_allocator(allocator).cast(),
        c_slice(input, header_size),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_compressed_size(
    block: *mut lzma_block,
    unpadded_size: lzma_vli,
) -> lzma_ret {
    xz_core::common::block_util::lzma_block_compressed_size(block.cast(), unpadded_size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_unpadded_size(block: *const lzma_block) -> lzma_vli {
    xz_core::common::block_util::lzma_block_unpadded_size(block.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_total_size(block: *const lzma_block) -> lzma_vli {
    xz_core::common::block_util::lzma_block_total_size(block.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_encoder(
    strm: *mut lzma_stream,
    block: *mut lzma_block,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::block_encoder::lzma_block_encoder(strm.cast(), block.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_decoder(
    strm: *mut lzma_stream,
    block: *mut lzma_block,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::block_decoder::lzma_block_decoder(strm.cast(), block.cast())
}

#[unsafe(no_mangle)]
pub extern "C" fn lzma_block_buffer_bound(uncompressed_size: size_t) -> size_t {
    xz_core::common::block_buffer_encoder::lzma_block_buffer_bound(uncompressed_size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_buffer_encode(
    block: *mut lzma_block,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::block_buffer_encoder::lzma_block_buffer_encode(
        block.cast(),
        normalize_c_allocator(allocator).cast(),
        input,
        in_size,
        out,
        out_pos,
        out_size,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_uncomp_encode(
    block: *mut lzma_block,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::block_buffer_encoder::lzma_block_uncomp_encode(
        block.cast(),
        input,
        in_size,
        out,
        out_pos,
        out_size,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_block_buffer_decode(
    block: *mut lzma_block,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::block_buffer_decoder::lzma_block_buffer_decode(
        block.cast(),
        normalize_c_allocator(allocator).cast(),
        input,
        in_pos,
        in_size,
        out,
        out_pos,
        out_size,
    )
}

/* `lzma/index.h`: extended index operations */

#[unsafe(no_mangle)]
pub extern "C" fn lzma_index_memusage(streams: lzma_vli, blocks: lzma_vli) -> u64 {
    xz_core::common::index::lzma_index_memusage(streams, blocks)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_memused(i: *const lzma_index) -> u64 {
    xz_core::common::index::lzma_index_memused(&*i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_init(allocator: *const lzma_allocator) -> *mut lzma_index {
    xz_core::common::index::lzma_index_init(normalize_c_allocator(allocator).cast()).cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_append(
    i: *mut lzma_index,
    allocator: *const lzma_allocator,
    unpadded_size: lzma_vli,
    uncompressed_size: lzma_vli,
) -> lzma_ret {
    xz_core::common::index::lzma_index_append(
        i.cast(),
        normalize_c_allocator(allocator).cast(),
        unpadded_size,
        uncompressed_size,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_stream_flags(
    i: *mut lzma_index,
    stream_flags: *const lzma_stream_flags,
) -> lzma_ret {
    xz_core::common::index::lzma_index_stream_flags(i.cast(), stream_flags.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_checks(i: *const lzma_index) -> u32 {
    xz_core::common::index::lzma_index_checks(&*i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_stream_padding(
    i: *mut lzma_index,
    stream_padding: lzma_vli,
) -> lzma_ret {
    xz_core::common::index::lzma_index_stream_padding(i.cast(), stream_padding)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_stream_count(i: *const lzma_index) -> lzma_vli {
    xz_core::common::index::lzma_index_stream_count(&*i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_block_count(i: *const lzma_index) -> lzma_vli {
    xz_core::common::index::lzma_index_block_count(&*i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_size(i: *const lzma_index) -> lzma_vli {
    xz_core::common::index::lzma_index_size(&*i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_stream_size(i: *const lzma_index) -> lzma_vli {
    xz_core::common::index::lzma_index_stream_size(&*i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_total_size(i: *const lzma_index) -> lzma_vli {
    xz_core::common::index::lzma_index_total_size(&*i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_file_size(i: *const lzma_index) -> lzma_vli {
    xz_core::common::index::lzma_index_file_size(&*i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_iter_init(iter: *mut lzma_index_iter, i: *const lzma_index) {
    xz_core::common::index::lzma_index_iter_init(&mut *iter.cast(), &*i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_iter_rewind(iter: *mut lzma_index_iter) {
    xz_core::common::index::lzma_index_iter_rewind(&mut *iter.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_iter_next(
    iter: *mut lzma_index_iter,
    mode: lzma_index_iter_mode,
) -> lzma_bool {
    xz_core::common::index::lzma_index_iter_next(&mut *iter.cast(), mode)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_iter_locate(
    iter: *mut lzma_index_iter,
    target: lzma_vli,
) -> lzma_bool {
    xz_core::common::index::lzma_index_iter_locate(&mut *iter.cast(), target)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_cat(
    dest: *mut lzma_index,
    src: *mut lzma_index,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    xz_core::common::index::lzma_index_cat(
        dest.cast(),
        src.cast(),
        normalize_c_allocator(allocator).cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_dup(
    i: *const lzma_index,
    allocator: *const lzma_allocator,
) -> *mut lzma_index {
    xz_core::common::index::lzma_index_dup(i.cast(), normalize_c_allocator(allocator).cast()).cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_encoder(
    strm: *mut lzma_stream,
    i: *const lzma_index,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::index_encoder::lzma_index_encoder(strm.cast(), i.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_decoder(
    strm: *mut lzma_stream,
    i: *mut *mut lzma_index,
    memlimit: u64,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::index_decoder::lzma_index_decoder(strm.cast(), i.cast(), memlimit)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_buffer_encode(
    i: *const lzma_index,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::index_encoder::lzma_index_buffer_encode(i.cast(), out, out_pos, out_size)
}

/* `lzma/index_hash.h` */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_hash_init(
    index_hash: *mut lzma_index_hash,
    allocator: *const lzma_allocator,
) -> *mut lzma_index_hash {
    xz_core::common::index_hash::lzma_index_hash_init(
        index_hash.cast(),
        normalize_c_allocator(allocator).cast(),
    )
    .cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_hash_end(
    index_hash: *mut lzma_index_hash,
    allocator: *const lzma_allocator,
) {
    xz_core::common::index_hash::lzma_index_hash_end(
        index_hash.cast(),
        normalize_c_allocator(allocator).cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_hash_append(
    index_hash: *mut lzma_index_hash,
    unpadded_size: lzma_vli,
    uncompressed_size: lzma_vli,
) -> lzma_ret {
    xz_core::common::index_hash::lzma_index_hash_append(
        index_hash.cast(),
        unpadded_size,
        uncompressed_size,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_hash_decode(
    index_hash: *mut lzma_index_hash,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
) -> lzma_ret {
    xz_core::common::index_hash::lzma_index_hash_decode(index_hash.cast(), input, in_pos, in_size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_index_hash_size(index_hash: *const lzma_index_hash) -> lzma_vli {
    xz_core::common::index_hash::lzma_index_hash_size(&*index_hash.cast())
}

/* `lzma/filter.h`: filter flags / string conversion */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_filter_flags_size(
    size: *mut u32,
    filter: *const lzma_filter,
) -> lzma_ret {
    xz_core::common::filter_flags_encoder::lzma_filter_flags_size(&mut *size, &*filter.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_filter_flags_encode(
    filter: *const lzma_filter,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    xz_core::common::filter_flags_encoder::lzma_filter_flags_encode(
        &*filter.cast(),
        c_slice_mut(out, out_size),
        &mut *out_pos,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_filter_flags_decode(
    filter: *mut lzma_filter,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
) -> lzma_ret {
    xz_core::common::filter_flags_decoder::lzma_filter_flags_decode(
        &mut *filter.cast(),
        normalize_c_allocator(allocator).cast(),
        c_slice(input, in_size),
        &mut *in_pos,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_filters_free(
    filters: *mut lzma_filter,
    allocator: *const lzma_allocator,
) {
    xz_core::common::filter_common::lzma_filters_free(
        filters.cast(),
        normalize_c_allocator(allocator).cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_str_to_filters(
    str_: *const c_char,
    error_pos: *mut c_int,
    filters: *mut lzma_filter,
    flags: u32,
    allocator: *const lzma_allocator,
) -> *const c_char {
    xz_core::common::string_conversion::lzma_str_to_filters(
        str_,
        error_pos,
        filters.cast(),
        flags,
        normalize_c_allocator(allocator).cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_str_from_filters(
    str_: *mut *mut c_char,
    filters: *const lzma_filter,
    flags: u32,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    xz_core::common::string_conversion::lzma_str_from_filters(
        str_,
        filters.cast(),
        flags,
        normalize_c_allocator(allocator).cast(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_str_list_filters(
    str_: *mut *mut c_char,
    filter_id: lzma_vli,
    flags: u32,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    xz_core::common::string_conversion::lzma_str_list_filters(
        str_,
        filter_id,
        flags,
        normalize_c_allocator(allocator).cast(),
    )
}

/* `lzma/container.h`: additional container helpers */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_microlzma_encoder(
    strm: *mut lzma_stream,
    options: *const lzma_options_lzma,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::microlzma_encoder::lzma_microlzma_encoder(strm.cast(), options.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_microlzma_decoder(
    strm: *mut lzma_stream,
    comp_size: u64,
    uncomp_size: u64,
    uncomp_size_is_exact: lzma_bool,
    dict_size: u32,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::microlzma_decoder::lzma_microlzma_decoder(
        strm.cast(),
        comp_size,
        uncomp_size,
        uncomp_size_is_exact,
        dict_size,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_file_info_decoder(
    strm: *mut lzma_stream,
    i: *mut *mut lzma_index,
    memlimit: u64,
    file_size: u64,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::file_info::lzma_file_info_decoder(strm.cast(), i.cast(), memlimit, file_size)
}

/*********************
 * Container Formats *
 *********************/

/* `lzma/container.h`: multithreaded API */

#[cfg(feature = "parallel")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_encoder_mt(
    strm: *mut lzma_stream,
    options: *const lzma_mt,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::stream_mt::lzma_stream_encoder_mt(strm.cast(), options.cast())
}

#[cfg(feature = "parallel")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_decoder_mt(
    strm: *mut lzma_stream,
    options: *const lzma_mt,
) -> lzma_ret {
    normalize_c_stream_allocator(strm);
    xz_core::common::stream_mt::lzma_stream_decoder_mt(strm.cast(), options.cast())
}

#[cfg(feature = "parallel")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lzma_stream_encoder_mt_memusage(options: *const lzma_mt) -> u64 {
    xz_core::common::stream_mt::lzma_stream_encoder_mt_memusage(options.cast())
}
