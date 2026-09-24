use core::ptr::NonNull;

use crate::types::{
    LZMA_PROG_ERROR, LZMA_VLI_UNKNOWN, c_uint, c_void, lzma_action, lzma_allocator, lzma_check,
    lzma_filter, lzma_ret, lzma_vli, size_t, uintptr_t,
};

/// One stage of a coder chain: the implementation behind [`lzma_next_coder`].
/// C keeps a `void *` next to a table of function pointers, some of them NULL;
/// here the table is this trait's vtable. The methods C leaves NULL return
/// `None`, and the callers that used to test the slot test the result instead.
///
/// The state is allocated through `lzma_allocator` by the init function that
/// fills [`lzma_next_coder::coder`], and [`end`](Self::end) frees it again,
/// as C's end functions do; `lzma_next_end` calls nothing after it.
pub trait NextCoder {
    unsafe fn code(
        &mut self,
        allocator: *const lzma_allocator,
        input: *const u8,
        in_pos: *mut size_t,
        in_size: size_t,
        out: *mut u8,
        out_pos: *mut size_t,
        out_size: size_t,
        action: lzma_action,
    ) -> lzma_ret;
    /// Releases what the state owns and frees the state. The default is for a
    /// state that owns nothing.
    unsafe fn end(&mut self, allocator: *const lzma_allocator) {
        crate::alloc::internal_free_dyn(self, allocator);
    }
    /// `(progress_in, progress_out)`, or `None` to report the stream's totals.
    unsafe fn get_progress(&mut self) -> Option<(u64, u64)> {
        None
    }
    unsafe fn get_check(&self) -> Option<lzma_check> {
        None
    }
    unsafe fn memconfig(
        &mut self,
        _memusage: *mut u64,
        _old_memlimit: *mut u64,
        _new_memlimit: u64,
    ) -> Option<lzma_ret> {
        None
    }
    /// Whether [`update`](Self::update) is implemented. `lzma_filters_update`
    /// answers a coder that cannot be updated before it validates the filters.
    fn can_update(&self) -> bool {
        false
    }
    unsafe fn update(
        &mut self,
        _allocator: *const lzma_allocator,
        _filters: *const lzma_filter,
        _reversed_filters: *const lzma_filter,
    ) -> lzma_ret {
        LZMA_PROG_ERROR
    }
    unsafe fn set_out_limit(
        &mut self,
        _uncomp_size: *mut u64,
        _out_limit: u64,
    ) -> Option<lzma_ret> {
        None
    }
}

pub type lzma_next_coder = lzma_next_coder_s;

#[derive(Copy, Clone)]
pub struct lzma_next_coder_s {
    pub coder: Option<NonNull<dyn NextCoder>>,
    pub id: lzma_vli,
    pub init: uintptr_t,
}

pub const LZMA_NEXT_CODER_INIT: lzma_next_coder = lzma_next_coder_s {
    coder: None,
    id: LZMA_VLI_UNKNOWN,
    init: 0,
};

impl lzma_next_coder_s {
    /// The state as the type the init function identified by `init` allocated,
    /// or null when there is none. Callers compare `init` first.
    pub fn coder_as<T: NextCoder>(&self) -> *mut T {
        match self.coder {
            Some(p) => p.cast::<T>().as_ptr(),
            None => core::ptr::null_mut(),
        }
    }
    /// Stores a freshly allocated, non-null state.
    pub unsafe fn set_coder<T: NextCoder + 'static>(&mut self, coder: *mut T) {
        debug_assert!(!coder.is_null());
        let coder: NonNull<dyn NextCoder> = NonNull::new_unchecked(coder);
        self.coder = Some(coder);
    }
    /// Callers only reach this with a coder present; C would call through a
    /// NULL pointer otherwise.
    pub unsafe fn code(
        &mut self,
        allocator: *const lzma_allocator,
        input: *const u8,
        in_pos: *mut size_t,
        in_size: size_t,
        out: *mut u8,
        out_pos: *mut size_t,
        out_size: size_t,
        action: lzma_action,
    ) -> lzma_ret {
        debug_assert!(self.coder.is_some());
        self.coder.unwrap_unchecked().as_mut().code(
            allocator, input, in_pos, in_size, out, out_pos, out_size, action,
        )
    }
    pub unsafe fn get_progress(&mut self) -> Option<(u64, u64)> {
        self.coder?.as_mut().get_progress()
    }
    pub unsafe fn get_check(&self) -> Option<lzma_check> {
        self.coder?.as_ref().get_check()
    }
    pub unsafe fn memconfig(
        &mut self,
        memusage: *mut u64,
        old_memlimit: *mut u64,
        new_memlimit: u64,
    ) -> Option<lzma_ret> {
        self.coder?
            .as_mut()
            .memconfig(memusage, old_memlimit, new_memlimit)
    }
    pub fn can_update(&self) -> bool {
        self.coder
            .map_or(false, |p| unsafe { p.as_ref() }.can_update())
    }
    pub unsafe fn update(
        &mut self,
        allocator: *const lzma_allocator,
        filters: *const lzma_filter,
        reversed_filters: *const lzma_filter,
    ) -> lzma_ret {
        match self.coder {
            Some(mut p) => p.as_mut().update(allocator, filters, reversed_filters),
            None => LZMA_PROG_ERROR,
        }
    }
    pub unsafe fn set_out_limit(
        &mut self,
        uncomp_size: *mut u64,
        out_limit: u64,
    ) -> Option<lzma_ret> {
        self.coder?.as_mut().set_out_limit(uncomp_size, out_limit)
    }
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
