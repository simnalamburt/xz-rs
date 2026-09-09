use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_index_hash_s {
    pub sequence: index_hash_seq,
    pub blocks: lzma_index_hash_info,
    pub records: lzma_index_hash_info,
    pub remaining: lzma_vli,
    pub unpadded_size: lzma_vli,
    pub uncompressed_size: lzma_vli,
    pub pos: size_t,
    pub crc32: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_index_hash_info {
    pub blocks_size: lzma_vli,
    pub uncompressed_size: lzma_vli,
    pub count: lzma_vli,
    pub index_list_size: lzma_vli,
    pub check: lzma_check_state,
}
pub type index_hash_seq = c_uint;
pub const SEQ_CRC32: index_hash_seq = 6;
pub const SEQ_PADDING: index_hash_seq = 5;
pub const SEQ_PADDING_INIT: index_hash_seq = 4;
pub const SEQ_UNCOMPRESSED: index_hash_seq = 3;
pub const SEQ_UNPADDED: index_hash_seq = 2;
pub const SEQ_COUNT: index_hash_seq = 1;
pub const SEQ_BLOCK: index_hash_seq = 0;
pub type lzma_index_hash = lzma_index_hash_s;
#[inline]
fn index_stream_size(
    blocks_size: lzma_vli,
    count: lzma_vli,
    index_list_size: lzma_vli,
) -> lzma_vli {
    (LZMA_STREAM_HEADER_SIZE as lzma_vli)
        .wrapping_add(blocks_size)
        .wrapping_add(index_size(count, index_list_size))
        .wrapping_add(LZMA_STREAM_HEADER_SIZE as lzma_vli)
}
pub unsafe fn lzma_index_hash_init(
    mut index_hash: *mut lzma_index_hash,
    allocator: *const lzma_allocator,
) -> *mut lzma_index_hash {
    if index_hash.is_null() {
        index_hash = crate::alloc::internal_alloc_object::<lzma_index_hash>(allocator);
        if index_hash.is_null() {
            return core::ptr::null_mut();
        }
    }
    (*index_hash).sequence = SEQ_BLOCK;
    (*index_hash).blocks.blocks_size = 0;
    (*index_hash).blocks.uncompressed_size = 0;
    (*index_hash).blocks.count = 0;
    (*index_hash).blocks.index_list_size = 0;
    (*index_hash).records.blocks_size = 0;
    (*index_hash).records.uncompressed_size = 0;
    (*index_hash).records.count = 0;
    (*index_hash).records.index_list_size = 0;
    (*index_hash).unpadded_size = 0;
    (*index_hash).uncompressed_size = 0;
    (*index_hash).pos = 0;
    (*index_hash).crc32 = 0;
    lzma_check_init(
        ::core::ptr::addr_of_mut!((*index_hash).blocks.check),
        LZMA_CHECK_SHA256,
    );
    lzma_check_init(
        ::core::ptr::addr_of_mut!((*index_hash).records.check),
        LZMA_CHECK_SHA256,
    );
    index_hash
}
pub unsafe fn lzma_index_hash_end(
    index_hash: *mut lzma_index_hash,
    allocator: *const lzma_allocator,
) {
    crate::alloc::internal_free(index_hash, allocator);
}
pub fn lzma_index_hash_size(index_hash: &lzma_index_hash) -> lzma_vli {
    index_size(index_hash.blocks.count, index_hash.blocks.index_list_size)
}
unsafe fn hash_append(
    info: *mut lzma_index_hash_info,
    unpadded_size: lzma_vli,
    uncompressed_size: lzma_vli,
) {
    (*info).blocks_size = (*info).blocks_size.wrapping_add(vli_ceil4(unpadded_size));
    (*info).uncompressed_size = (*info).uncompressed_size.wrapping_add(uncompressed_size);
    (*info).index_list_size = (*info).index_list_size.wrapping_add(
        lzma_vli_size(unpadded_size).wrapping_add(lzma_vli_size(uncompressed_size)) as lzma_vli,
    );
    (*info).count = (*info).count.wrapping_add(1);
    let sizes: [lzma_vli; 2] = [unpadded_size, uncompressed_size];
    lzma_check_update(
        ::core::ptr::addr_of_mut!((*info).check),
        LZMA_CHECK_SHA256,
        ::core::ptr::addr_of!(sizes) as *const lzma_vli as *const u8,
        core::mem::size_of::<[lzma_vli; 2]>(),
    );
}
pub unsafe fn lzma_index_hash_append(
    index_hash: *mut lzma_index_hash,
    unpadded_size: lzma_vli,
    uncompressed_size: lzma_vli,
) -> lzma_ret {
    if index_hash.is_null()
        || (*index_hash).sequence != SEQ_BLOCK
        || unpadded_size < UNPADDED_SIZE_MIN
        || unpadded_size > UNPADDED_SIZE_MAX
        || uncompressed_size > LZMA_VLI_MAX
    {
        return LZMA_PROG_ERROR;
    }
    hash_append(
        ::core::ptr::addr_of_mut!((*index_hash).blocks),
        unpadded_size,
        uncompressed_size,
    );
    if (*index_hash).blocks.blocks_size > LZMA_VLI_MAX
        || (*index_hash).blocks.uncompressed_size > LZMA_VLI_MAX
        || index_size(
            (*index_hash).blocks.count,
            (*index_hash).blocks.index_list_size,
        ) > LZMA_BACKWARD_SIZE_MAX
        || index_stream_size(
            (*index_hash).blocks.blocks_size,
            (*index_hash).blocks.count,
            (*index_hash).blocks.index_list_size,
        ) > LZMA_VLI_MAX
    {
        return LZMA_DATA_ERROR;
    }
    LZMA_OK
}
pub unsafe fn lzma_index_hash_decode(
    index_hash: *mut lzma_index_hash,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
) -> lzma_ret {
    if *in_pos >= in_size {
        return LZMA_BUF_ERROR;
    }
    let in_start: size_t = *in_pos;
    let mut ret: lzma_ret = LZMA_OK;
    while *in_pos < in_size {
        match (*index_hash).sequence {
            0 => {
                let byte = *input.add(*in_pos);
                *in_pos += 1;
                if byte != INDEX_INDICATOR {
                    return LZMA_DATA_ERROR;
                }
                (*index_hash).sequence = SEQ_COUNT;
                continue;
            }
            1 => {
                ret = lzma_vli_decode(
                    &mut (*index_hash).remaining,
                    Some(&mut (*index_hash).pos),
                    c_slice(input, in_size),
                    &mut *in_pos,
                );
                if ret != LZMA_STREAM_END {
                    break;
                }
                if (*index_hash).remaining != (*index_hash).blocks.count {
                    return LZMA_DATA_ERROR;
                }
                ret = LZMA_OK;
                (*index_hash).pos = 0;
                (*index_hash).sequence = (if (*index_hash).remaining == 0 {
                    SEQ_PADDING_INIT
                } else {
                    SEQ_UNPADDED
                }) as index_hash_seq;
                continue;
            }
            2 | 3 => {
                let size = if (*index_hash).sequence == SEQ_UNPADDED {
                    &mut (*index_hash).unpadded_size
                } else {
                    &mut (*index_hash).uncompressed_size
                };
                ret = lzma_vli_decode(
                    size,
                    Some(&mut (*index_hash).pos),
                    c_slice(input, in_size),
                    &mut *in_pos,
                );
                if ret != LZMA_STREAM_END {
                    break;
                }
                ret = LZMA_OK;
                (*index_hash).pos = 0;
                if (*index_hash).sequence == SEQ_UNPADDED {
                    if (*index_hash).unpadded_size < UNPADDED_SIZE_MIN
                        || (*index_hash).unpadded_size > UNPADDED_SIZE_MAX
                    {
                        return LZMA_DATA_ERROR;
                    }
                    (*index_hash).sequence = SEQ_UNCOMPRESSED;
                } else {
                    hash_append(
                        ::core::ptr::addr_of_mut!((*index_hash).records),
                        (*index_hash).unpadded_size,
                        (*index_hash).uncompressed_size,
                    );
                    if (*index_hash).blocks.blocks_size < (*index_hash).records.blocks_size
                        || (*index_hash).blocks.uncompressed_size
                            < (*index_hash).records.uncompressed_size
                        || (*index_hash).blocks.index_list_size
                            < (*index_hash).records.index_list_size
                    {
                        return LZMA_DATA_ERROR;
                    }
                    (*index_hash).remaining -= 1;
                    (*index_hash).sequence = (if (*index_hash).remaining == 0 {
                        SEQ_PADDING_INIT
                    } else {
                        SEQ_UNPADDED
                    }) as index_hash_seq;
                }
                continue;
            }
            4 => {
                (*index_hash).pos = ((4_u64).wrapping_sub(index_size_unpadded(
                    (*index_hash).records.count,
                    (*index_hash).records.index_list_size,
                )) & 3) as size_t;
                (*index_hash).sequence = SEQ_PADDING;
                continue;
            }
            5 => {}
            6 => loop {
                if *in_pos == in_size {
                    return LZMA_OK;
                }
                let val = *input.add(*in_pos);
                *in_pos += 1;
                if (*index_hash).crc32 >> ((*index_hash).pos * 8) & 0xff != val as u32 {
                    return LZMA_DATA_ERROR;
                }
                (*index_hash).pos += 1;
                if (*index_hash).pos >= 4 {
                    return LZMA_STREAM_END;
                }
            },
            _ => return LZMA_PROG_ERROR,
        }
        if (*index_hash).pos > 0 {
            (*index_hash).pos -= 1;
            let byte = *input.add(*in_pos);
            *in_pos += 1;
            if byte != 0 {
                return LZMA_DATA_ERROR;
            }
            continue;
        }
        if (*index_hash).blocks.blocks_size != (*index_hash).records.blocks_size
            || (*index_hash).blocks.uncompressed_size != (*index_hash).records.uncompressed_size
            || (*index_hash).blocks.index_list_size != (*index_hash).records.index_list_size
        {
            return LZMA_DATA_ERROR;
        }
        lzma_check_finish(
            ::core::ptr::addr_of_mut!((*index_hash).blocks.check),
            LZMA_CHECK_SHA256,
        );
        lzma_check_finish(
            ::core::ptr::addr_of_mut!((*index_hash).records.check),
            LZMA_CHECK_SHA256,
        );
        if memcmp(
            ::core::ptr::addr_of_mut!((*index_hash).blocks.check.buffer.u8_0) as *const c_void,
            ::core::ptr::addr_of_mut!((*index_hash).records.check.buffer.u8_0) as *const c_void,
            lzma_check_size(LZMA_CHECK_SHA256) as size_t,
        ) != 0
        {
            return LZMA_DATA_ERROR;
        }
        (*index_hash).crc32 =
            lzma_crc32(input.add(in_start), *in_pos - in_start, (*index_hash).crc32);
        (*index_hash).sequence = SEQ_CRC32;
        loop {
            if *in_pos == in_size {
                return LZMA_OK;
            }
            let val = *input.add(*in_pos);
            *in_pos += 1;
            if (*index_hash).crc32 >> ((*index_hash).pos * 8) & 0xff != val as u32 {
                return LZMA_DATA_ERROR;
            }
            (*index_hash).pos += 1;
            if (*index_hash).pos >= 4 {
                break;
            }
        }
        return LZMA_STREAM_END;
    }
    let in_used: size_t = *in_pos - in_start;
    if in_used > 0 {
        (*index_hash).crc32 = lzma_crc32(input.add(in_start), in_used, (*index_hash).crc32);
    }
    ret
}
