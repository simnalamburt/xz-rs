use crate::types::*;
use core::mem::MaybeUninit;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_stream_coder {
    pub sequence: stream_encoder_seq,
    pub block_encoder_is_initialized: bool,
    pub block_encoder: lzma_next_coder,
    pub block_options: lzma_block,
    pub filters: [lzma_filter; 5],
    pub index_encoder: lzma_next_coder,
    pub index: *mut lzma_index,
    pub buffer_pos: size_t,
    pub buffer_size: size_t,
    pub buffer: [u8; LZMA_BLOCK_HEADER_SIZE_MAX as usize],
}
#[inline(always)]
unsafe fn supported_action_slot(actions: *mut bool, index: lzma_action) -> *mut bool {
    debug_assert!((index as usize) < 5);
    actions.add(index as usize)
}
pub type stream_encoder_seq = c_uint;
pub const SEQ_STREAM_FOOTER: stream_encoder_seq = 5;
pub const SEQ_INDEX_ENCODE: stream_encoder_seq = 4;
pub const SEQ_BLOCK_ENCODE: stream_encoder_seq = 3;
pub const SEQ_BLOCK_HEADER: stream_encoder_seq = 2;
pub const SEQ_BLOCK_INIT: stream_encoder_seq = 1;
pub const SEQ_STREAM_HEADER: stream_encoder_seq = 0;
unsafe fn block_encoder_init(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    (*coder).block_options.compressed_size = LZMA_VLI_UNKNOWN;
    (*coder).block_options.uncompressed_size = LZMA_VLI_UNKNOWN;
    let ret_: lzma_ret = lzma_block_header_size(&mut (*coder).block_options);
    if ret_ != LZMA_OK {
        return ret_;
    }
    lzma_block_encoder_init(
        ::core::ptr::addr_of_mut!((*coder).block_encoder),
        allocator,
        ::core::ptr::addr_of_mut!((*coder).block_options),
    )
}
unsafe fn stream_encode(
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
    #[inline(always)]
    unsafe fn convert_action(action: lzma_action) -> lzma_action {
        static CONVERT: [lzma_action; 5] = [
            LZMA_RUN,
            LZMA_SYNC_FLUSH,
            LZMA_FINISH,
            LZMA_FINISH,
            LZMA_FINISH,
        ];
        debug_assert!((action as usize) < CONVERT.len());
        *CONVERT.as_ptr().add(action as usize)
    }
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    while *out_pos < out_size {
        match (*coder).sequence {
            0 | 2 | 5 => {
                lzma_bufcpy(
                    ::core::ptr::addr_of_mut!((*coder).buffer) as *mut u8,
                    ::core::ptr::addr_of_mut!((*coder).buffer_pos),
                    (*coder).buffer_size,
                    out,
                    out_pos,
                    out_size,
                );
                if (*coder).buffer_pos < (*coder).buffer_size {
                    return LZMA_OK;
                }
                if (*coder).sequence == SEQ_STREAM_FOOTER {
                    return LZMA_STREAM_END;
                }
                (*coder).buffer_pos = 0;
                (*coder).sequence += 1;
            }
            1 => {
                if *in_pos == in_size {
                    if action != LZMA_FINISH {
                        return if action == LZMA_RUN {
                            LZMA_OK
                        } else {
                            LZMA_STREAM_END
                        };
                    }
                    let ret_: lzma_ret = lzma_index_encoder_init(
                        ::core::ptr::addr_of_mut!((*coder).index_encoder),
                        allocator,
                        (*coder).index,
                    );
                    if ret_ != LZMA_OK {
                        return ret_;
                    }
                    (*coder).sequence = SEQ_INDEX_ENCODE;
                } else {
                    if !(*coder).block_encoder_is_initialized {
                        let ret__0: lzma_ret = block_encoder_init(coder, allocator);
                        if ret__0 != LZMA_OK {
                            return ret__0;
                        }
                    }
                    (*coder).block_encoder_is_initialized = false;
                    if lzma_block_header_encode(&(*coder).block_options, &mut (*coder).buffer)
                        != LZMA_OK
                    {
                        return LZMA_PROG_ERROR;
                    }
                    (*coder).buffer_size = (*coder).block_options.header_size as size_t;
                    (*coder).sequence = SEQ_BLOCK_HEADER;
                }
            }
            3 => {
                debug_assert!((*coder).block_encoder.code.is_some());
                let code = (*coder).block_encoder.code.unwrap_unchecked();
                let ret: lzma_ret = code(
                    (*coder).block_encoder.coder,
                    allocator,
                    input,
                    in_pos,
                    in_size,
                    out,
                    out_pos,
                    out_size,
                    convert_action(action),
                );
                if ret != LZMA_STREAM_END || action == LZMA_SYNC_FLUSH {
                    return ret;
                }
                let unpadded_size: lzma_vli =
                    lzma_block_unpadded_size(::core::ptr::addr_of_mut!((*coder).block_options))
                        as lzma_vli;
                let ret__1: lzma_ret = lzma_index_append(
                    (*coder).index,
                    allocator,
                    unpadded_size,
                    (*coder).block_options.uncompressed_size,
                );
                if ret__1 != LZMA_OK {
                    return ret__1;
                }
                (*coder).sequence = SEQ_BLOCK_INIT;
            }
            4 => {
                debug_assert!((*coder).index_encoder.code.is_some());
                let code = (*coder).index_encoder.code.unwrap_unchecked();
                let ret_0: lzma_ret = code(
                    (*coder).index_encoder.coder,
                    allocator,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                    0,
                    out,
                    out_pos,
                    out_size,
                    LZMA_RUN,
                );
                if ret_0 != LZMA_STREAM_END {
                    return ret_0;
                }
                // Zeroed rather than uninit: a reference to the struct is
                // only valid if every field is, including the reserved enums
                // the encoder never reads. LZMA_RESERVED_ENUM is 0.
                let mut stream_flags = MaybeUninit::<lzma_stream_flags>::zeroed().assume_init();
                stream_flags.version = 0;
                stream_flags.backward_size = lzma_index_size(&*(*coder).index);
                stream_flags.check = (*coder).block_options.check;
                if lzma_stream_footer_encode(
                    &stream_flags,
                    (*coder).buffer.subarray_mut::<0, STREAM_HEADER_SIZE>(),
                ) != LZMA_OK
                {
                    return LZMA_PROG_ERROR;
                }
                (*coder).buffer_size = LZMA_STREAM_HEADER_SIZE as size_t;
                (*coder).sequence = SEQ_STREAM_FOOTER;
            }
            _ => return LZMA_PROG_ERROR,
        }
    }
    LZMA_OK
}
unsafe fn stream_encoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).block_encoder), allocator);
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).index_encoder), allocator);
    lzma_index_end((*coder).index, allocator);
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
        allocator,
    );
    crate::alloc::internal_free(coder, allocator);
}
#[inline(always)]
unsafe fn stream_encoder_update_before_block(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
    temp_ptr: *mut lzma_filter,
    filters_ptr: *mut lzma_filter,
) -> lzma_ret {
    (*coder).block_encoder_is_initialized = false;
    (*coder).block_options.filters = temp_ptr;
    let ret = block_encoder_init(coder, allocator);
    (*coder).block_options.filters = filters_ptr;
    if ret != LZMA_OK {
        return ret;
    }
    (*coder).block_encoder_is_initialized = true;
    LZMA_OK
}

#[inline(always)]
unsafe fn stream_encoder_update_mid_block(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter,
    reversed_filters: *const lzma_filter,
) -> lzma_ret {
    debug_assert!((*coder).block_encoder.update.is_some());
    let update = (*coder).block_encoder.update.unwrap_unchecked();
    update(
        (*coder).block_encoder.coder,
        allocator,
        filters,
        reversed_filters,
    )
}

unsafe fn stream_encoder_update(
    coder_ptr: *mut c_void,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter,
    reversed_filters: *const lzma_filter,
) -> lzma_ret {
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    let mut temp: [lzma_filter; 5] = [lzma_filter {
        id: 0,
        options: core::ptr::null_mut(),
    }; 5];
    let temp_ptr = ::core::ptr::addr_of_mut!(temp) as *mut lzma_filter;
    let filters_ptr = ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter;
    let ret_: lzma_ret = lzma_filters_copy(filters, temp_ptr, allocator);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret = if (*coder).sequence <= SEQ_BLOCK_INIT {
        stream_encoder_update_before_block(coder, allocator, temp_ptr, filters_ptr)
    } else if (*coder).sequence <= SEQ_BLOCK_ENCODE {
        stream_encoder_update_mid_block(coder, allocator, filters, reversed_filters)
    } else {
        LZMA_PROG_ERROR
    };
    if ret != LZMA_OK {
        lzma_filters_free(temp_ptr, allocator);
        return ret;
    }

    lzma_filters_free(filters_ptr, allocator);
    (*coder).filters = temp;
    LZMA_OK
}
unsafe fn stream_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter,
    check: lzma_check,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<
            unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *const lzma_filter,
                lzma_check,
            ) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        stream_encoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *const lzma_filter,
                lzma_check,
            ) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<
            unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *const lzma_filter,
                lzma_check,
            ) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        stream_encoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *const lzma_filter,
                lzma_check,
            ) -> lzma_ret,
    ));
    if filters.is_null() {
        return LZMA_PROG_ERROR;
    }
    let mut coder: *mut lzma_stream_coder = (*next).coder as *mut lzma_stream_coder;
    if coder.is_null() {
        coder = crate::common::common::lzma_alloc_object::<lzma_stream_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            stream_encode
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
            Some(stream_encoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*next).update = Some(
            stream_encoder_update
                as unsafe fn(
                    *mut c_void,
                    *const lzma_allocator,
                    *const lzma_filter,
                    *const lzma_filter,
                ) -> lzma_ret,
        );
        (*coder).filters[0].id = LZMA_VLI_UNKNOWN;
        (*coder).block_encoder = lzma_next_coder_s {
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
        (*coder).index_encoder = lzma_next_coder_s {
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
        (*coder).index = core::ptr::null_mut();
    }
    (*coder).sequence = SEQ_STREAM_HEADER;
    (*coder).block_options.version = 0;
    (*coder).block_options.check = check;
    lzma_index_end((*coder).index, allocator);
    (*coder).index = lzma_index_init(allocator);
    if (*coder).index.is_null() {
        return LZMA_MEM_ERROR;
    }
    // See the footer path: a reference needs every field valid, and
    // LZMA_RESERVED_ENUM is 0.
    let mut stream_flags = MaybeUninit::<lzma_stream_flags>::zeroed().assume_init();
    stream_flags.version = 0;
    stream_flags.check = check;
    let ret_: lzma_ret = lzma_stream_header_encode(
        &stream_flags,
        (*coder).buffer.subarray_mut::<0, STREAM_HEADER_SIZE>(),
    );
    if ret_ != LZMA_OK {
        return ret_;
    }
    (*coder).buffer_pos = 0;
    (*coder).buffer_size = LZMA_STREAM_HEADER_SIZE as size_t;
    stream_encoder_update(coder as *mut c_void, allocator, filters, core::ptr::null())
}
pub unsafe fn lzma_stream_encoder(
    strm: *mut lzma_stream,
    filters: *const lzma_filter,
    check: lzma_check,
) -> lzma_ret {
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = stream_encoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        filters,
        check,
    );
    if ret__0 != LZMA_OK {
        lzma_end(strm);
        return ret__0;
    }
    *supported_action_slot(
        ::core::ptr::addr_of_mut!((*(*strm).internal).supported_actions) as *mut bool,
        LZMA_RUN,
    ) = true;
    *supported_action_slot(
        ::core::ptr::addr_of_mut!((*(*strm).internal).supported_actions) as *mut bool,
        LZMA_SYNC_FLUSH,
    ) = true;
    *supported_action_slot(
        ::core::ptr::addr_of_mut!((*(*strm).internal).supported_actions) as *mut bool,
        LZMA_FULL_FLUSH,
    ) = true;
    *supported_action_slot(
        ::core::ptr::addr_of_mut!((*(*strm).internal).supported_actions) as *mut bool,
        LZMA_FULL_BARRIER,
    ) = true;
    *supported_action_slot(
        ::core::ptr::addr_of_mut!((*(*strm).internal).supported_actions) as *mut bool,
        LZMA_FINISH,
    ) = true;
    LZMA_OK
}
