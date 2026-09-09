use crate::common::index::{lzma_index_block_count, lzma_index_iter_init, lzma_index_iter_next};
use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_index_coder {
    pub sequence: index_encoder_seq,
    pub index: *const lzma_index,
    pub iter: lzma_index_iter,
    pub pos: size_t,
    pub crc32: u32,
}
pub type index_encoder_seq = c_uint;
pub const SEQ_CRC32: index_encoder_seq = 6;
pub const SEQ_PADDING: index_encoder_seq = 5;
pub const SEQ_NEXT: index_encoder_seq = 4;
pub const SEQ_UNCOMPRESSED: index_encoder_seq = 3;
pub const SEQ_UNPADDED: index_encoder_seq = 2;
pub const SEQ_COUNT: index_encoder_seq = 1;
pub const SEQ_INDICATOR: index_encoder_seq = 0;
unsafe fn index_encode(
    coder_ptr: *mut c_void,
    _allocator: *const lzma_allocator,
    _in_0: *const u8,
    _in_pos: *mut size_t,
    _in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    _action: lzma_action,
) -> lzma_ret {
    let coder: *mut lzma_index_coder = coder_ptr as *mut lzma_index_coder;
    let out_start: size_t = *out_pos;
    let mut ret: lzma_ret = LZMA_OK;
    while *out_pos < out_size {
        match (*coder).sequence {
            0 => {
                *out.add(*out_pos) = INDEX_INDICATOR;
                *out_pos += 1;
                (*coder).sequence = SEQ_COUNT;
                continue;
            }
            1 => {
                let count: lzma_vli = lzma_index_block_count(&*(*coder).index) as lzma_vli;
                ret = lzma_vli_encode(
                    count,
                    Some(&mut (*coder).pos),
                    c_slice_mut(out, out_size),
                    &mut *out_pos,
                );
                if ret != LZMA_STREAM_END {
                    break;
                }
                ret = LZMA_OK;
                (*coder).pos = 0;
                (*coder).sequence = SEQ_NEXT;
                continue;
            }
            4 => {
                if lzma_index_iter_next(&mut (*coder).iter, LZMA_INDEX_ITER_BLOCK) != 0 {
                    (*coder).pos = lzma_index_padding_size(&*(*coder).index) as size_t;
                    (*coder).sequence = SEQ_PADDING;
                    continue;
                } else {
                    (*coder).sequence = SEQ_UNPADDED;
                }
            }
            2 | 3 => {}
            5 => {
                if (*coder).pos > 0 {
                    (*coder).pos -= 1;
                    *out.add(*out_pos) = 0;
                    *out_pos += 1;
                    continue;
                } else {
                    (*coder).crc32 =
                        lzma_crc32(out.add(out_start), *out_pos - out_start, (*coder).crc32);
                    (*coder).sequence = SEQ_CRC32;
                }
            }
            6 => {}
            _ => return LZMA_PROG_ERROR,
        }
        if (*coder).sequence == SEQ_UNPADDED || (*coder).sequence == SEQ_UNCOMPRESSED {
            let size: lzma_vli = if (*coder).sequence == SEQ_UNPADDED {
                (*coder).iter.block.unpadded_size
            } else {
                (*coder).iter.block.uncompressed_size
            };
            ret = lzma_vli_encode(
                size,
                Some(&mut (*coder).pos),
                c_slice_mut(out, out_size),
                &mut *out_pos,
            );
            if ret != LZMA_STREAM_END {
                break;
            }
            ret = LZMA_OK;
            (*coder).pos = 0;
            (*coder).sequence += 1;
        } else {
            loop {
                if *out_pos == out_size {
                    return LZMA_OK;
                }
                *out.add(*out_pos) = ((*coder).crc32 >> ((*coder).pos * 8) & 0xff) as u8;
                *out_pos += 1;
                (*coder).pos += 1;
                if (*coder).pos >= 4 {
                    break;
                }
            }
            return LZMA_STREAM_END;
        }
    }
    let out_used: size_t = *out_pos - out_start;
    if out_used > 0 {
        (*coder).crc32 = lzma_crc32(out.add(out_start), out_used, (*coder).crc32);
    }
    ret
}
unsafe fn index_encoder_end(coder: *mut c_void, allocator: *const lzma_allocator) {
    crate::alloc::internal_free(coder as *mut lzma_index_coder, allocator);
}
unsafe fn index_encoder_reset(coder: *mut lzma_index_coder, i: *const lzma_index) {
    lzma_index_iter_init(&mut (*coder).iter, &*i);
    (*coder).sequence = SEQ_INDICATOR;
    (*coder).index = i;
    (*coder).pos = 0;
    (*coder).crc32 = 0;
}
pub(crate) unsafe fn lzma_index_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    i: *const lzma_index,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<
            unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_index) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        lzma_index_encoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *const lzma_index,
            ) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<
            unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_index) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        lzma_index_encoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *const lzma_index,
            ) -> lzma_ret,
    ));
    if i.is_null() {
        return LZMA_PROG_ERROR;
    }
    if (*next).coder.is_null() {
        (*next).coder =
            crate::alloc::internal_alloc_object::<lzma_index_coder>(allocator) as *mut c_void;
        if (*next).coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).code = Some(
            index_encode
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
            Some(index_encoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
    }
    index_encoder_reset((*next).coder as *mut lzma_index_coder, i);
    LZMA_OK
}
pub unsafe fn lzma_index_encoder(strm: *mut lzma_stream, i: *const lzma_index) -> lzma_ret {
    let ret: lzma_ret = lzma_strm_init(strm);
    if ret != LZMA_OK {
        return ret;
    }
    let ret: lzma_ret = lzma_index_encoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        i,
    );
    if ret != LZMA_OK {
        lzma_end(strm);
        return ret;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
pub unsafe fn lzma_index_buffer_encode(
    i: *const lzma_index,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    if i.is_null() || out.is_null() || out_pos.is_null() || *out_pos > out_size {
        return LZMA_PROG_ERROR;
    }
    if ((out_size - *out_pos) as lzma_vli) < lzma_index_size(&*i) {
        return LZMA_BUF_ERROR;
    }
    let mut coder: lzma_index_coder = lzma_index_coder {
        sequence: SEQ_INDICATOR,
        index: core::ptr::null(),
        iter: lzma_index_iter {
            stream: lzma_index_iter_stream {
                flags: core::ptr::null(),
                reserved_ptr1: core::ptr::null(),
                reserved_ptr2: core::ptr::null(),
                reserved_ptr3: core::ptr::null(),
                number: 0,
                block_count: 0,
                compressed_offset: 0,
                uncompressed_offset: 0,
                compressed_size: 0,
                uncompressed_size: 0,
                padding: 0,
                reserved_vli1: 0,
                reserved_vli2: 0,
                reserved_vli3: 0,
                reserved_vli4: 0,
            },
            block: lzma_index_iter_block {
                number_in_file: 0,
                compressed_file_offset: 0,
                uncompressed_file_offset: 0,
                number_in_stream: 0,
                compressed_stream_offset: 0,
                uncompressed_stream_offset: 0,
                uncompressed_size: 0,
                unpadded_size: 0,
                total_size: 0,
                reserved_vli1: 0,
                reserved_vli2: 0,
                reserved_vli3: 0,
                reserved_vli4: 0,
                reserved_ptr1: core::ptr::null(),
                reserved_ptr2: core::ptr::null(),
                reserved_ptr3: core::ptr::null(),
                reserved_ptr4: core::ptr::null(),
            },
            internal: [lzma_index_iter_internal {
                p: core::ptr::null(),
            }; 6],
        },
        pos: 0,
        crc32: 0,
    };
    index_encoder_reset(::core::ptr::addr_of_mut!(coder), i);
    let out_start: size_t = *out_pos;
    let mut ret: lzma_ret = index_encode(
        ::core::ptr::addr_of_mut!(coder) as *mut c_void,
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null_mut(),
        0,
        out,
        out_pos,
        out_size,
        LZMA_RUN,
    );
    if ret == LZMA_STREAM_END {
        ret = LZMA_OK;
    } else {
        *out_pos = out_start;
        ret = LZMA_PROG_ERROR;
    }
    ret
}
