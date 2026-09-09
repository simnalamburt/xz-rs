use crate::types::{
    c_uint, c_void, lzma_action, lzma_allocator, lzma_check, lzma_filter, lzma_ret, lzma_vli,
    size_t, uintptr_t,
};

pub type lzma_end_function = Option<unsafe fn(*mut c_void, *const lzma_allocator) -> ()>;
pub type lzma_code_function = Option<
    unsafe fn(
        *mut c_void,
        *const lzma_allocator,
        *const u8,
        *mut size_t,
        size_t,
        *mut u8,
        *mut size_t,
        size_t,
        lzma_action,
    ) -> lzma_ret,
>;

pub type lzma_next_coder = lzma_next_coder_s;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_next_coder_s {
    pub coder: *mut c_void,
    pub id: lzma_vli,
    pub init: uintptr_t,
    pub code: lzma_code_function,
    pub end: lzma_end_function,
    pub get_progress: Option<unsafe fn(*mut c_void, &mut u64, &mut u64) -> ()>,
    pub get_check: Option<unsafe fn(*const c_void) -> lzma_check>,
    pub memconfig: Option<unsafe fn(*mut c_void, *mut u64, *mut u64, u64) -> lzma_ret>,
    pub update: Option<
        unsafe fn(
            *mut c_void,
            *const lzma_allocator,
            *const lzma_filter,
            *const lzma_filter,
        ) -> lzma_ret,
    >,
    pub set_out_limit: Option<unsafe fn(*mut c_void, *mut u64, u64) -> lzma_ret>,
}

pub type lzma_init_function = Option<
    unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_filter_info) -> lzma_ret,
>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_filter_info_s {
    pub id: lzma_vli,
    pub init: lzma_init_function,
    pub options: *mut c_void,
}

pub type lzma_filter_info = lzma_filter_info_s;

pub type lzma_internal_sequence = c_uint;
pub const ISEQ_RUN: lzma_internal_sequence = 0;
pub const ISEQ_SYNC_FLUSH: lzma_internal_sequence = 1;
pub const ISEQ_FULL_FLUSH: lzma_internal_sequence = 2;
pub const ISEQ_FINISH: lzma_internal_sequence = 3;
pub const ISEQ_FULL_BARRIER: lzma_internal_sequence = 4;
pub const ISEQ_END: lzma_internal_sequence = 5;
pub const ISEQ_ERROR: lzma_internal_sequence = 6;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_internal_s {
    pub next: lzma_next_coder,
    pub sequence: lzma_internal_sequence,
    pub avail_in: size_t,
    pub supported_actions: [bool; 5],
    pub allow_buf_error: bool,
}

pub type lzma_internal = lzma_internal_s;

pub const LZMA_MEMUSAGE_BASE: u64 = 1 << 15;
pub const LZMA_SUPPORTED_FLAGS: c_uint = crate::types::LZMA_TELL_NO_CHECK
    | crate::types::LZMA_TELL_UNSUPPORTED_CHECK
    | crate::types::LZMA_TELL_ANY_CHECK
    | crate::types::LZMA_IGNORE_CHECK
    | crate::types::LZMA_CONCATENATED
    | crate::types::LZMA_FAIL_FAST;
pub const LZMA_THREADS_MAX: u32 = 16384;
