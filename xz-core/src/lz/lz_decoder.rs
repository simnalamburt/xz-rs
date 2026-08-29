use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_lz_options {
    pub dict_size: size_t,
    pub preset_dict: *const u8,
    pub preset_dict_size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_coder {
    pub dict: lzma_dict,
    pub lz: lzma_lz_decoder,
    pub next: lzma_next_coder,
    pub next_finished: bool,
    pub this_finished: bool,
    pub temp: lz_decoder_temp_buf,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lz_decoder_temp_buf {
    pub pos: size_t,
    pub size: size_t,
    pub buffer: [u8; LZMA_BUFFER_SIZE as usize],
}
pub const LZMA_BUFFER_SIZE: u32 = 4096;
pub const LZ_DICT_EXTRA: u32 = 0;
pub const LZMA_LZ_DECODER_INIT: lzma_lz_decoder = lzma_lz_decoder {
    coder: core::ptr::null_mut(),
    code: lzma_lz_decoder_code_uninitialized,
    reset: None,
    set_uncompressed: None,
    end: None,
};
unsafe fn lz_decoder_reset(coder: *mut lzma_coder) {
    (*coder).dict.pos = LZ_DICT_INIT_POS as size_t;
    (*coder).dict.full = 0;
    *(*coder).dict.buf.add((LZ_DICT_INIT_POS - 1) as size_t) = '\0' as i32 as u8;
    (*coder).dict.has_wrapped = false;
    (*coder).dict.need_reset = false;
}
unsafe fn decode_buffer(
    coder: *mut lzma_coder,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    loop {
        if (*coder).dict.pos == (*coder).dict.size {
            (*coder).dict.pos = LZ_DICT_REPEAT_MAX as size_t;
            (*coder).dict.has_wrapped = true;
            core::ptr::copy_nonoverlapping(
                (*coder)
                    .dict
                    .buf
                    .add((*coder).dict.size - LZ_DICT_REPEAT_MAX as size_t)
                    as *const u8,
                (*coder).dict.buf as *mut u8,
                LZ_DICT_REPEAT_MAX as size_t,
            );
        }
        let dict_start: size_t = (*coder).dict.pos;
        (*coder).dict.limit = (*coder).dict.pos.wrapping_add(
            if out_size.wrapping_sub(*out_pos) < (*coder).dict.size.wrapping_sub((*coder).dict.pos)
            {
                out_size.wrapping_sub(*out_pos)
            } else {
                (*coder).dict.size.wrapping_sub((*coder).dict.pos)
            },
        );
        let ret: lzma_ret = ((*coder).lz.code)(
            (*coder).lz.coder,
            ::core::ptr::addr_of_mut!((*coder).dict),
            input,
            in_pos,
            in_size,
        );
        let copy_size: size_t = (*coder).dict.pos.wrapping_sub(dict_start);
        if copy_size > 0 {
            core::ptr::copy_nonoverlapping(
                (*coder).dict.buf.add(dict_start) as *const u8,
                out.add(*out_pos) as *mut u8,
                copy_size,
            );
        }
        *out_pos = (*out_pos).wrapping_add(copy_size);
        if (*coder).dict.need_reset {
            lz_decoder_reset(coder);
            if ret != LZMA_OK || *out_pos == out_size {
                return ret;
            }
        } else if ret != LZMA_OK || *out_pos == out_size || (*coder).dict.pos < (*coder).dict.size {
            return ret;
        }
    }
}
unsafe fn lz_decode(
    coder_ptr: *mut c_void,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    action: lzma_action,
) -> lzma_ret {
    let coder: *mut lzma_coder = coder_ptr as *mut lzma_coder;
    if (*coder).next.code.is_none() {
        return decode_buffer(coder, input, in_pos, in_size, out, out_pos, out_size);
    }
    while *out_pos < out_size {
        if !(*coder).next_finished && (*coder).temp.pos == (*coder).temp.size {
            (*coder).temp.pos = 0;
            (*coder).temp.size = 0;
            debug_assert!((*coder).next.code.is_some());
            let next_code = (*coder).next.code.unwrap_unchecked();
            let ret: lzma_ret = next_code(
                (*coder).next.coder,
                allocator,
                input,
                in_pos,
                in_size,
                ::core::ptr::addr_of_mut!((*coder).temp.buffer) as *mut u8,
                ::core::ptr::addr_of_mut!((*coder).temp.size),
                LZMA_BUFFER_SIZE as size_t,
                action,
            );
            if ret == LZMA_STREAM_END {
                (*coder).next_finished = true;
            } else if ret != LZMA_OK || (*coder).temp.size == 0 {
                return ret;
            }
        }
        if (*coder).this_finished {
            if (*coder).temp.size != 0 {
                return LZMA_DATA_ERROR;
            }
            if (*coder).next_finished {
                return LZMA_STREAM_END;
            }
            return LZMA_OK;
        }
        let ret: lzma_ret = decode_buffer(
            coder,
            ::core::ptr::addr_of_mut!((*coder).temp.buffer) as *mut u8,
            ::core::ptr::addr_of_mut!((*coder).temp.pos),
            (*coder).temp.size,
            out,
            out_pos,
            out_size,
        );
        if ret == LZMA_STREAM_END {
            (*coder).this_finished = true;
        } else if ret != LZMA_OK {
            return ret;
        } else if (*coder).next_finished && *out_pos < out_size {
            return LZMA_DATA_ERROR;
        }
    }
    LZMA_OK
}
unsafe fn lz_decoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_coder = coder_ptr as *mut lzma_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).next), allocator);
    crate::alloc::internal_free_array(
        (*coder).dict.buf,
        (*coder).dict.size.wrapping_add(LZ_DICT_EXTRA as size_t),
        allocator,
    );
    if let Some(end) = (*coder).lz.end {
        end((*coder).lz.coder, allocator);
    } else {
        #[cfg(feature = "custom_allocator")]
        crate::alloc::internal_free_bytes((*coder).lz.coder, 0, allocator);
        #[cfg(not(feature = "custom_allocator"))]
        debug_assert!((*coder).lz.coder.is_null());
    }
    crate::alloc::internal_free(coder, allocator);
}
pub unsafe fn lzma_lz_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
    lz_init: unsafe fn(
        *mut lzma_lz_decoder,
        *const lzma_allocator,
        lzma_vli,
        *const c_void,
        *mut lzma_lz_options,
    ) -> lzma_ret,
) -> lzma_ret {
    let mut coder: *mut lzma_coder = (*next).coder as *mut lzma_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            lz_decode
                as unsafe fn(
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
        );
        (*next).end = Some(lz_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*coder).dict.buf = core::ptr::null_mut();
        (*coder).dict.size = 0;
        (*coder).lz = LZMA_LZ_DECODER_INIT;
        (*coder).next = lzma_next_coder_s {
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
    let mut lz_options: lzma_lz_options = lzma_lz_options {
        dict_size: 0,
        preset_dict: core::ptr::null(),
        preset_dict_size: 0,
    };
    let ret: lzma_ret = lz_init(
        ::core::ptr::addr_of_mut!((*coder).lz),
        allocator,
        (*filters).id,
        (*filters).options,
        ::core::ptr::addr_of_mut!(lz_options),
    );
    if ret != LZMA_OK {
        return ret;
    }
    if (*coder).lz.coder.is_null() {
        return LZMA_PROG_ERROR;
    }
    if lz_options.dict_size < 4096 {
        lz_options.dict_size = 4096;
    }
    if lz_options.dict_size
        > (isize::MAX as size_t)
            .saturating_sub(15)
            .saturating_sub((2 * LZ_DICT_REPEAT_MAX) as size_t)
            .saturating_sub(LZ_DICT_EXTRA as size_t)
    {
        return LZMA_MEM_ERROR;
    }
    lz_options.dict_size = lz_options.dict_size.wrapping_add(15) & !(15);
    let alloc_size: size_t = lz_options
        .dict_size
        .wrapping_add((2 * LZ_DICT_REPEAT_MAX) as size_t);
    if (*coder).dict.size != alloc_size {
        crate::alloc::internal_free_array(
            (*coder).dict.buf,
            (*coder).dict.size.wrapping_add(LZ_DICT_EXTRA as size_t),
            allocator,
        );
        (*coder).dict.buf = crate::alloc::internal_alloc_array::<u8>(
            alloc_size.wrapping_add(LZ_DICT_EXTRA as size_t),
            allocator,
        );
        if (*coder).dict.buf.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*coder).dict.size = alloc_size;
    }
    lz_decoder_reset((*next).coder as *mut lzma_coder);
    if !lz_options.preset_dict.is_null() && lz_options.preset_dict_size > 0 {
        let copy_size: size_t = if lz_options.preset_dict_size < lz_options.dict_size {
            lz_options.preset_dict_size
        } else {
            lz_options.dict_size
        };
        let offset: size_t = lz_options.preset_dict_size.wrapping_sub(copy_size);
        core::ptr::copy_nonoverlapping(
            lz_options.preset_dict.add(offset) as *const u8,
            (*coder).dict.buf.add((*coder).dict.pos) as *mut u8,
            copy_size,
        );
        (*coder).dict.pos = (*coder).dict.pos.wrapping_add(copy_size);
        (*coder).dict.full = copy_size;
    }
    (*coder).next_finished = false;
    (*coder).this_finished = false;
    (*coder).temp.pos = 0;
    (*coder).temp.size = 0;
    lzma_next_filter_init(
        ::core::ptr::addr_of_mut!((*coder).next),
        allocator,
        filters.offset(1),
    )
}
pub fn lzma_lz_decoder_memusage(dictionary_size: size_t) -> u64 {
    (core::mem::size_of::<lzma_coder>() as u64)
        .wrapping_add(dictionary_size as u64)
        .wrapping_add((2 * LZ_DICT_REPEAT_MAX) as u64)
        .wrapping_add(LZ_DICT_EXTRA as u64)
}

#[cfg(all(test, target_pointer_width = "32"))]
mod tests {
    use super::*;
    use crate::common::common::lzma_next_end;

    unsafe fn huge_dict_lz_init(
        lz: *mut lzma_lz_decoder,
        _allocator: *const lzma_allocator,
        _id: lzma_vli,
        _opt: *const c_void,
        lz_options: *mut lzma_lz_options,
    ) -> lzma_ret {
        unsafe fn noop_end(_coder: *mut c_void, _allocator: *const lzma_allocator) {}
        (*lz).coder = core::ptr::without_provenance_mut(1);
        (*lz).end = Some(noop_end);
        (*lz_options).dict_size = 1 << 31;
        (*lz_options).preset_dict = core::ptr::null();
        (*lz_options).preset_dict_size = 0;
        LZMA_OK
    }

    #[test]
    fn lz_decoder_init_rejects_dict_larger_than_isize_max() {
        let mut next = lzma_next_coder_s {
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
        let filters = [lzma_filter_info {
            id: 0,
            init: None,
            options: core::ptr::null_mut(),
        }];

        let ret = unsafe {
            lzma_lz_decoder_init(
                core::ptr::addr_of_mut!(next),
                core::ptr::null(),
                filters.as_ptr(),
                huge_dict_lz_init,
            )
        };
        unsafe { lzma_next_end(core::ptr::addr_of_mut!(next), core::ptr::null()) };
        assert_eq!(ret, LZMA_MEM_ERROR);
    }
}
