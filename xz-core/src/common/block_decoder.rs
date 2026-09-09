use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_block_coder {
    pub sequence: block_decoder_seq,
    pub next: lzma_next_coder,
    pub block: *mut lzma_block,
    pub compressed_size: lzma_vli,
    pub uncompressed_size: lzma_vli,
    pub compressed_limit: lzma_vli,
    pub uncompressed_limit: lzma_vli,
    pub check_pos: size_t,
    pub check: lzma_check_state,
    pub ignore_check: bool,
}
pub type block_decoder_seq = c_uint;
pub const SEQ_CHECK: block_decoder_seq = 2;
pub const SEQ_PADDING: block_decoder_seq = 1;
pub const SEQ_CODE: block_decoder_seq = 0;
#[inline]
fn is_size_valid(size: lzma_vli, reference: lzma_vli) -> bool {
    reference == LZMA_VLI_UNKNOWN || reference == size
}
unsafe fn block_decode(
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
    match (*coder).sequence {
        0 => {
            let in_start: size_t = *in_pos;
            let out_start: size_t = *out_pos;
            let in_stop: size_t = *in_pos
                + (if ((in_size - *in_pos) as lzma_vli)
                    < (*coder).compressed_limit - (*coder).compressed_size
                {
                    (in_size - *in_pos) as lzma_vli
                } else {
                    (*coder).compressed_limit - (*coder).compressed_size
                }) as size_t;
            let out_stop: size_t = *out_pos
                + (if ((out_size - *out_pos) as lzma_vli)
                    < (*coder).uncompressed_limit - (*coder).uncompressed_size
                {
                    (out_size - *out_pos) as lzma_vli
                } else {
                    (*coder).uncompressed_limit - (*coder).uncompressed_size
                }) as size_t;
            debug_assert!((*coder).next.code.is_some());
            let code = (*coder).next.code.unwrap_unchecked();
            let ret: lzma_ret = code(
                (*coder).next.coder,
                allocator,
                input,
                in_pos,
                in_stop,
                out,
                out_pos,
                out_stop,
                action,
            );
            let in_used: size_t = *in_pos - in_start;
            let out_used: size_t = *out_pos - out_start;
            (*coder).compressed_size += in_used as lzma_vli;
            (*coder).uncompressed_size += out_used as lzma_vli;
            if ret == LZMA_OK {
                let comp_done: bool = (*coder).compressed_size == (*(*coder).block).compressed_size;
                let uncomp_done: bool =
                    (*coder).uncompressed_size == (*(*coder).block).uncompressed_size;
                if comp_done && uncomp_done {
                    return LZMA_DATA_ERROR;
                }
                if comp_done && *out_pos < out_size {
                    return LZMA_DATA_ERROR;
                }
                if uncomp_done && *in_pos < in_size {
                    return LZMA_DATA_ERROR;
                }
            }
            if !(*coder).ignore_check && out_used > 0 {
                lzma_check_update(
                    ::core::ptr::addr_of_mut!((*coder).check),
                    (*(*coder).block).check,
                    out.add(out_start),
                    out_used,
                );
            }
            if ret != LZMA_STREAM_END {
                return ret;
            }
            if !is_size_valid((*coder).compressed_size, (*(*coder).block).compressed_size)
                || !is_size_valid(
                    (*coder).uncompressed_size,
                    (*(*coder).block).uncompressed_size,
                )
            {
                return LZMA_DATA_ERROR;
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
            if *in_pos >= in_size {
                return LZMA_OK;
            }
            (*coder).compressed_size += 1;
            let byte = *input.add(*in_pos);
            *in_pos += 1;
            if byte != 0 {
                return LZMA_DATA_ERROR;
            }
        }
        if (*(*coder).block).check == LZMA_CHECK_NONE {
            return LZMA_STREAM_END;
        }
        if !(*coder).ignore_check {
            lzma_check_finish(
                ::core::ptr::addr_of_mut!((*coder).check),
                (*(*coder).block).check,
            );
        }
        (*coder).sequence = SEQ_CHECK;
    }
    let check_size: size_t = lzma_check_size((*(*coder).block).check) as size_t;
    lzma_bufcpy(
        input,
        in_pos,
        in_size,
        ::core::ptr::addr_of_mut!((*(*coder).block).raw_check) as *mut u8,
        ::core::ptr::addr_of_mut!((*coder).check_pos),
        check_size,
    );
    if (*coder).check_pos < check_size {
        return LZMA_OK;
    }
    if !(*coder).ignore_check
        && lzma_check_is_supported((*(*coder).block).check) != 0
        && memcmp(
            ::core::ptr::addr_of_mut!((*(*coder).block).raw_check) as *const c_void,
            ::core::ptr::addr_of_mut!((*coder).check.buffer.u8_0) as *const c_void,
            check_size,
        ) != 0
    {
        return LZMA_DATA_ERROR;
    }
    LZMA_STREAM_END
}
unsafe fn block_decoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_block_coder = coder_ptr as *mut lzma_block_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).next), allocator);
    crate::alloc::internal_free(coder, allocator);
}
pub(crate) unsafe fn lzma_block_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    block: *mut lzma_block,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *mut lzma_block) -> lzma_ret>,
        uintptr_t,
    >(Some(
        lzma_block_decoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *mut lzma_block) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *mut lzma_block) -> lzma_ret>,
        uintptr_t,
    >(Some(
        lzma_block_decoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *mut lzma_block) -> lzma_ret,
    ));
    if lzma_block_unpadded_size(block) == 0
        || !((*block).uncompressed_size <= LZMA_VLI_MAX
            || (*block).uncompressed_size == LZMA_VLI_UNKNOWN)
    {
        return LZMA_PROG_ERROR;
    }
    let mut coder: *mut lzma_block_coder = (*next).coder as *mut lzma_block_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_block_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            block_decode
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
            Some(block_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
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
    (*coder).compressed_limit = if (*block).compressed_size == LZMA_VLI_UNKNOWN {
        (LZMA_VLI_MAX & !(3))
            - (*block).header_size as lzma_vli
            - lzma_check_size((*block).check) as lzma_vli
    } else {
        (*block).compressed_size
    };
    (*coder).uncompressed_limit = if (*block).uncompressed_size == LZMA_VLI_UNKNOWN {
        LZMA_VLI_MAX
    } else {
        (*block).uncompressed_size
    };
    (*coder).check_pos = 0;
    lzma_check_init(::core::ptr::addr_of_mut!((*coder).check), (*block).check);
    (*coder).ignore_check = if (*block).version >= 1 {
        (*block).ignore_check != 0
    } else {
        false
    };
    lzma_raw_decoder_init(
        ::core::ptr::addr_of_mut!((*coder).next),
        allocator,
        (*block).filters,
    )
}
pub unsafe fn lzma_block_decoder(strm: *mut lzma_stream, block: *mut lzma_block) -> lzma_ret {
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = lzma_block_decoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        block,
    );
    if ret__0 != LZMA_OK {
        lzma_end(strm);
        return ret__0;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
