use crate::common::index::lzma_index_prealloc;
use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_index_coder {
    pub sequence: index_decoder_seq,
    pub memlimit: u64,
    pub index: *mut lzma_index,
    pub index_ptr: *mut *mut lzma_index,
    pub count: lzma_vli,
    pub unpadded_size: lzma_vli,
    pub uncompressed_size: lzma_vli,
    pub pos: size_t,
    pub crc32: u32,
}
pub type index_decoder_seq = c_uint;
pub const SEQ_CRC32: index_decoder_seq = 7;
pub const SEQ_PADDING: index_decoder_seq = 6;
pub const SEQ_PADDING_INIT: index_decoder_seq = 5;
pub const SEQ_UNCOMPRESSED: index_decoder_seq = 4;
pub const SEQ_UNPADDED: index_decoder_seq = 3;
pub const SEQ_MEMUSAGE: index_decoder_seq = 2;
pub const SEQ_COUNT: index_decoder_seq = 1;
pub const SEQ_INDICATOR: index_decoder_seq = 0;
unsafe fn index_decode(
    coder_ptr: *mut c_void,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    _out: *mut u8,
    _out_pos: *mut size_t,
    _out_size: size_t,
    _action: lzma_action,
) -> lzma_ret {
    let coder: *mut lzma_index_coder = coder_ptr as *mut lzma_index_coder;
    let in_start: size_t = *in_pos;
    let mut ret: lzma_ret = LZMA_OK;

    while *in_pos < in_size {
        loop {
            match (*coder).sequence {
                SEQ_INDICATOR => {
                    let byte = *input.add(*in_pos);
                    *in_pos += 1;
                    if byte != INDEX_INDICATOR {
                        return LZMA_DATA_ERROR;
                    }

                    (*coder).sequence = SEQ_COUNT;
                    break;
                }

                SEQ_COUNT => {
                    ret = lzma_vli_decode(
                        &mut (*coder).count,
                        Some(&mut (*coder).pos),
                        c_slice(input, in_size),
                        &mut *in_pos,
                    );
                    if ret != LZMA_STREAM_END {
                        return goto_out(coder, input, in_start, in_pos, ret);
                    }

                    (*coder).pos = 0;
                    (*coder).sequence = SEQ_MEMUSAGE;
                    continue;
                }

                SEQ_MEMUSAGE => {
                    if lzma_index_memusage(1, (*coder).count) > (*coder).memlimit {
                        ret = LZMA_MEMLIMIT_ERROR;
                        return goto_out(coder, input, in_start, in_pos, ret);
                    }

                    lzma_index_prealloc(&mut *(*coder).index, (*coder).count);
                    ret = LZMA_OK;
                    (*coder).sequence = if (*coder).count == 0 {
                        SEQ_PADDING_INIT
                    } else {
                        SEQ_UNPADDED
                    };
                    break;
                }

                SEQ_UNPADDED | SEQ_UNCOMPRESSED => {
                    let size = if (*coder).sequence == SEQ_UNPADDED {
                        &mut (*coder).unpadded_size
                    } else {
                        &mut (*coder).uncompressed_size
                    };

                    ret = lzma_vli_decode(
                        size,
                        Some(&mut (*coder).pos),
                        c_slice(input, in_size),
                        &mut *in_pos,
                    );
                    if ret != LZMA_STREAM_END {
                        return goto_out(coder, input, in_start, in_pos, ret);
                    }

                    ret = LZMA_OK;
                    (*coder).pos = 0;

                    if (*coder).sequence == SEQ_UNPADDED {
                        if (*coder).unpadded_size < UNPADDED_SIZE_MIN
                            || (*coder).unpadded_size > UNPADDED_SIZE_MAX
                        {
                            return LZMA_DATA_ERROR;
                        }

                        (*coder).sequence = SEQ_UNCOMPRESSED;
                    } else {
                        let ret_ = lzma_index_append(
                            (*coder).index,
                            allocator,
                            (*coder).unpadded_size,
                            (*coder).uncompressed_size,
                        );
                        if ret_ != LZMA_OK {
                            return ret_;
                        }

                        (*coder).count -= 1;
                        (*coder).sequence = if (*coder).count == 0 {
                            SEQ_PADDING_INIT
                        } else {
                            SEQ_UNPADDED
                        };
                    }

                    break;
                }

                SEQ_PADDING_INIT => {
                    (*coder).pos = lzma_index_padding_size(&*(*coder).index) as size_t;
                    (*coder).sequence = SEQ_PADDING;
                    continue;
                }

                SEQ_PADDING => {
                    if (*coder).pos > 0 {
                        (*coder).pos -= 1;
                        let byte = *input.add(*in_pos);
                        *in_pos += 1;
                        if byte != 0x00 {
                            return LZMA_DATA_ERROR;
                        }

                        break;
                    }

                    (*coder).crc32 =
                        lzma_crc32(input.add(in_start), *in_pos - in_start, (*coder).crc32);
                    (*coder).sequence = SEQ_CRC32;
                    continue;
                }

                SEQ_CRC32 => {
                    loop {
                        if *in_pos == in_size {
                            return LZMA_OK;
                        }

                        let byte = *input.add(*in_pos);
                        *in_pos += 1;
                        if ((*coder).crc32 >> ((*coder).pos * 8)) & 0xff != byte as u32 {
                            return LZMA_DATA_ERROR;
                        }

                        (*coder).pos += 1;
                        if (*coder).pos >= 4 {
                            break;
                        }
                    }

                    *(*coder).index_ptr = (*coder).index;
                    (*coder).index = core::ptr::null_mut();
                    return LZMA_STREAM_END;
                }

                _ => return LZMA_PROG_ERROR,
            }
        }
    }

    goto_out(coder, input, in_start, in_pos, ret)
}

#[inline(always)]
unsafe fn goto_out(
    coder: *mut lzma_index_coder,
    input: *const u8,
    in_start: size_t,
    in_pos: *mut size_t,
    ret: lzma_ret,
) -> lzma_ret {
    let in_used = *in_pos - in_start;
    if in_used > 0 {
        (*coder).crc32 = lzma_crc32(input.add(in_start), in_used, (*coder).crc32);
    }

    ret
}
unsafe fn index_decoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_index_coder = coder_ptr as *mut lzma_index_coder;
    lzma_index_end((*coder).index, allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn index_decoder_memconfig(
    coder_ptr: *mut c_void,
    memusage: *mut u64,
    old_memlimit: *mut u64,
    new_memlimit: u64,
) -> lzma_ret {
    let coder: *mut lzma_index_coder = coder_ptr as *mut lzma_index_coder;
    *memusage = lzma_index_memusage(1, (*coder).count);
    *old_memlimit = (*coder).memlimit;
    if new_memlimit != 0 {
        if new_memlimit < *memusage {
            return LZMA_MEMLIMIT_ERROR;
        }
        (*coder).memlimit = new_memlimit;
    }
    LZMA_OK
}
unsafe fn index_decoder_reset(
    coder: *mut lzma_index_coder,
    allocator: *const lzma_allocator,
    i: *mut *mut lzma_index,
    memlimit: u64,
) -> lzma_ret {
    (*coder).index_ptr = i;
    *i = core::ptr::null_mut();
    (*coder).index = lzma_index_init(allocator);
    if (*coder).index.is_null() {
        return LZMA_MEM_ERROR;
    }
    (*coder).sequence = SEQ_INDICATOR;
    (*coder).memlimit = if 1 > memlimit { 1 } else { memlimit };
    (*coder).count = 0;
    (*coder).pos = 0;
    (*coder).crc32 = 0;
    LZMA_OK
}
pub(crate) unsafe fn lzma_index_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    i: *mut *mut lzma_index,
    memlimit: u64,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<
            unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *mut *mut lzma_index,
                u64,
            ) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        lzma_index_decoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *mut *mut lzma_index,
                u64,
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
                *mut *mut lzma_index,
                u64,
            ) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        lzma_index_decoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *mut *mut lzma_index,
                u64,
            ) -> lzma_ret,
    ));
    if i.is_null() {
        return LZMA_PROG_ERROR;
    }
    let mut coder: *mut lzma_index_coder = (*next).coder as *mut lzma_index_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_index_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            index_decode
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
            Some(index_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*next).memconfig = Some(
            index_decoder_memconfig as unsafe fn(*mut c_void, *mut u64, *mut u64, u64) -> lzma_ret,
        );
        (*coder).index = core::ptr::null_mut();
    } else {
        lzma_index_end((*coder).index, allocator);
    }
    index_decoder_reset(coder, allocator, i, memlimit)
}
pub unsafe fn lzma_index_decoder(
    strm: *mut lzma_stream,
    i: *mut *mut lzma_index,
    memlimit: u64,
) -> lzma_ret {
    if !i.is_null() {
        *i = core::ptr::null_mut();
    }
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = lzma_index_decoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        i,
        memlimit,
    );
    if ret__0 != LZMA_OK {
        lzma_end(strm);
        return ret__0;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
pub unsafe fn lzma_index_buffer_decode(
    i: *mut *mut lzma_index,
    memlimit: *mut u64,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
) -> lzma_ret {
    if !i.is_null() {
        *i = core::ptr::null_mut();
    }
    if i.is_null() || memlimit.is_null() || input.is_null() || in_pos.is_null() || *in_pos > in_size
    {
        return LZMA_PROG_ERROR;
    }
    let mut coder: lzma_index_coder = lzma_index_coder {
        sequence: SEQ_INDICATOR,
        memlimit: 0,
        index: core::ptr::null_mut(),
        index_ptr: core::ptr::null_mut(),
        count: 0,
        unpadded_size: 0,
        uncompressed_size: 0,
        pos: 0,
        crc32: 0,
    };
    let ret_: lzma_ret =
        index_decoder_reset(::core::ptr::addr_of_mut!(coder), allocator, i, *memlimit);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let in_start: size_t = *in_pos;
    let mut ret: lzma_ret = index_decode(
        ::core::ptr::addr_of_mut!(coder) as *mut c_void,
        allocator,
        input,
        in_pos,
        in_size,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        LZMA_RUN,
    );
    if ret == LZMA_STREAM_END {
        ret = LZMA_OK;
    } else {
        lzma_index_end(coder.index, allocator);
        *in_pos = in_start;
        if ret == LZMA_OK {
            ret = LZMA_DATA_ERROR;
        } else if ret == LZMA_MEMLIMIT_ERROR {
            *memlimit = lzma_index_memusage(1, coder.count);
        }
    }
    ret
}
