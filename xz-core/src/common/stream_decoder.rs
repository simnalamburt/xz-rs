use crate::types::*;
use core::mem::MaybeUninit;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_stream_coder {
    pub sequence: stream_decoder_seq,
    pub block_decoder: lzma_next_coder,
    pub block_options: lzma_block,
    pub stream_flags: lzma_stream_flags,
    pub index_hash: *mut lzma_index_hash,
    pub memlimit: u64,
    pub memusage: u64,
    pub tell_no_check: bool,
    pub tell_unsupported_check: bool,
    pub tell_any_check: bool,
    pub ignore_check: bool,
    pub concatenated: bool,
    pub first_stream: bool,
    pub pos: size_t,
    pub buffer: [u8; LZMA_BLOCK_HEADER_SIZE_MAX as usize],
}
pub type stream_decoder_seq = c_uint;
pub const SEQ_STREAM_PADDING: stream_decoder_seq = 6;
pub const SEQ_STREAM_FOOTER: stream_decoder_seq = 5;
pub const SEQ_INDEX: stream_decoder_seq = 4;
pub const SEQ_BLOCK_RUN: stream_decoder_seq = 3;
pub const SEQ_BLOCK_INIT: stream_decoder_seq = 2;
pub const SEQ_BLOCK_HEADER: stream_decoder_seq = 1;
pub const SEQ_STREAM_HEADER: stream_decoder_seq = 0;
unsafe fn stream_decoder_reset(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    (*coder).index_hash = lzma_index_hash_init((*coder).index_hash, allocator);
    if (*coder).index_hash.is_null() {
        return LZMA_MEM_ERROR;
    }
    (*coder).sequence = SEQ_STREAM_HEADER;
    (*coder).pos = 0;
    LZMA_OK
}
unsafe fn stream_decode(
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
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    loop {
        match (*coder).sequence {
            SEQ_STREAM_HEADER => {
                lzma_bufcpy(
                    input,
                    in_pos,
                    in_size,
                    ::core::ptr::addr_of_mut!((*coder).buffer) as *mut u8,
                    ::core::ptr::addr_of_mut!((*coder).pos),
                    LZMA_STREAM_HEADER_SIZE as size_t,
                );
                if (*coder).pos < LZMA_STREAM_HEADER_SIZE as size_t {
                    return LZMA_OK;
                }
                (*coder).pos = 0;
                let ret: lzma_ret = lzma_stream_header_decode(
                    &mut (*coder).stream_flags,
                    (*coder).buffer.subarray::<0, STREAM_HEADER_SIZE>(),
                );
                if ret != LZMA_OK {
                    return if ret == LZMA_FORMAT_ERROR && !(*coder).first_stream {
                        LZMA_DATA_ERROR
                    } else {
                        ret
                    };
                }
                (*coder).first_stream = false;
                (*coder).block_options.check = (*coder).stream_flags.check;
                (*coder).sequence = SEQ_BLOCK_HEADER;
                if (*coder).tell_no_check && (*coder).stream_flags.check == LZMA_CHECK_NONE {
                    return LZMA_NO_CHECK;
                }
                if (*coder).tell_unsupported_check
                    && lzma_check_is_supported((*coder).stream_flags.check) == 0
                {
                    return LZMA_UNSUPPORTED_CHECK;
                }
                if (*coder).tell_any_check {
                    return LZMA_GET_CHECK;
                }
                continue;
            }
            SEQ_BLOCK_HEADER => {
                if *in_pos >= in_size {
                    return LZMA_OK;
                }
                if (*coder).pos == 0 {
                    if *input.add(*in_pos) == INDEX_INDICATOR {
                        (*coder).sequence = SEQ_INDEX;
                        continue;
                    }
                    (*coder).block_options.header_size = ((*input.add(*in_pos) as u32) + 1) * 4;
                }

                lzma_bufcpy(
                    input,
                    in_pos,
                    in_size,
                    ::core::ptr::addr_of_mut!((*coder).buffer) as *mut u8,
                    ::core::ptr::addr_of_mut!((*coder).pos),
                    (*coder).block_options.header_size as size_t,
                );
                if (*coder).pos < (*coder).block_options.header_size as size_t {
                    return LZMA_OK;
                }

                (*coder).pos = 0;
                (*coder).sequence = SEQ_BLOCK_INIT;
                continue;
            }
            SEQ_BLOCK_INIT => {
                (*coder).block_options.version = 1;
                let mut filters = MaybeUninit::<[lzma_filter; 5]>::uninit();
                let filters_ptr = filters.as_mut_ptr() as *mut lzma_filter;
                (*coder).block_options.filters = filters_ptr;
                let ret: lzma_ret = lzma_block_header_decode(
                    &mut (*coder).block_options,
                    allocator,
                    &(*coder).buffer,
                );
                if ret != LZMA_OK {
                    lzma_filters_free(filters_ptr, allocator);
                    (*coder).block_options.filters = core::ptr::null_mut();
                    return ret;
                }

                (*coder).block_options.ignore_check = (*coder).ignore_check as lzma_bool;

                let memusage: u64 = lzma_raw_decoder_memusage(filters_ptr) as u64;
                let ret = if memusage == UINT64_MAX {
                    LZMA_OPTIONS_ERROR
                } else {
                    (*coder).memusage = memusage;
                    if memusage > (*coder).memlimit {
                        LZMA_MEMLIMIT_ERROR
                    } else {
                        lzma_block_decoder_init(
                            ::core::ptr::addr_of_mut!((*coder).block_decoder),
                            allocator,
                            ::core::ptr::addr_of_mut!((*coder).block_options),
                        )
                    }
                };

                lzma_filters_free(filters_ptr, allocator);
                (*coder).block_options.filters = core::ptr::null_mut();
                if ret != LZMA_OK {
                    return ret;
                }

                (*coder).sequence = SEQ_BLOCK_RUN;
                continue;
            }
            SEQ_BLOCK_RUN => {
                debug_assert!((*coder).block_decoder.code.is_some());
                let code = (*coder).block_decoder.code.unwrap_unchecked();
                let ret: lzma_ret = code(
                    (*coder).block_decoder.coder,
                    allocator,
                    input,
                    in_pos,
                    in_size,
                    out,
                    out_pos,
                    out_size,
                    action,
                );
                if ret != LZMA_STREAM_END {
                    return ret;
                }

                let ret: lzma_ret = lzma_index_hash_append(
                    (*coder).index_hash,
                    lzma_block_unpadded_size(::core::ptr::addr_of_mut!((*coder).block_options)),
                    (*coder).block_options.uncompressed_size,
                );
                if ret != LZMA_OK {
                    return ret;
                }

                (*coder).sequence = SEQ_BLOCK_HEADER;
            }
            SEQ_INDEX => {
                if *in_pos >= in_size {
                    return LZMA_OK;
                }
                let ret: lzma_ret =
                    lzma_index_hash_decode((*coder).index_hash, input, in_pos, in_size);
                if ret != LZMA_STREAM_END {
                    return ret;
                }
                (*coder).sequence = SEQ_STREAM_FOOTER;
                continue;
            }
            SEQ_STREAM_FOOTER => {
                lzma_bufcpy(
                    input,
                    in_pos,
                    in_size,
                    ::core::ptr::addr_of_mut!((*coder).buffer) as *mut u8,
                    ::core::ptr::addr_of_mut!((*coder).pos),
                    LZMA_STREAM_HEADER_SIZE as size_t,
                );
                if (*coder).pos < LZMA_STREAM_HEADER_SIZE as size_t {
                    return LZMA_OK;
                }

                (*coder).pos = 0;
                let mut footer_flags = MaybeUninit::<lzma_stream_flags>::zeroed();
                let ret: lzma_ret = lzma_stream_footer_decode(
                    footer_flags.assume_init_mut(),
                    (*coder).buffer.subarray::<0, STREAM_HEADER_SIZE>(),
                );
                if ret != LZMA_OK {
                    return if ret == LZMA_FORMAT_ERROR {
                        LZMA_DATA_ERROR
                    } else {
                        ret
                    };
                }
                let footer_flags = footer_flags.assume_init();
                if lzma_index_hash_size(&*(*coder).index_hash) != footer_flags.backward_size {
                    return LZMA_DATA_ERROR;
                }
                let ret: lzma_ret =
                    lzma_stream_flags_compare(&(*coder).stream_flags, &footer_flags);
                if ret != LZMA_OK {
                    return ret;
                }
                if !(*coder).concatenated {
                    return LZMA_STREAM_END;
                }
                (*coder).sequence = SEQ_STREAM_PADDING;
                continue;
            }
            SEQ_STREAM_PADDING => {
                loop {
                    if *in_pos >= in_size {
                        if action != LZMA_FINISH {
                            return LZMA_OK;
                        }
                        return if (*coder).pos == 0 {
                            LZMA_STREAM_END
                        } else {
                            LZMA_DATA_ERROR
                        };
                    }
                    if *input.add(*in_pos) != 0 {
                        break;
                    }
                    *in_pos += 1;
                    (*coder).pos = ((*coder).pos + 1) & 3;
                }

                if (*coder).pos != 0 {
                    *in_pos += 1;
                    return LZMA_DATA_ERROR;
                }

                let ret: lzma_ret = stream_decoder_reset(coder, allocator);
                if ret != LZMA_OK {
                    return ret;
                }
            }
            _ => return LZMA_PROG_ERROR,
        }
    }
}
unsafe fn stream_decoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).block_decoder), allocator);
    lzma_index_hash_end((*coder).index_hash, allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn stream_decoder_get_check(coder_ptr: *const c_void) -> lzma_check {
    let coder: *const lzma_stream_coder = coder_ptr as *const lzma_stream_coder;
    (*coder).stream_flags.check
}
unsafe fn stream_decoder_memconfig(
    coder_ptr: *mut c_void,
    memusage: *mut u64,
    old_memlimit: *mut u64,
    new_memlimit: u64,
) -> lzma_ret {
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    *memusage = (*coder).memusage;
    *old_memlimit = (*coder).memlimit;
    if new_memlimit != 0 {
        if new_memlimit < (*coder).memusage {
            return LZMA_MEMLIMIT_ERROR;
        }
        (*coder).memlimit = new_memlimit;
    }
    LZMA_OK
}
pub(crate) unsafe fn lzma_stream_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    memlimit: u64,
    flags: u32,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret>,
        uintptr_t,
    >(Some(
        lzma_stream_decoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret>,
        uintptr_t,
    >(Some(
        lzma_stream_decoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret,
    ));
    if flags & !(LZMA_SUPPORTED_FLAGS as u32) != 0 {
        return LZMA_OPTIONS_ERROR;
    }
    let mut coder: *mut lzma_stream_coder = (*next).coder as *mut lzma_stream_coder;
    if coder.is_null() {
        coder = crate::common::common::lzma_alloc_object::<lzma_stream_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            stream_decode
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
            Some(stream_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*next).get_check =
            Some(stream_decoder_get_check as unsafe fn(*const c_void) -> lzma_check);
        (*next).memconfig = Some(
            stream_decoder_memconfig as unsafe fn(*mut c_void, *mut u64, *mut u64, u64) -> lzma_ret,
        );
        (*coder).block_decoder = lzma_next_coder_s {
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
        (*coder).index_hash = core::ptr::null_mut();
    }
    (*coder).memlimit = if 1 > memlimit { 1 } else { memlimit };
    (*coder).memusage = LZMA_MEMUSAGE_BASE;
    (*coder).tell_no_check = flags & LZMA_TELL_NO_CHECK as u32 != 0;
    (*coder).tell_unsupported_check = flags & LZMA_TELL_UNSUPPORTED_CHECK as u32 != 0;
    (*coder).tell_any_check = flags & LZMA_TELL_ANY_CHECK as u32 != 0;
    (*coder).ignore_check = flags & LZMA_IGNORE_CHECK as u32 != 0;
    (*coder).concatenated = flags & LZMA_CONCATENATED as u32 != 0;
    (*coder).first_stream = true;
    stream_decoder_reset(coder, allocator)
}
pub unsafe fn lzma_stream_decoder(strm: *mut lzma_stream, memlimit: u64, flags: u32) -> lzma_ret {
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = lzma_stream_decoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        memlimit,
        flags,
    );
    if ret__0 != LZMA_OK {
        lzma_end(strm);
        return ret__0;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
