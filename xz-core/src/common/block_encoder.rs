use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_block_coder {
    pub next: lzma_next_coder,
    pub block: *mut lzma_block,
    pub sequence: block_encoder_seq,
    pub compressed_size: lzma_vli,
    pub uncompressed_size: lzma_vli,
    pub pos: size_t,
    pub check: lzma_check_state,
}
pub type block_encoder_seq = c_uint;
pub const SEQ_CHECK: block_encoder_seq = 2;
pub const SEQ_PADDING: block_encoder_seq = 1;
pub const SEQ_CODE: block_encoder_seq = 0;
unsafe fn block_encode(
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
    let coder: *mut lzma_block_coder = coder_ptr as *mut lzma_block_coder;
    if (LZMA_VLI_MAX).wrapping_sub((*coder).uncompressed_size)
        < in_size.wrapping_sub(*in_pos) as lzma_vli
    {
        return LZMA_DATA_ERROR;
    }
    match (*coder).sequence {
        0 => {
            let in_start: size_t = *in_pos;
            let out_start: size_t = *out_pos;
            debug_assert!((*coder).next.code.is_some());
            let code = (*coder).next.code.unwrap_unchecked();
            let ret: lzma_ret = code(
                (*coder).next.coder,
                allocator,
                input,
                in_pos,
                in_size,
                out,
                out_pos,
                out_size,
                action,
            );
            let in_used: size_t = *in_pos - in_start;
            let out_used: size_t = *out_pos - out_start;
            if (COMPRESSED_SIZE_MAX).wrapping_sub((*coder).compressed_size) < out_used as lzma_vli {
                return LZMA_DATA_ERROR;
            }
            (*coder).compressed_size += out_used as lzma_vli;
            (*coder).uncompressed_size += in_used as lzma_vli;
            if in_used > 0 {
                lzma_check_update(
                    ::core::ptr::addr_of_mut!((*coder).check),
                    (*(*coder).block).check,
                    input.add(in_start),
                    in_used,
                );
            }
            if ret != LZMA_STREAM_END || action == LZMA_SYNC_FLUSH {
                return ret;
            }
            (*(*coder).block).compressed_size = (*coder).compressed_size;
            (*(*coder).block).uncompressed_size = (*coder).uncompressed_size;
            (*coder).sequence = SEQ_PADDING;
        }
        1 | 2 => {}
        _ => return LZMA_PROG_ERROR,
    }
    if (*coder).sequence != SEQ_CHECK {
        while (*coder).compressed_size & 3 != 0 {
            if *out_pos >= out_size {
                return LZMA_OK;
            }
            *out.add(*out_pos) = 0;
            *out_pos += 1;
            (*coder).compressed_size += 1;
        }
        if (*(*coder).block).check == LZMA_CHECK_NONE {
            return LZMA_STREAM_END;
        }
        lzma_check_finish(
            ::core::ptr::addr_of_mut!((*coder).check),
            (*(*coder).block).check,
        );
        (*coder).sequence = SEQ_CHECK;
    }
    let check_size: size_t = lzma_check_size((*(*coder).block).check) as size_t;
    lzma_bufcpy(
        ::core::ptr::addr_of_mut!((*coder).check.buffer.u8_0) as *mut u8,
        ::core::ptr::addr_of_mut!((*coder).pos),
        check_size,
        out,
        out_pos,
        out_size,
    );
    if (*coder).pos < check_size {
        return LZMA_OK;
    }
    core::ptr::copy_nonoverlapping(
        ::core::ptr::addr_of_mut!((*coder).check.buffer.u8_0) as *const u8,
        ::core::ptr::addr_of_mut!((*(*coder).block).raw_check) as *mut u8,
        check_size,
    );
    LZMA_STREAM_END
}
unsafe fn block_encoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_block_coder = coder_ptr as *mut lzma_block_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).next), allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn block_encoder_update(
    coder_ptr: *mut c_void,
    allocator: *const lzma_allocator,
    _filters: *const lzma_filter,
    reversed_filters: *const lzma_filter,
) -> lzma_ret {
    let coder: *mut lzma_block_coder = coder_ptr as *mut lzma_block_coder;
    if (*coder).sequence != SEQ_CODE {
        return LZMA_PROG_ERROR;
    }
    lzma_next_filter_update(
        ::core::ptr::addr_of_mut!((*coder).next),
        allocator,
        reversed_filters,
    )
}
pub(crate) unsafe fn lzma_block_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    block: *mut lzma_block,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *mut lzma_block) -> lzma_ret>,
        uintptr_t,
    >(Some(
        lzma_block_encoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *mut lzma_block) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *mut lzma_block) -> lzma_ret>,
        uintptr_t,
    >(Some(
        lzma_block_encoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *mut lzma_block) -> lzma_ret,
    ));
    if block.is_null() {
        return LZMA_PROG_ERROR;
    }
    if (*block).version > 1 {
        return LZMA_OPTIONS_ERROR;
    }
    if (*block).check as c_uint > LZMA_CHECK_ID_MAX as c_uint {
        return LZMA_PROG_ERROR;
    }
    if lzma_check_is_supported((*block).check) == 0 {
        return LZMA_UNSUPPORTED_CHECK;
    }
    let mut coder: *mut lzma_block_coder = (*next).coder as *mut lzma_block_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_block_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            block_encode
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
        (*next).end =
            Some(block_encoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*next).update = Some(
            block_encoder_update
                as unsafe fn(
                    *mut c_void,
                    *const lzma_allocator,
                    *const lzma_filter,
                    *const lzma_filter,
                ) -> lzma_ret,
        );
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
    (*coder).sequence = SEQ_CODE;
    (*coder).block = block;
    (*coder).compressed_size = 0;
    (*coder).uncompressed_size = 0;
    (*coder).pos = 0;
    lzma_check_init(::core::ptr::addr_of_mut!((*coder).check), (*block).check);
    lzma_raw_encoder_init(
        ::core::ptr::addr_of_mut!((*coder).next),
        allocator,
        (*block).filters,
    )
}
pub unsafe fn lzma_block_encoder(strm: *mut lzma_stream, block: *mut lzma_block) -> lzma_ret {
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = lzma_block_encoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        block,
    );
    if ret__0 != LZMA_OK {
        lzma_end(strm);
        return ret__0;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_SYNC_FLUSH as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
