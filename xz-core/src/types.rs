pub use std::os::raw::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_ulonglong, c_void};
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub type size_t = usize;
#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub type size_t = libc::size_t;
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub type uintptr_t = usize;
#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub type uintptr_t = libc::uintptr_t;
#[cfg(target_env = "msvc")]
pub type __enum_ty = c_int;
#[cfg(not(target_env = "msvc"))]
pub type __enum_ty = c_uint;
pub type lzma_bool = c_uchar;
pub type lzma_ret = __enum_ty;
pub type lzma_action = __enum_ty;
pub type lzma_check = __enum_ty;
pub type lzma_vli = u64;
pub type lzma_reserved_enum = __enum_ty;
pub type lzma_mode = __enum_ty;
pub type lzma_match_finder = __enum_ty;
pub type lzma_lzma_state = c_uint;
pub type lzma_delta_type = c_uint;
pub type probability = u16;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_allocator {
    pub alloc: Option<unsafe extern "C" fn(*mut c_void, size_t, size_t) -> *mut c_void>,
    pub free: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    pub opaque: *mut c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_filter {
    pub id: lzma_vli,
    pub options: *mut c_void,
}
pub use crate::common::common_types::{
    ISEQ_END, ISEQ_ERROR, ISEQ_FINISH, ISEQ_FULL_BARRIER, ISEQ_FULL_FLUSH, ISEQ_RUN,
    ISEQ_SYNC_FLUSH, LZMA_MEMUSAGE_BASE, LZMA_SUPPORTED_FLAGS, LZMA_THREADS_MAX,
    lzma_code_function, lzma_end_function, lzma_filter_info, lzma_filter_info_s,
    lzma_init_function, lzma_internal, lzma_internal_s, lzma_internal_sequence, lzma_next_coder,
    lzma_next_coder_s,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_options_lzma {
    pub dict_size: u32,
    pub preset_dict: *const u8,
    pub preset_dict_size: u32,
    pub lc: u32,
    pub lp: u32,
    pub pb: u32,
    pub mode: lzma_mode,
    pub nice_len: u32,
    pub mf: lzma_match_finder,
    pub depth: u32,
    pub ext_flags: u32,
    pub ext_size_low: u32,
    pub ext_size_high: u32,
    pub reserved_int4: u32,
    pub reserved_int5: u32,
    pub reserved_int6: u32,
    pub reserved_int7: u32,
    pub reserved_int8: u32,
    pub reserved_enum1: lzma_reserved_enum,
    pub reserved_enum2: lzma_reserved_enum,
    pub reserved_enum3: lzma_reserved_enum,
    pub reserved_enum4: lzma_reserved_enum,
    pub reserved_ptr1: *mut c_void,
    pub reserved_ptr2: *mut c_void,
}
#[derive(Clone)]
#[repr(C)]
pub struct lzma_stream {
    pub next_in: *const u8,
    pub avail_in: size_t,
    pub total_in: u64,
    pub next_out: *mut u8,
    pub avail_out: size_t,
    pub total_out: u64,
    // Only xz-sys uses the allocator; without custom_allocator the field
    // is omitted for a leaner build, diverging from the C lzma_stream
    // layout on purpose.
    #[cfg(feature = "custom_allocator")]
    pub allocator: *const lzma_allocator,
    pub internal: *mut lzma_internal,
    pub reserved_ptr1: *mut c_void,
    pub reserved_ptr2: *mut c_void,
    pub reserved_ptr3: *mut c_void,
    pub reserved_ptr4: *mut c_void,
    pub seek_pos: u64,
    pub reserved_int2: u64,
    pub reserved_int3: size_t,
    pub reserved_int4: size_t,
    pub reserved_enum1: lzma_reserved_enum,
    pub reserved_enum2: lzma_reserved_enum,
}
pub const LZMA_OK: lzma_ret = 0;
pub const LZMA_STREAM_END: lzma_ret = 1;
pub const LZMA_NO_CHECK: lzma_ret = 2;
pub const LZMA_UNSUPPORTED_CHECK: lzma_ret = 3;
pub const LZMA_GET_CHECK: lzma_ret = 4;
pub const LZMA_MEM_ERROR: lzma_ret = 5;
pub const LZMA_MEMLIMIT_ERROR: lzma_ret = 6;
pub const LZMA_FORMAT_ERROR: lzma_ret = 7;
pub const LZMA_OPTIONS_ERROR: lzma_ret = 8;
pub const LZMA_DATA_ERROR: lzma_ret = 9;
pub const LZMA_BUF_ERROR: lzma_ret = 10;
pub const LZMA_PROG_ERROR: lzma_ret = 11;
pub const LZMA_SEEK_NEEDED: lzma_ret = 12;
pub const LZMA_RET_INTERNAL1: lzma_ret = 101;
pub const LZMA_RET_INTERNAL2: lzma_ret = 102;
pub const LZMA_RET_INTERNAL3: lzma_ret = 103;
pub const LZMA_RET_INTERNAL4: lzma_ret = 104;
pub const LZMA_RET_INTERNAL5: lzma_ret = 105;
pub const LZMA_RET_INTERNAL6: lzma_ret = 106;
pub const LZMA_RET_INTERNAL7: lzma_ret = 107;
pub const LZMA_RET_INTERNAL8: lzma_ret = 108;
pub const LZMA_RUN: lzma_action = 0;
pub const LZMA_SYNC_FLUSH: lzma_action = 1;
pub const LZMA_FULL_FLUSH: lzma_action = 2;
pub const LZMA_FINISH: lzma_action = 3;
pub const LZMA_FULL_BARRIER: lzma_action = 4;
pub const LZMA_CHECK_NONE: lzma_check = 0;
pub const LZMA_CHECK_CRC32: lzma_check = 1;
pub const LZMA_CHECK_CRC64: lzma_check = 4;
pub const LZMA_CHECK_SHA256: lzma_check = 10;
pub const LZMA_RESERVED_ENUM: lzma_reserved_enum = 0;
pub const UINT32_MAX: c_uint = u32::MAX;
pub const UINT64_MAX: u64 = u64::MAX;
pub const LZMA_VLI_MAX: lzma_vli = u64::MAX / 2;
pub const LZMA_VLI_UNKNOWN: lzma_vli = u64::MAX;
pub const LZMA_MF_HC3: lzma_match_finder = 3;
pub const LZMA_MF_HC4: lzma_match_finder = 4;
pub const LZMA_MF_BT2: lzma_match_finder = 18;
pub const LZMA_MF_BT3: lzma_match_finder = 19;
pub const LZMA_MF_BT4: lzma_match_finder = 20;
pub const LZMA_MODE_FAST: lzma_mode = 1;
pub const LZMA_MODE_NORMAL: lzma_mode = 2;
pub const LZMA_CHECK_ID_MAX: lzma_check = 15;
pub const LZMA_DELTA_TYPE_BYTE: lzma_delta_type = 0;
pub const LZMA_FILTER_DELTA: lzma_vli = 0x3;
pub const LZMA_FILTER_X86: lzma_vli = 0x4;
pub const LZMA_FILTER_POWERPC: lzma_vli = 0x5;
pub const LZMA_FILTER_IA64: lzma_vli = 0x6;
pub const LZMA_FILTER_ARM: lzma_vli = 0x7;
pub const LZMA_FILTER_ARMTHUMB: lzma_vli = 0x8;
pub const LZMA_FILTER_SPARC: lzma_vli = 0x9;
pub const LZMA_FILTER_ARM64: lzma_vli = 0xa;
pub const LZMA_FILTER_RISCV: lzma_vli = 0xb;
pub const LZMA_FILTER_LZMA1: lzma_vli = 0x4000000000000001;
pub const LZMA_FILTER_LZMA1EXT: lzma_vli = 0x4000000000000002;
pub const LZMA_FILTER_LZMA2: lzma_vli = 0x21;
pub const LZMA_FILTER_RESERVED_START: lzma_vli = 1 << 62;
pub const LZMA_TELL_NO_CHECK: c_uint = 0x1;
pub const LZMA_TELL_UNSUPPORTED_CHECK: c_uint = 0x2;
pub const LZMA_TELL_ANY_CHECK: c_uint = 0x4;
pub const LZMA_CONCATENATED: c_uint = 0x8;
pub const LZMA_IGNORE_CHECK: c_uint = 0x10;
pub const LZMA_FAIL_FAST: c_uint = 0x20;
pub const LZMA_STREAM_HEADER_SIZE: u32 = 12;
/// [`LZMA_STREAM_HEADER_SIZE`] as a `usize`, for use as a const generic
/// argument.
pub const STREAM_HEADER_SIZE: usize = LZMA_STREAM_HEADER_SIZE as usize;
pub const LZMA_BLOCK_HEADER_SIZE_MAX: u32 = 1024;
pub const LZMA_DICT_SIZE_MIN: c_uint = 4096;
pub const STATE_LIT_LIT: lzma_lzma_state = 0;
pub const STATE_MATCH_LIT_LIT: lzma_lzma_state = 1;
pub const STATE_REP_LIT_LIT: lzma_lzma_state = 2;
pub const STATE_SHORTREP_LIT_LIT: lzma_lzma_state = 3;
pub const STATE_MATCH_LIT: lzma_lzma_state = 4;
pub const STATE_REP_LIT: lzma_lzma_state = 5;
pub const STATE_SHORTREP_LIT: lzma_lzma_state = 6;
pub const STATE_LIT_MATCH: lzma_lzma_state = 7;
pub const STATE_LIT_LONGREP: lzma_lzma_state = 8;
pub const STATE_LIT_SHORTREP: lzma_lzma_state = 9;
pub const STATE_NONLIT_MATCH: lzma_lzma_state = 10;
pub const STATE_NONLIT_REP: lzma_lzma_state = 11;
pub const LZMA_VLI_BYTES_MAX: u32 = 9;
pub const LZMA_FILTERS_MAX: u32 = 4;
pub const LZMA_LCLP_MAX: u32 = 4;
pub const LZMA_PB_MAX: u32 = 4;
pub const LZMA_DELTA_DIST_MAX: u32 = 256;
pub const LZMA_BACKWARD_SIZE_MIN: u32 = 4;
pub const LZMA_BACKWARD_SIZE_MAX: u64 = 1 << 34;
// Match C's UINTPTR_MAX and SIZE_MAX macros. On LLP64 targets, size_t and
// uintptr_t are wider than unsigned long.
pub const UINTPTR_MAX: uintptr_t = uintptr_t::MAX;
pub const SIZE_MAX: size_t = size_t::MAX;
pub const INDEX_INDICATOR: u8 = 0;
pub const UNPADDED_SIZE_MIN: c_ulonglong = 5;
pub const UNPADDED_SIZE_MAX: c_ulonglong = LZMA_VLI_MAX & !3;
pub const LZMA_DELTA_DIST_MIN: u32 = 1;
pub const LZMA_LZMA1EXT_ALLOW_EOPM: c_uint = 0x1;
pub const RC_BIT_MODEL_TOTAL_BITS: u32 = 11;
pub const RC_BIT_MODEL_TOTAL: c_uint = 1u32 << RC_BIT_MODEL_TOTAL_BITS;
pub const MATCH_LEN_MIN: u32 = 2;
pub const REPS: u32 = 4;
pub const LIT_STATES: u32 = 7;
pub const FASTPOS_BITS: u32 = 13;
pub const OPTS: u32 = 1 << 12;
pub const LZ_DICT_REPEAT_MAX: u32 = 288;
pub const LZ_DICT_INIT_POS: u32 = 2 * LZ_DICT_REPEAT_MAX;
pub const ALIGN_BITS: u32 = 4;
pub const ALIGN_SIZE: u32 = 1 << ALIGN_BITS;
pub const ALIGN_MASK: u32 = ALIGN_SIZE - 1;
pub const DIST_STATES: u32 = 4;
pub const DIST_SLOT_BITS: u32 = 6;
pub const DIST_MODEL_START: u32 = 4;
pub const DIST_MODEL_END: u32 = 14;
pub const FULL_DISTANCES_BITS: u32 = DIST_MODEL_END / 2;
pub const FULL_DISTANCES: u32 = 1 << FULL_DISTANCES_BITS;
pub const STATES: u32 = 12;
pub const LITERAL_CODER_SIZE: c_uint = 0x300;
pub const LEN_LOW_BITS: u32 = 3;
pub const LEN_LOW_SYMBOLS: u32 = 1 << LEN_LOW_BITS;
pub const LEN_MID_BITS: u32 = 3;
pub const LEN_MID_SYMBOLS: u32 = 1 << LEN_MID_BITS;
pub const LEN_HIGH_BITS: u32 = 8;
pub const LEN_HIGH_SYMBOLS: u32 = 1 << LEN_HIGH_BITS;
pub const RC_MOVE_BITS: u32 = 5;
pub const RC_SHIFT_BITS: u32 = 8;
pub const RC_TOP_BITS: u32 = 24;
pub const RC_TOP_VALUE: c_uint = 1u32 << RC_TOP_BITS;
pub const RC_MOVE_REDUCING_BITS: u32 = 4;
pub const LZMA2_CHUNK_MAX: c_uint = 1u32 << 16;
pub const LZMA2_HEADER_UNCOMPRESSED: u32 = 3;
pub const HASH_2_SIZE: c_uint = 1u32 << 10;
pub const HASH_3_SIZE: c_uint = 1u32 << 16;
pub const LZMA_CHECK_SIZE_MAX: u32 = 64;
pub const LZMA_STREAM_FLAGS_SIZE: u32 = 2;
pub const LZMA_PRESET_EXTREME: c_uint = 1u32 << 31;
pub const COMPRESSED_SIZE_MAX: c_ulonglong = (LZMA_VLI_MAX
    .wrapping_sub(LZMA_BLOCK_HEADER_SIZE_MAX as u64)
    .wrapping_sub(LZMA_CHECK_SIZE_MAX as u64))
    & !3u64;
pub type worker_state = c_uint;
pub type lzma_index_iter_mode = c_uint;
pub const THR_IDLE: worker_state = 0;
pub const THR_RUN: worker_state = 1;
pub const LZMA_INDEX_ITER_ANY: lzma_index_iter_mode = 0;
pub const LZMA_INDEX_ITER_STREAM: lzma_index_iter_mode = 1;
pub const LZMA_INDEX_ITER_BLOCK: lzma_index_iter_mode = 2;
pub const LZMA_INDEX_ITER_NONEMPTY_BLOCK: lzma_index_iter_mode = 3;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_index_iter {
    pub stream: lzma_index_iter_stream,
    pub block: lzma_index_iter_block,
    pub internal: [lzma_index_iter_internal; 6],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union lzma_index_iter_internal {
    pub p: *const c_void,
    pub s: size_t,
    pub v: lzma_vli,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_index_iter_block {
    pub number_in_file: lzma_vli,
    pub compressed_file_offset: lzma_vli,
    pub uncompressed_file_offset: lzma_vli,
    pub number_in_stream: lzma_vli,
    pub compressed_stream_offset: lzma_vli,
    pub uncompressed_stream_offset: lzma_vli,
    pub uncompressed_size: lzma_vli,
    pub unpadded_size: lzma_vli,
    pub total_size: lzma_vli,
    pub reserved_vli1: lzma_vli,
    pub reserved_vli2: lzma_vli,
    pub reserved_vli3: lzma_vli,
    pub reserved_vli4: lzma_vli,
    pub reserved_ptr1: *const c_void,
    pub reserved_ptr2: *const c_void,
    pub reserved_ptr3: *const c_void,
    pub reserved_ptr4: *const c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_index_iter_stream {
    pub flags: *const lzma_stream_flags,
    pub reserved_ptr1: *const c_void,
    pub reserved_ptr2: *const c_void,
    pub reserved_ptr3: *const c_void,
    pub number: lzma_vli,
    pub block_count: lzma_vli,
    pub compressed_offset: lzma_vli,
    pub uncompressed_offset: lzma_vli,
    pub compressed_size: lzma_vli,
    pub uncompressed_size: lzma_vli,
    pub padding: lzma_vli,
    pub reserved_vli1: lzma_vli,
    pub reserved_vli2: lzma_vli,
    pub reserved_vli3: lzma_vli,
    pub reserved_vli4: lzma_vli,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_block {
    pub version: u32,
    pub header_size: u32,
    pub check: lzma_check,
    pub compressed_size: lzma_vli,
    pub uncompressed_size: lzma_vli,
    pub filters: *mut lzma_filter,
    pub raw_check: [u8; 64],
    pub reserved_ptr1: *mut c_void,
    pub reserved_ptr2: *mut c_void,
    pub reserved_ptr3: *mut c_void,
    pub reserved_int1: u32,
    pub reserved_int2: u32,
    pub reserved_int3: lzma_vli,
    pub reserved_int4: lzma_vli,
    pub reserved_int5: lzma_vli,
    pub reserved_int6: lzma_vli,
    pub reserved_int7: lzma_vli,
    pub reserved_int8: lzma_vli,
    pub reserved_enum1: lzma_reserved_enum,
    pub reserved_enum2: lzma_reserved_enum,
    pub reserved_enum3: lzma_reserved_enum,
    pub reserved_enum4: lzma_reserved_enum,
    pub ignore_check: lzma_bool,
    pub reserved_bool2: lzma_bool,
    pub reserved_bool3: lzma_bool,
    pub reserved_bool4: lzma_bool,
    pub reserved_bool5: lzma_bool,
    pub reserved_bool6: lzma_bool,
    pub reserved_bool7: lzma_bool,
    pub reserved_bool8: lzma_bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_stream_flags {
    pub version: u32,
    pub backward_size: lzma_vli,
    pub check: lzma_check,
    pub reserved_enum1: lzma_reserved_enum,
    pub reserved_enum2: lzma_reserved_enum,
    pub reserved_enum3: lzma_reserved_enum,
    pub reserved_enum4: lzma_reserved_enum,
    pub reserved_bool1: lzma_bool,
    pub reserved_bool2: lzma_bool,
    pub reserved_bool3: lzma_bool,
    pub reserved_bool4: lzma_bool,
    pub reserved_bool5: lzma_bool,
    pub reserved_bool6: lzma_bool,
    pub reserved_bool7: lzma_bool,
    pub reserved_bool8: lzma_bool,
    pub reserved_int1: u32,
    pub reserved_int2: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_sha256_state {
    pub state: [u32; 8],
    pub size: u64,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union lzma_check_state_buffer {
    pub u8_0: [u8; 64],
    pub u32_0: [u32; 16],
    pub u64_0: [u64; 8],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union lzma_check_state_inner {
    pub crc32: u32,
    pub crc64: u64,
    pub sha256: lzma_sha256_state,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_check_state {
    pub buffer: lzma_check_state_buffer,
    pub state: lzma_check_state_inner,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_options_delta {
    pub type_: lzma_delta_type,
    pub dist: u32,
    pub reserved_int1: u32,
    pub reserved_int2: u32,
    pub reserved_int3: u32,
    pub reserved_int4: u32,
    pub reserved_ptr1: *mut c_void,
    pub reserved_ptr2: *mut c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_options_easy {
    pub filters: [lzma_filter; 5],
    pub opt_lzma: lzma_options_lzma,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_dict {
    pub buf: *mut u8,
    pub pos: size_t,
    pub full: size_t,
    pub limit: size_t,
    pub size: size_t,
    pub has_wrapped: bool,
    pub need_reset: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_lz_decoder {
    pub coder: *mut c_void,
    pub code: lzma_lz_decoder_code_function,
    pub reset: Option<unsafe fn(*mut c_void, *const c_void) -> ()>,
    pub set_uncompressed: Option<unsafe fn(*mut c_void, lzma_vli, bool) -> ()>,
    pub end: Option<unsafe fn(*mut c_void, *const lzma_allocator) -> ()>,
}
pub type lzma_lz_decoder_code_function =
    unsafe fn(*mut c_void, *mut lzma_dict, *const u8, *mut size_t, size_t) -> lzma_ret;

#[cold]
pub unsafe fn lzma_lz_decoder_code_uninitialized(
    _coder: *mut c_void,
    _dict: *mut lzma_dict,
    _input: *const u8,
    _in_pos: *mut size_t,
    _in_size: size_t,
) -> lzma_ret {
    panic!("uninitialized LZ decoder callback")
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_match {
    pub len: u32,
    pub dist: u32,
}
pub type lzma_mf_find_function = unsafe fn(*mut lzma_mf, *mut lzma_match) -> u32;
pub type lzma_mf_skip_function = unsafe fn(*mut lzma_mf, u32) -> ();

#[cold]
pub unsafe fn lzma_mf_find_uninitialized(_mf: *mut lzma_mf, _matches: *mut lzma_match) -> u32 {
    panic!("uninitialized match finder callback")
}

#[cold]
pub unsafe fn lzma_mf_skip_uninitialized(_mf: *mut lzma_mf, _amount: u32) {
    panic!("uninitialized match finder skip callback")
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_mf_s {
    pub buffer: *mut u8,
    pub size: u32,
    pub keep_size_before: u32,
    pub keep_size_after: u32,
    pub offset: u32,
    pub read_pos: u32,
    pub read_ahead: u32,
    pub read_limit: u32,
    pub write_pos: u32,
    pub pending: u32,
    pub find: lzma_mf_find_function,
    pub skip: lzma_mf_skip_function,
    pub hash: *mut u32,
    pub son: *mut u32,
    pub cyclic_pos: u32,
    pub cyclic_size: u32,
    pub hash_mask: u32,
    pub depth: u32,
    pub nice_len: u32,
    pub match_len_max: u32,
    pub action: lzma_action,
    pub hash_count: u32,
    pub sons_count: u32,
}
pub type lzma_mf = lzma_mf_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_delta_coder {
    pub next: lzma_next_coder,
    pub distance: size_t,
    pub pos: u8,
    pub history: [u8; LZMA_DELTA_DIST_MAX as usize],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_lz_encoder {
    pub coder: *mut c_void,
    pub code: lzma_lz_encoder_code_function,
    pub end: Option<unsafe fn(*mut c_void, *const lzma_allocator) -> ()>,
    pub options_update: Option<unsafe fn(*mut c_void, *const lzma_filter) -> lzma_ret>,
    pub set_out_limit: Option<unsafe fn(*mut c_void, *mut u64, u64) -> lzma_ret>,
}
pub type lzma_lz_encoder_code_function =
    unsafe fn(*mut c_void, *mut lzma_mf, *mut u8, *mut size_t, size_t) -> lzma_ret;

#[cold]
pub unsafe fn lzma_lz_encoder_code_uninitialized(
    _coder: *mut c_void,
    _mf: *mut lzma_mf,
    _out: *mut u8,
    _out_pos: *mut size_t,
    _out_size: size_t,
) -> lzma_ret {
    panic!("uninitialized LZ encoder callback")
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_outbuf_s {
    pub next: *mut lzma_outbuf,
    pub worker: *mut c_void,
    pub allocated: size_t,
    pub pos: size_t,
    pub decoder_in_pos: size_t,
    pub finished: bool,
    pub finish_ret: lzma_ret,
    pub unpadded_size: lzma_vli,
    pub uncompressed_size: lzma_vli,
    pub buf: [u8; 0],
}
pub type lzma_outbuf = lzma_outbuf_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_outq {
    pub head: *mut lzma_outbuf,
    pub tail: *mut lzma_outbuf,
    pub read_pos: size_t,
    pub cache: *mut lzma_outbuf,
    pub mem_allocated: u64,
    pub mem_in_use: u64,
    pub bufs_in_use: u32,
    pub bufs_allocated: u32,
    pub bufs_limit: u32,
}
pub use crate::common::index::{lzma_index, lzma_index_s};
pub use crate::common::index_hash::{lzma_index_hash, lzma_index_hash_s};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_lzma1_encoder_s {
    pub rc: lzma_range_encoder,
    pub uncomp_size: u64,
    pub out_limit: u64,
    pub uncomp_size_ptr: *mut u64,
    pub state: lzma_lzma_state,
    pub reps: [u32; 4],
    pub matches: [lzma_match; 274],
    pub matches_count: u32,
    pub longest_match_length: u32,
    pub fast_mode: bool,
    pub is_initialized: bool,
    pub is_flushed: bool,
    pub use_eopm: bool,
    pub pos_mask: u32,
    pub literal_context_bits: u32,
    pub literal_mask: u32,
    pub literal: [probability; 12288],
    pub is_match: [[probability; 16]; 12],
    pub is_rep: [probability; 12],
    pub is_rep0: [probability; 12],
    pub is_rep1: [probability; 12],
    pub is_rep2: [probability; 12],
    pub is_rep0_long: [[probability; 16]; 12],
    pub dist_slot: [[probability; 64]; 4],
    pub dist_special: [probability; 114],
    pub dist_align: [probability; 16],
    pub match_len_encoder: lzma_length_encoder,
    pub rep_len_encoder: lzma_length_encoder,
    pub dist_slot_prices: [[u32; 64]; 4],
    pub dist_prices: [[u32; 128]; 4],
    pub dist_table_size: u32,
    pub match_price_count: u32,
    pub align_prices: [u32; 16],
    pub align_price_count: u32,
    pub opts_end_index: u32,
    pub opts_current_index: u32,
    pub opts: [lzma_optimal; OPTS as usize],
}
pub type lzma_lzma1_encoder = lzma_lzma1_encoder_s;
/// Fixed-size windows into a fixed-size buffer at compile-time offsets.
///
/// `OFF + M <= N` is proved when the method is instantiated, so the access has
/// no bound check and no panic path: the cast is discharged by the const
/// assertion rather than by a runtime test. `N` comes from the receiver, so a
/// caller names only the offset and the length.
pub trait FixedBuf<const N: usize> {
    fn subarray<const OFF: usize, const M: usize>(&self) -> &[u8; M];
    fn subarray_mut<const OFF: usize, const M: usize>(&mut self) -> &mut [u8; M];
}

impl<const N: usize> FixedBuf<N> for [u8; N] {
    #[inline]
    fn subarray<const OFF: usize, const M: usize>(&self) -> &[u8; M] {
        const { assert!(OFF + M <= N) };
        unsafe { &*self.as_ptr().add(OFF).cast::<[u8; M]>() }
    }
    #[inline]
    fn subarray_mut<const OFF: usize, const M: usize>(&mut self) -> &mut [u8; M] {
        const { assert!(OFF + M <= N) };
        unsafe { &mut *self.as_mut_ptr().add(OFF).cast::<[u8; M]>() }
    }
}

#[inline]
pub fn read32le(buf: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*buf)
}
#[inline]
pub fn write32le(buf: &mut [u8; 4], num: u32) {
    *buf = num.to_le_bytes();
}
#[inline]
pub fn vli_ceil4(vli: lzma_vli) -> lzma_vli {
    vli.wrapping_add(3) & !(3)
}
#[inline]
pub fn index_size_unpadded(count: lzma_vli, index_list_size: lzma_vli) -> lzma_vli {
    (1u32.wrapping_add(lzma_vli_size(count)) as lzma_vli)
        .wrapping_add(index_list_size)
        .wrapping_add(4)
}
#[inline]
pub fn lzma_outq_has_buf(outq: &lzma_outq) -> bool {
    outq.bufs_in_use < outq.bufs_limit
}
#[inline]
pub fn lzma_outq_is_empty(outq: &lzma_outq) -> bool {
    outq.bufs_in_use == 0
}
#[inline]
pub unsafe fn mf_ptr(mf: *const lzma_mf) -> *const u8 {
    (*mf).buffer.offset((*mf).read_pos as isize)
}
#[inline]
pub unsafe fn mf_avail(mf: *const lzma_mf) -> u32 {
    (*mf).write_pos.wrapping_sub((*mf).read_pos)
}
#[inline]
pub unsafe fn mf_skip(mf: *mut lzma_mf, amount: u32) {
    mf_skip_raw(mf, amount, (*mf).skip);
}

#[inline(always)]
pub unsafe fn mf_skip_raw(mf: *mut lzma_mf, amount: u32, skip: unsafe fn(*mut lzma_mf, u32) -> ()) {
    if amount != 0 {
        skip(mf, amount);
        (*mf).read_ahead = (*mf).read_ahead.wrapping_add(amount);
    }
}
// Buffers compared with lzma_memcmplen must allocate and zero this many
// extra tail bytes: the fast paths below may read up to this far past
// `limit`. Each cfg here is repeated verbatim on the matching branch of
// lzma_memcmplen, and every branch there asserts at compile time which
// value it needs, so editing one side without the other fails the build
// instead of silently reading out of bounds.
//
// The architecture lists follow TUKLIB_FAST_UNALIGNED_ACCESS, which C
// enables unconditionally on x86, x86-64, and PowerPC. ARM64 is included
// because unaligned access works on every ARM64 target Rust supports.
// 32-bit ARM, RISC-V, and LoongArch stay on the byte-at-a-time branch:
// C decides those from compiler feature macros that cfg cannot observe.

// 64-bit word compares.
#[cfg(all(
    target_pointer_width = "64",
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "powerpc64"
    )
))]
pub const LZMA_MEMCMPLEN_EXTRA: u32 = 8;

// SSE2 128-bit compares. x86-64 uses the 64-bit word branch above instead,
// like in C, so this is reached on 32-bit x86 and on x32.
#[cfg(all(
    any(
        target_arch = "x86",
        all(target_arch = "x86_64", target_pointer_width = "32")
    ),
    target_feature = "sse2"
))]
pub const LZMA_MEMCMPLEN_EXTRA: u32 = 16;

// 32-bit word compares.
#[cfg(any(
    all(target_arch = "x86", not(target_feature = "sse2")),
    target_arch = "powerpc"
))]
pub const LZMA_MEMCMPLEN_EXTRA: u32 = 4;

// Byte at a time; reads nothing past `limit`.
#[cfg(not(any(
    all(
        target_pointer_width = "64",
        any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "powerpc64"
        )
    ),
    all(
        any(
            target_arch = "x86",
            all(target_arch = "x86_64", target_pointer_width = "32")
        ),
        target_feature = "sse2"
    ),
    any(
        all(target_arch = "x86", not(target_feature = "sse2")),
        target_arch = "powerpc"
    )
)))]
pub const LZMA_MEMCMPLEN_EXTRA: u32 = 0;

#[inline(always)]
pub unsafe fn lzma_memcmplen(buf1: *const u8, buf2: *const u8, mut len: u32, limit: u32) -> u32 {
    debug_assert!(len <= limit);
    debug_assert!(limit <= u32::MAX / 2);

    #[cfg(all(
        target_pointer_width = "64",
        any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "powerpc64"
        )
    ))]
    {
        const _: () = assert!(LZMA_MEMCMPLEN_EXTRA == 8);
        while len < limit {
            let lhs = core::ptr::read_unaligned(buf1.add(len as usize) as *const u64);
            let rhs = core::ptr::read_unaligned(buf2.add(len as usize) as *const u64);
            // Little endian subtracts because sub+jnz fuses on more
            // processors than xor+jnz; big endian must xor so that the
            // first differing byte is the most significant one.
            let x = if cfg!(target_endian = "little") {
                lhs.wrapping_sub(rhs)
            } else {
                lhs ^ rhs
            };
            if x != 0 {
                let bits = if cfg!(target_endian = "little") {
                    x.trailing_zeros()
                } else {
                    x.leading_zeros()
                };
                return core::cmp::min(len + (bits >> 3), limit);
            }
            len += 8;
        }
        limit
    }

    // SSE2 version for 32-bit x86; on x86-64 the version above is used
    // instead, like in C memcmplen.h.
    #[cfg(all(
        any(
            target_arch = "x86",
            all(target_arch = "x86_64", target_pointer_width = "32")
        ),
        target_feature = "sse2"
    ))]
    {
        const _: () = assert!(LZMA_MEMCMPLEN_EXTRA == 16);
        #[cfg(target_arch = "x86")]
        use core::arch::x86::{__m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8};
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::{__m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8};
        while len < limit {
            let x: u32 = 0xffff
                ^ _mm_movemask_epi8(_mm_cmpeq_epi8(
                    _mm_loadu_si128(buf1.add(len as usize) as *const __m128i),
                    _mm_loadu_si128(buf2.add(len as usize) as *const __m128i),
                )) as u32;
            if x != 0 {
                len += x.trailing_zeros();
                return core::cmp::min(len, limit);
            }
            len += 16;
        }
        limit
    }

    // Generic 32-bit word compares for targets with fast unaligned access
    // but no 64-bit or SSE2 branch above.
    #[cfg(any(
        all(target_arch = "x86", not(target_feature = "sse2")),
        target_arch = "powerpc"
    ))]
    {
        const _: () = assert!(LZMA_MEMCMPLEN_EXTRA == 4);
        while len < limit {
            let lhs = core::ptr::read_unaligned(buf1.add(len as usize) as *const u32);
            let rhs = core::ptr::read_unaligned(buf2.add(len as usize) as *const u32);
            let mut x = if cfg!(target_endian = "little") {
                lhs.wrapping_sub(rhs)
            } else {
                lhs ^ rhs
            };
            if x != 0 {
                if cfg!(target_endian = "little") {
                    if x & 0xFFFF == 0 {
                        len += 2;
                        x >>= 16;
                    }
                    if x & 0xFF == 0 {
                        len += 1;
                    }
                } else {
                    if x & 0xFFFF_0000 == 0 {
                        len += 2;
                        x <<= 16;
                    }
                    if x & 0xFF00_0000 == 0 {
                        len += 1;
                    }
                }
                return core::cmp::min(len, limit);
            }
            len += 4;
        }
        limit
    }

    #[cfg(not(any(
        all(
            target_pointer_width = "64",
            any(
                target_arch = "x86_64",
                target_arch = "aarch64",
                target_arch = "powerpc64"
            )
        ),
        all(
            any(
                target_arch = "x86",
                all(target_arch = "x86_64", target_pointer_width = "32")
            ),
            target_feature = "sse2"
        ),
        any(
            all(target_arch = "x86", not(target_feature = "sse2")),
            target_arch = "powerpc"
        )
    )))]
    {
        const _: () = assert!(LZMA_MEMCMPLEN_EXTRA == 0);
        while len < limit && *buf1.add(len as usize) == *buf2.add(len as usize) {
            len += 1;
        }
        len
    }
}
#[inline]
pub fn get_dist_slot(dist: u32) -> u32 {
    if dist < 1 << FASTPOS_BITS + (0 + 0 * (FASTPOS_BITS - 1)) {
        return lzma_fastpos[dist as usize] as u32;
    }
    if dist < 1 << FASTPOS_BITS + (0 + 1 * (FASTPOS_BITS - 1)) {
        return (lzma_fastpos[(dist >> 0 + 1 * (FASTPOS_BITS - 1)) as usize] as u32)
            .wrapping_add((2 * (0 + 1 * (FASTPOS_BITS - 1))) as u32);
    }
    (lzma_fastpos[(dist >> 0 + 2 * (FASTPOS_BITS - 1)) as usize] as u32)
        .wrapping_add((2 * (0 + 2 * (FASTPOS_BITS - 1))) as u32)
}
#[inline]
fn rc_price_at(index: u32) -> u32 {
    debug_assert!((index as usize) < 128);
    unsafe { *lzma_rc_prices.as_ptr().add(index as usize) as u32 }
}
#[inline]
pub fn rc_bit_price(prob: probability, bit: u32) -> u32 {
    rc_price_at(
        ((prob as u32 ^ 0u32.wrapping_sub(bit) & (RC_BIT_MODEL_TOTAL as u32).wrapping_sub(1))
            >> RC_MOVE_REDUCING_BITS) as u32,
    )
}
#[inline]
pub fn rc_bit_0_price(prob: probability) -> u32 {
    rc_price_at((prob >> RC_MOVE_REDUCING_BITS) as u32)
}
#[inline]
pub fn rc_bit_1_price(prob: probability) -> u32 {
    rc_price_at(
        ((prob as u32 ^ RC_BIT_MODEL_TOTAL.wrapping_sub(1)) >> RC_MOVE_REDUCING_BITS) as u32,
    )
}
#[inline]
pub unsafe fn rc_bittree_price(probs: *const probability, bit_levels: u32, mut symbol: u32) -> u32 {
    let mut price: u32 = 0;
    symbol = (symbol as u32).wrapping_add(1u32 << bit_levels) as u32;
    loop {
        let bit: u32 = symbol & 1;
        symbol >>= 1;
        price = price.wrapping_add(rc_bit_price(*probs.offset(symbol as isize), bit));
        if symbol == 1 {
            break;
        }
    }
    price
}
#[inline]
pub fn mf_get_hash_bytes(match_finder: lzma_match_finder) -> u32 {
    match_finder as u32 & 0xf
}
#[inline]
pub unsafe fn is_lclppb_valid(options: *const lzma_options_lzma) -> bool {
    (*options).lc <= LZMA_LCLP_MAX
        && (*options).lp <= LZMA_LCLP_MAX
        && (*options).lc.wrapping_add((*options).lp) <= LZMA_LCLP_MAX
        && (*options).pb <= LZMA_PB_MAX
}
#[inline]
pub unsafe fn literal_init(probs: *mut probability, lc: u32, lp: u32) {
    let coders: size_t = (LITERAL_CODER_SIZE << lc.wrapping_add(lp)) as size_t;
    let mut i: size_t = 0;
    while i < coders {
        *probs.add(i) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        i += 1;
    }
}
pub fn is_backward_size_valid(options: &lzma_stream_flags) -> bool {
    options.backward_size >= LZMA_BACKWARD_SIZE_MIN as lzma_vli
        && options.backward_size <= LZMA_BACKWARD_SIZE_MAX
        && options.backward_size & 3 == 0
}
#[inline]
pub fn index_size(count: lzma_vli, index_list_size: lzma_vli) -> lzma_vli {
    vli_ceil4(index_size_unpadded(count, index_list_size))
}
/// Borrow a C buffer as a slice.
///
/// The transpiled coder interface passes a pointer and a size, and allows the
/// pointer to be NULL as long as the size is zero. `from_raw_parts` requires a
/// non-null, aligned pointer even for an empty slice, so the zero case gets an
/// empty slice of its own.
///
/// # Safety
/// `ptr` must be readable for `len` bytes, or `len` must be zero.
#[inline]
pub(crate) unsafe fn c_slice<'a>(ptr: *const u8, len: size_t) -> &'a [u8] {
    if len == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(ptr, len)
    }
}

/// Mutable form of [`c_slice`].
///
/// # Safety
/// `ptr` must be writable for `len` bytes, or `len` must be zero.
#[inline]
pub(crate) unsafe fn c_slice_mut<'a>(ptr: *mut u8, len: size_t) -> &'a mut [u8] {
    if len == 0 {
        &mut []
    } else {
        core::slice::from_raw_parts_mut(ptr, len)
    }
}

/// Memory one output buffer of `buf_size` bytes costs, header included.
///
/// The argument is 64-bit because callers reach this both with a size_t from a
/// buffer that was allocated and with an lzma_vli from caller options or a
/// Block header. C takes a size_t for both, so on a 32-bit target the second
/// kind truncates to a few hundred bytes, turning a memlimit rejection into an
/// accept.
pub(crate) fn lzma_outq_outbuf_memusage(buf_size: u64) -> u64 {
    (core::mem::size_of::<lzma_outbuf>() as u64).saturating_add(buf_size)
}
#[inline]
pub unsafe fn aligned_read32ne(buf: *const u8) -> u32 {
    *(buf as *const u32)
}
pub type rc_symbol = u8;
pub const RC_FLUSH: rc_symbol = 4;
pub const RC_DIRECT_1: rc_symbol = 3;
pub const RC_DIRECT_0: rc_symbol = 2;
pub const RC_BIT_1: rc_symbol = 1;
pub const RC_BIT_0: rc_symbol = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_range_encoder {
    pub low: u64,
    pub cache_size: u64,
    pub range: u32,
    pub cache: u8,
    pub out_total: u64,
    pub count: size_t,
    pub pos: size_t,
    pub symbols: [rc_symbol; 53],
    pub probs: [*mut probability; 53],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_optimal {
    pub state: lzma_lzma_state,
    pub prev_1_is_literal: bool,
    pub prev_2: bool,
    pub pos_prev_2: u32,
    pub back_prev_2: u32,
    pub price: u32,
    pub pos_prev: u32,
    pub back_prev: u32,
    pub backs: [u32; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_length_encoder {
    pub choice: probability,
    pub choice2: probability,
    pub low: [[probability; 8]; 16],
    pub mid: [[probability; 8]; 16],
    pub high: [probability; 256],
    pub prices: [[u32; 272]; 16],
    pub table_size: u32,
    pub counters: [u32; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_options_bcj {
    pub start_offset: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_mt {
    pub flags: u32,
    pub threads: u32,
    pub block_size: u64,
    pub timeout: u32,
    pub preset: u32,
    pub filters: *const lzma_filter,
    pub check: lzma_check,
    pub reserved_enum1: lzma_reserved_enum,
    pub reserved_enum2: lzma_reserved_enum,
    pub reserved_enum3: lzma_reserved_enum,
    pub reserved_int1: u32,
    pub reserved_int2: u32,
    pub reserved_int3: u32,
    pub reserved_int4: u32,
    pub memlimit_threading: u64,
    pub memlimit_stop: u64,
    pub reserved_int7: u64,
    pub reserved_int8: u64,
    pub reserved_ptr1: *mut c_void,
    pub reserved_ptr2: *mut c_void,
    pub reserved_ptr3: *mut c_void,
    pub reserved_ptr4: *mut c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_filter_coder {
    pub id: lzma_vli,
    pub init: lzma_init_function,
    pub memusage: Option<unsafe fn(*const c_void) -> u64>,
}
pub type lzma_filter_find = Option<unsafe fn(lzma_vli) -> *const lzma_filter_coder>;
pub(crate) type lzma_simple_filter_function =
    unsafe fn(*mut c_void, u32, bool, *mut u8, size_t) -> size_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct lzma_simple_coder {
    pub(crate) next: lzma_next_coder,
    pub(crate) end_was_reached: bool,
    pub(crate) is_encoder: bool,
    pub(crate) filter: lzma_simple_filter_function,
    pub(crate) simple: *mut c_void,
    pub(crate) now_pos: u32,
    pub(crate) allocated: size_t,
    pub(crate) pos: size_t,
    pub(crate) filtered: size_t,
    pub(crate) size: size_t,
    pub(crate) buffer: [u8; 0],
}
pub use crate::check::check::{
    lzma_check_finish, lzma_check_init, lzma_check_is_supported, lzma_check_size, lzma_check_update,
};
pub use crate::check::crc32_fast::crc32;
pub(crate) use crate::check::crc32_fast::lzma_crc32;
pub use crate::check::crc64_fast::crc64;
pub(crate) use crate::common::block_decoder::lzma_block_decoder_init;
pub(crate) use crate::common::block_encoder::lzma_block_encoder_init;
pub use crate::common::block_header_decoder::lzma_block_header_decode;
pub use crate::common::block_header_encoder::{lzma_block_header_encode, lzma_block_header_size};
pub use crate::common::block_util::lzma_block_unpadded_size;
#[cfg(feature = "custom_allocator")]
pub use crate::common::common::{lzma_alloc, lzma_alloc_zero, lzma_free};
pub use crate::common::common::{
    lzma_bufcpy, lzma_end, lzma_next_end, lzma_next_filter_init, lzma_next_filter_update,
    lzma_strm_init,
};
pub use crate::common::easy_preset::lzma_easy_preset;
pub use crate::common::filter_common::{
    lzma_filters_copy, lzma_filters_free, lzma_raw_coder_init, lzma_raw_coder_memusage,
};
pub use crate::common::filter_decoder::{lzma_raw_decoder_init, lzma_raw_decoder_memusage};
pub use crate::common::filter_encoder::{lzma_raw_encoder_init, lzma_raw_encoder_memusage};
pub use crate::common::hardware_cputhreads::lzma_cputhreads;
pub use crate::common::hardware_physmem::lzma_physmem;
pub use crate::common::index::{
    lzma_index_append, lzma_index_end, lzma_index_init, lzma_index_memusage,
    lzma_index_padding_size, lzma_index_size,
};
pub(crate) use crate::common::index_encoder::lzma_index_encoder_init;
pub use crate::common::index_hash::{
    lzma_index_hash_append, lzma_index_hash_decode, lzma_index_hash_end, lzma_index_hash_init,
    lzma_index_hash_size,
};
pub use crate::common::outqueue::{
    lzma_outq_end, lzma_outq_get_buf, lzma_outq_init, lzma_outq_is_readable,
    lzma_outq_prealloc_buf, lzma_outq_read,
};
pub(crate) use crate::common::stream_decoder::lzma_stream_decoder_init;
pub use crate::common::stream_flags_common::lzma_stream_flags_compare;
pub use crate::common::stream_flags_decoder::{
    lzma_stream_footer_decode, lzma_stream_header_decode,
};
pub use crate::common::stream_flags_encoder::{
    lzma_stream_footer_encode, lzma_stream_header_encode,
};
pub use crate::common::threading::{
    MYTHREAD_RET_VALUE, mythread, mythread_cond, mythread_cond_destroy, mythread_cond_init,
    mythread_cond_signal, mythread_cond_timedwait, mythread_cond_wait, mythread_condtime,
    mythread_condtime_set, mythread_create, mythread_join, mythread_mutex, mythread_mutex_destroy,
    mythread_mutex_init, mythread_mutex_lock, mythread_mutex_unlock,
};
pub use crate::common::vli_decoder::lzma_vli_decode;
pub use crate::common::vli_encoder::lzma_vli_encode;
pub use crate::common::vli_size::lzma_vli_size;
pub use crate::delta::delta_common::lzma_delta_coder_init;
pub(crate) use crate::delta::delta_common::lzma_delta_coder_memusage;
pub use crate::lz::lz_encoder_mf::{lzma_mf_find, lzma_mf_find_raw};
pub use crate::lzma::fastpos_table::lzma_fastpos;
pub use crate::lzma::lzma_decoder::lzma_lzma_lclppb_decode;
pub(crate) use crate::lzma::lzma_decoder::{
    lzma_lzma_decoder_init, lzma_lzma_decoder_memusage_nocheck,
};
pub use crate::lzma::lzma_encoder::lzma_lzma_lclppb_encode;
pub(crate) use crate::lzma::lzma_encoder::{lzma_lzma_encoder_init, lzma_lzma_encoder_memusage};
pub use crate::lzma::lzma_encoder_presets::lzma_lzma_preset;
pub use crate::rangecoder::price_table::lzma_rc_prices;
pub(crate) use crate::simple::simple_coder::lzma_simple_coder_init;
unsafe extern "C" {
    pub fn memcmp(s1: *const c_void, s2: *const c_void, n: size_t) -> c_int;
    pub fn strlen(s: *const c_char) -> size_t;
}

pub unsafe fn memchr(s: *const c_void, c: c_int, n: size_t) -> *mut c_void {
    let bytes = core::slice::from_raw_parts(s as *const u8, n);
    let needle = c as u8;
    match ::memchr::memchr(needle, bytes) {
        Some(index) => (s as *const u8).add(index) as *mut c_void,
        None => core::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::{SIZE_MAX, UINTPTR_MAX, size_t, uintptr_t};

    #[test]
    fn c_size_bounds_use_pointer_width_types() {
        assert_eq!(SIZE_MAX, size_t::MAX);
        assert_eq!(UINTPTR_MAX, uintptr_t::MAX);
    }
}
