#[cfg(feature = "custom_allocator")]
pub use crate::alloc::{lzma_alloc, lzma_alloc_zero, lzma_free};
use crate::types::*;
pub const LZMA_VERSION_MAJOR: u32 = 5;
pub const LZMA_VERSION_MINOR: u32 = 8;
pub const LZMA_VERSION_PATCH: u32 = 3;
pub const LZMA_VERSION_STABILITY: u32 = LZMA_VERSION_STABILITY_STABLE;
pub const LZMA_VERSION_STABILITY_STABLE: u32 = 2;
pub const LZMA_VERSION: c_uint = LZMA_VERSION_MAJOR * 10000000
    + LZMA_VERSION_MINOR * 10000
    + LZMA_VERSION_PATCH * 10
    + LZMA_VERSION_STABILITY;
pub const LZMA_TIMED_OUT: c_uint = 101;
pub fn lzma_version_number() -> u32 {
    LZMA_VERSION as u32
}
pub fn lzma_version_string() -> *const c_char {
    crate::c_str!("5.8.3")
}
#[inline]
pub unsafe fn lzma_stream_allocator(strm: *const lzma_stream) -> *const lzma_allocator {
    #[cfg(feature = "custom_allocator")]
    {
        unsafe { (*strm).allocator }
    }
    #[cfg(not(feature = "custom_allocator"))]
    {
        let _ = strm;
        core::ptr::null()
    }
}
#[inline]
pub unsafe fn lzma_alloc_object<T>(allocator: *const lzma_allocator) -> *mut T {
    debug_assert!(core::mem::align_of::<T>() <= 16);
    crate::alloc::internal_alloc_object::<T>(allocator)
}
pub unsafe fn lzma_bufcpy(
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> size_t {
    if *in_pos > in_size || *out_pos > out_size {
        return 0;
    }
    if (input.is_null() && *in_pos != in_size) || (out.is_null() && *out_pos != out_size) {
        return 0;
    }
    debug_assert!(!input.is_null() || *in_pos == in_size);
    debug_assert!(!out.is_null() || *out_pos == out_size);
    debug_assert!(*in_pos <= in_size);
    debug_assert!(*out_pos <= out_size);

    let in_avail: size_t = in_size - *in_pos;
    let out_avail: size_t = out_size - *out_pos;
    let copy_size: size_t = if in_avail < out_avail {
        in_avail
    } else {
        out_avail
    };
    if copy_size > 0 {
        core::ptr::copy_nonoverlapping(
            input.add(*in_pos) as *const u8,
            out.add(*out_pos) as *mut u8,
            copy_size,
        );
    }
    *in_pos += copy_size;
    *out_pos += copy_size;
    copy_size
}
pub unsafe fn lzma_next_filter_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
) -> lzma_ret {
    if core::mem::transmute::<lzma_init_function, uintptr_t>((*filters).init) != (*next).init {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<lzma_init_function, uintptr_t>((*filters).init);
    (*next).id = (*filters).id;
    if let Some(init) = (*filters).init {
        init(next, allocator, filters)
    } else {
        LZMA_OK
    }
}
pub unsafe fn lzma_next_filter_update(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    reversed_filters: *const lzma_filter,
) -> lzma_ret {
    if (*reversed_filters).id != (*next).id {
        return LZMA_PROG_ERROR;
    }
    if (*reversed_filters).id == LZMA_VLI_UNKNOWN {
        return LZMA_OK;
    }
    debug_assert!((*next).update.is_some());
    let update = (*next).update.unwrap_unchecked();
    update(
        (*next).coder,
        allocator,
        core::ptr::null(),
        reversed_filters,
    )
}
pub unsafe fn lzma_next_end(next: *mut lzma_next_coder, allocator: *const lzma_allocator) {
    if (*next).init == 0 {
        return;
    }
    if let Some(end) = (*next).end {
        end((*next).coder, allocator);
    } else {
        crate::alloc::internal_free_untyped((*next).coder, allocator);
    }
    *next = lzma_next_coder_s {
        coder: core::ptr::null_mut(),
        id: LZMA_VLI_UNKNOWN,
        init: 0,
        code: None,
        end: None,
        get_progress: None,
        get_check: None,
        memconfig: None,
        update: None,
        set_out_limit: None,
    };
}
pub unsafe fn lzma_strm_init(strm: *mut lzma_stream) -> lzma_ret {
    if strm.is_null() {
        return LZMA_PROG_ERROR;
    }
    if (*strm).internal.is_null() {
        (*strm).internal = lzma_alloc_object::<lzma_internal>(lzma_stream_allocator(strm));
        if (*strm).internal.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*(*strm).internal).next = lzma_next_coder_s {
            coder: core::ptr::null_mut(),
            id: LZMA_VLI_UNKNOWN,
            init: 0,
            code: None,
            end: None,
            get_progress: None,
            get_check: None,
            memconfig: None,
            update: None,
            set_out_limit: None,
        };
    }
    core::ptr::write_bytes(
        ::core::ptr::addr_of_mut!((*(*strm).internal).supported_actions) as *mut u8,
        0 as u8,
        core::mem::size_of::<[bool; 5]>(),
    );
    (*(*strm).internal).sequence = ISEQ_RUN;
    (*(*strm).internal).allow_buf_error = false;
    (*(*strm).internal).avail_in = 0;
    (*strm).total_in = 0;
    (*strm).total_out = 0;
    LZMA_OK
}
pub unsafe fn lzma_code(strm: *mut lzma_stream, action: lzma_action) -> lzma_ret {
    if (*strm).next_in.is_null() && (*strm).avail_in != 0
        || (*strm).next_out.is_null() && (*strm).avail_out != 0
        || (*strm).internal.is_null()
        || (*(*strm).internal).next.code.is_none()
        || action as c_uint > LZMA_FULL_BARRIER as c_uint
        || !(*(*strm).internal).supported_actions[action as usize]
    {
        return LZMA_PROG_ERROR;
    }
    if !(*strm).reserved_ptr1.is_null()
        || !(*strm).reserved_ptr2.is_null()
        || !(*strm).reserved_ptr3.is_null()
        || !(*strm).reserved_ptr4.is_null()
        || (*strm).reserved_int2 != 0
        || (*strm).reserved_int3 != 0
        || (*strm).reserved_int4 != 0
        || (*strm).reserved_enum1 != LZMA_RESERVED_ENUM
        || (*strm).reserved_enum2 != LZMA_RESERVED_ENUM
    {
        return LZMA_OPTIONS_ERROR;
    }
    match (*(*strm).internal).sequence {
        0 => match action {
            1 => {
                (*(*strm).internal).sequence = ISEQ_SYNC_FLUSH;
            }
            2 => {
                (*(*strm).internal).sequence = ISEQ_FULL_FLUSH;
            }
            3 => {
                (*(*strm).internal).sequence = ISEQ_FINISH;
            }
            4 => {
                (*(*strm).internal).sequence = ISEQ_FULL_BARRIER;
            }
            0 | _ => {}
        },
        1 => {
            if action != LZMA_SYNC_FLUSH || (*(*strm).internal).avail_in != (*strm).avail_in {
                return LZMA_PROG_ERROR;
            }
        }
        2 => {
            if action != LZMA_FULL_FLUSH || (*(*strm).internal).avail_in != (*strm).avail_in {
                return LZMA_PROG_ERROR;
            }
        }
        3 => {
            if action != LZMA_FINISH || (*(*strm).internal).avail_in != (*strm).avail_in {
                return LZMA_PROG_ERROR;
            }
        }
        4 => {
            if action != LZMA_FULL_BARRIER || (*(*strm).internal).avail_in != (*strm).avail_in {
                return LZMA_PROG_ERROR;
            }
        }
        5 => return LZMA_STREAM_END,
        6 | _ => return LZMA_PROG_ERROR,
    }
    let mut in_pos: size_t = 0;
    let mut out_pos: size_t = 0;
    debug_assert!((*(*strm).internal).next.code.is_some());
    let code = (*(*strm).internal).next.code.unwrap_unchecked();
    let mut ret: lzma_ret = code(
        (*(*strm).internal).next.coder,
        lzma_stream_allocator(strm),
        (*strm).next_in,
        ::core::ptr::addr_of_mut!(in_pos),
        (*strm).avail_in,
        (*strm).next_out,
        ::core::ptr::addr_of_mut!(out_pos),
        (*strm).avail_out,
        action,
    );
    if in_pos > 0 {
        (*strm).next_in = (*strm).next_in.add(in_pos);
        (*strm).avail_in -= in_pos;
        (*strm).total_in = (*strm).total_in.wrapping_add(in_pos as u64);
    }
    if out_pos > 0 {
        (*strm).next_out = (*strm).next_out.add(out_pos);
        (*strm).avail_out -= out_pos;
        (*strm).total_out = (*strm).total_out.wrapping_add(out_pos as u64);
    }
    (*(*strm).internal).avail_in = (*strm).avail_in;
    match ret {
        0 => {
            if out_pos == 0 && in_pos == 0 {
                if (*(*strm).internal).allow_buf_error {
                    ret = LZMA_BUF_ERROR;
                } else {
                    (*(*strm).internal).allow_buf_error = true;
                }
            } else {
                (*(*strm).internal).allow_buf_error = false;
            }
        }
        101 => {
            (*(*strm).internal).allow_buf_error = false;
            ret = LZMA_OK;
        }
        12 => {
            (*(*strm).internal).allow_buf_error = false;
            if (*(*strm).internal).sequence == ISEQ_FINISH {
                (*(*strm).internal).sequence = ISEQ_RUN;
            }
        }
        1 => {
            if (*(*strm).internal).sequence == ISEQ_SYNC_FLUSH
                || (*(*strm).internal).sequence == ISEQ_FULL_FLUSH
                || (*(*strm).internal).sequence == ISEQ_FULL_BARRIER
            {
                (*(*strm).internal).sequence = ISEQ_RUN;
            } else {
                (*(*strm).internal).sequence = ISEQ_END;
            }
            (*(*strm).internal).allow_buf_error = false;
        }
        2 | 3 | 4 | 6 => {
            (*(*strm).internal).allow_buf_error = false;
        }
        _ => {
            (*(*strm).internal).sequence = ISEQ_ERROR;
        }
    }
    ret
}
pub unsafe fn lzma_end(strm: *mut lzma_stream) {
    if !strm.is_null() && !(*strm).internal.is_null() {
        lzma_next_end(
            ::core::ptr::addr_of_mut!((*(*strm).internal).next),
            lzma_stream_allocator(strm),
        );
        crate::alloc::internal_free((*strm).internal, lzma_stream_allocator(strm));
        (*strm).internal = core::ptr::null_mut();
    }
}
/// # Safety
/// `strm` may be NULL or zeroed, but if `internal` is set it must be the
/// coder `lzma_strm_init` installed.
pub unsafe fn lzma_get_progress(
    strm: *mut lzma_stream,
    progress_in: &mut u64,
    progress_out: &mut u64,
) {
    if strm.is_null() || (*strm).internal.is_null() {
        *progress_in = 0;
        *progress_out = 0;
        return;
    }

    if let Some(get_progress) = (*(*strm).internal).next.get_progress {
        get_progress((*(*strm).internal).next.coder, progress_in, progress_out);
    } else {
        *progress_in = (*strm).total_in;
        *progress_out = (*strm).total_out;
    };
}
pub unsafe fn lzma_get_check(strm: *const lzma_stream) -> lzma_check {
    unsafe {
        if strm.is_null() || (*strm).internal.is_null() {
            return LZMA_CHECK_NONE;
        }
        if let Some(get_check) = (*(*strm).internal).next.get_check {
            get_check((*(*strm).internal).next.coder)
        } else {
            LZMA_CHECK_NONE
        }
    }
}
pub unsafe fn lzma_memusage(strm: *const lzma_stream) -> u64 {
    unsafe {
        let mut memusage: u64 = 0;
        let mut old_memlimit: u64 = 0;
        if strm.is_null() || (*strm).internal.is_null() {
            return 0;
        }
        let Some(memconfig) = (*(*strm).internal).next.memconfig else {
            return 0;
        };
        if memconfig(
            (*(*strm).internal).next.coder,
            ::core::ptr::addr_of_mut!(memusage),
            ::core::ptr::addr_of_mut!(old_memlimit),
            0,
        ) != LZMA_OK
        {
            return 0;
        }
        memusage
    }
}
pub unsafe fn lzma_memlimit_get(strm: *const lzma_stream) -> u64 {
    unsafe {
        let mut old_memlimit: u64 = 0;
        let mut memusage: u64 = 0;
        if strm.is_null() || (*strm).internal.is_null() {
            return 0;
        }
        let Some(memconfig) = (*(*strm).internal).next.memconfig else {
            return 0;
        };
        if memconfig(
            (*(*strm).internal).next.coder,
            ::core::ptr::addr_of_mut!(memusage),
            ::core::ptr::addr_of_mut!(old_memlimit),
            0,
        ) != LZMA_OK
        {
            return 0;
        }
        old_memlimit
    }
}
pub unsafe fn lzma_memlimit_set(strm: *mut lzma_stream, mut new_memlimit: u64) -> lzma_ret {
    let mut old_memlimit: u64 = 0;
    let mut memusage: u64 = 0;
    if strm.is_null() || (*strm).internal.is_null() {
        return LZMA_PROG_ERROR;
    }
    let Some(memconfig) = (*(*strm).internal).next.memconfig else {
        return LZMA_PROG_ERROR;
    };
    if new_memlimit == 0 {
        new_memlimit = 1;
    }
    memconfig(
        (*(*strm).internal).next.coder,
        ::core::ptr::addr_of_mut!(memusage),
        ::core::ptr::addr_of_mut!(old_memlimit),
        new_memlimit,
    )
}
