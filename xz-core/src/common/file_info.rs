use crate::common::index::{
    lzma_index_cat, lzma_index_memused, lzma_index_stream_flags, lzma_index_stream_padding,
    lzma_index_total_size,
};
use crate::common::index_decoder::lzma_index_decoder_init;
use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_file_info_coder {
    pub sequence: file_info_seq,
    pub file_cur_pos: u64,
    pub file_target_pos: u64,
    pub file_size: u64,
    pub index_decoder: lzma_next_coder,
    pub index_remaining: lzma_vli,
    pub this_index: *mut lzma_index,
    pub stream_padding: lzma_vli,
    pub combined_index: *mut lzma_index,
    pub dest_index: *mut *mut lzma_index,
    pub external_seek_pos: *mut u64,
    pub memlimit: u64,
    pub first_header_flags: lzma_stream_flags,
    pub header_flags: lzma_stream_flags,
    pub footer_flags: lzma_stream_flags,
    pub temp_pos: size_t,
    pub temp_size: size_t,
    pub temp: [u8; 8192],
}
pub type file_info_seq = c_uint;
pub const SEQ_HEADER_COMPARE: file_info_seq = 7;
pub const SEQ_HEADER_DECODE: file_info_seq = 6;
pub const SEQ_INDEX_DECODE: file_info_seq = 5;
pub const SEQ_INDEX_INIT: file_info_seq = 4;
pub const SEQ_FOOTER: file_info_seq = 3;
pub const SEQ_PADDING_DECODE: file_info_seq = 2;
pub const SEQ_PADDING_SEEK: file_info_seq = 1;
pub const SEQ_MAGIC_BYTES: file_info_seq = 0;
unsafe fn fill_temp(
    coder: *mut lzma_file_info_coder,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
) -> bool {
    (*coder).file_cur_pos += lzma_bufcpy(
        input,
        in_pos,
        in_size,
        ::core::ptr::addr_of_mut!((*coder).temp) as *mut u8,
        ::core::ptr::addr_of_mut!((*coder).temp_pos),
        (*coder).temp_size,
    ) as u64;
    (*coder).temp_pos < (*coder).temp_size
}
unsafe fn seek_to_pos(
    coder: *mut lzma_file_info_coder,
    target_pos: u64,
    in_start: size_t,
    in_pos: *mut size_t,
    in_size: size_t,
) -> bool {
    let pos_min: u64 = (*coder).file_cur_pos - (*in_pos - in_start) as u64;
    let pos_max: u64 = (*coder).file_cur_pos + (in_size - *in_pos) as u64;
    let mut external_seek_needed: bool = false;
    if target_pos >= pos_min && target_pos <= pos_max {
        // target_pos may be behind file_cur_pos (in-buffer backward seek);
        // the difference is negative modulo 2^64 and must wrap like in C.
        *in_pos = (*in_pos).wrapping_add(target_pos.wrapping_sub((*coder).file_cur_pos) as size_t);
        external_seek_needed = false;
    } else {
        *(*coder).external_seek_pos = target_pos;
        external_seek_needed = true;
        *in_pos = in_size;
    }
    (*coder).file_cur_pos = target_pos;
    external_seek_needed
}
unsafe fn reverse_seek(
    coder: *mut lzma_file_info_coder,
    in_start: size_t,
    in_pos: *mut size_t,
    in_size: size_t,
) -> lzma_ret {
    if (*coder).file_target_pos < (2 * LZMA_STREAM_HEADER_SIZE) as u64 {
        return LZMA_DATA_ERROR;
    }
    (*coder).temp_pos = 0;
    if ((*coder).file_target_pos - LZMA_STREAM_HEADER_SIZE as u64)
        < core::mem::size_of::<[u8; 8192]>() as u64
    {
        (*coder).temp_size = ((*coder).file_target_pos - LZMA_STREAM_HEADER_SIZE as u64) as size_t;
    } else {
        (*coder).temp_size = core::mem::size_of::<[u8; 8192]>() as size_t;
    }
    if seek_to_pos(
        coder,
        (*coder).file_target_pos - (*coder).temp_size as u64,
        in_start,
        in_pos,
        in_size,
    ) {
        return LZMA_SEEK_NEEDED;
    }
    LZMA_OK
}
unsafe fn get_padding_size(buf: *const u8, mut buf_size: size_t) -> size_t {
    let mut padding: size_t = 0;
    while buf_size > 0 && {
        buf_size -= 1;
        *buf.add(buf_size) == 0
    } {
        padding += 1;
    }
    padding
}
fn hide_format_error(mut ret: lzma_ret) -> lzma_ret {
    if ret == LZMA_FORMAT_ERROR {
        ret = LZMA_DATA_ERROR;
    }
    ret
}
unsafe fn decode_index(
    coder: *mut lzma_file_info_coder,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    update_file_cur_pos: bool,
) -> lzma_ret {
    let in_start: size_t = *in_pos;
    let Some(code) = (*coder).index_decoder.code else {
        return LZMA_PROG_ERROR;
    };
    let ret: lzma_ret = code(
        (*coder).index_decoder.coder,
        allocator,
        input,
        in_pos,
        in_size,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        LZMA_RUN,
    );
    (*coder).index_remaining -= (*in_pos - in_start) as lzma_vli;
    if update_file_cur_pos {
        (*coder).file_cur_pos += (*in_pos - in_start) as u64;
    }
    ret
}
unsafe fn file_info_decode(
    coder_ptr: *mut c_void,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    mut in_size: size_t,
    _out: *mut u8,
    _out_pos: *mut size_t,
    _out_size: size_t,
    _action: lzma_action,
) -> lzma_ret {
    let coder: *mut lzma_file_info_coder = coder_ptr as *mut lzma_file_info_coder;
    let in_start: size_t = *in_pos;
    if (*coder).file_size - (*coder).file_cur_pos < (in_size - in_start) as u64 {
        in_size = in_start + ((*coder).file_size - (*coder).file_cur_pos) as size_t;
    }
    loop {
        match (*coder).sequence {
            SEQ_MAGIC_BYTES => {
                if (*coder).file_size < LZMA_STREAM_HEADER_SIZE as u64 {
                    return LZMA_FORMAT_ERROR;
                }
                if fill_temp(coder, input, in_pos, in_size) {
                    return LZMA_OK;
                }
                let ret: lzma_ret = lzma_stream_header_decode(
                    &mut (*coder).first_header_flags,
                    (*coder).temp.subarray::<0, STREAM_HEADER_SIZE>(),
                );
                if ret != LZMA_OK {
                    return ret;
                }
                if (*coder).file_size > LZMA_VLI_MAX as u64 || (*coder).file_size & 3 != 0 {
                    return LZMA_DATA_ERROR;
                }
                (*coder).file_target_pos = (*coder).file_size;
                (*coder).sequence = SEQ_PADDING_SEEK;
            }
            SEQ_PADDING_SEEK => {
                (*coder).sequence = SEQ_PADDING_DECODE;
                let ret: lzma_ret = reverse_seek(coder, in_start, in_pos, in_size);
                if ret != LZMA_OK {
                    return ret;
                }
            }
            SEQ_PADDING_DECODE => {
                if fill_temp(coder, input, in_pos, in_size) {
                    return LZMA_OK;
                }
                let new_padding: size_t =
                    get_padding_size((*coder).temp.as_ptr(), (*coder).temp_size);
                (*coder).stream_padding += new_padding as lzma_vli;
                (*coder).file_target_pos -= new_padding as u64;
                if new_padding == (*coder).temp_size {
                    (*coder).sequence = SEQ_PADDING_SEEK;
                    continue;
                }
                if (*coder).stream_padding & 3 != 0 {
                    return LZMA_DATA_ERROR;
                }
                (*coder).sequence = SEQ_FOOTER;
                (*coder).temp_size -= new_padding;
                (*coder).temp_pos = (*coder).temp_size;
                if (*coder).temp_size < LZMA_STREAM_HEADER_SIZE as size_t {
                    let ret: lzma_ret = reverse_seek(coder, in_start, in_pos, in_size);
                    if ret != LZMA_OK {
                        return ret;
                    }
                }
            }
            SEQ_FOOTER => {
                if fill_temp(coder, input, in_pos, in_size) {
                    return LZMA_OK;
                }
                (*coder).file_target_pos -= LZMA_STREAM_HEADER_SIZE as u64;
                (*coder).temp_size -= LZMA_STREAM_HEADER_SIZE as size_t;
                let Some(footer) = (*coder)
                    .temp
                    .get((*coder).temp_size..)
                    .and_then(|t| t.first_chunk())
                else {
                    return LZMA_PROG_ERROR;
                };
                let ret: lzma_ret = hide_format_error(lzma_stream_footer_decode(
                    &mut (*coder).footer_flags,
                    footer,
                ));
                if ret != LZMA_OK {
                    return ret;
                }
                if (*coder).file_target_pos
                    < (*coder).footer_flags.backward_size + LZMA_STREAM_HEADER_SIZE as lzma_vli
                {
                    return LZMA_DATA_ERROR;
                }
                (*coder).file_target_pos -= (*coder).footer_flags.backward_size as u64;
                (*coder).sequence = SEQ_INDEX_INIT;
                if (*coder).temp_size as lzma_vli >= (*coder).footer_flags.backward_size {
                    (*coder).temp_pos = ((*coder).temp_size as lzma_vli
                        - (*coder).footer_flags.backward_size)
                        as size_t;
                } else {
                    (*coder).temp_pos = 0;
                    (*coder).temp_size = 0;
                    if seek_to_pos(coder, (*coder).file_target_pos, in_start, in_pos, in_size) {
                        return LZMA_SEEK_NEEDED;
                    }
                }
            }
            SEQ_INDEX_INIT => {
                let mut memused: u64 = 0;
                if !(*coder).combined_index.is_null() {
                    memused = lzma_index_memused(&*(*coder).combined_index);
                    if memused > (*coder).memlimit {
                        return LZMA_PROG_ERROR;
                    }
                }
                let ret: lzma_ret = lzma_index_decoder_init(
                    ::core::ptr::addr_of_mut!((*coder).index_decoder),
                    allocator,
                    ::core::ptr::addr_of_mut!((*coder).this_index),
                    (*coder).memlimit - memused,
                );
                if ret != LZMA_OK {
                    return ret;
                }
                (*coder).index_remaining = (*coder).footer_flags.backward_size;
                (*coder).sequence = SEQ_INDEX_DECODE;
            }
            SEQ_INDEX_DECODE => {
                let ret: lzma_ret = if (*coder).temp_size != 0 {
                    decode_index(
                        coder,
                        allocator,
                        ::core::ptr::addr_of_mut!((*coder).temp) as *mut u8,
                        ::core::ptr::addr_of_mut!((*coder).temp_pos),
                        (*coder).temp_size,
                        false,
                    )
                } else {
                    let mut in_stop: size_t = in_size;
                    if (in_size - *in_pos) as lzma_vli > (*coder).index_remaining {
                        in_stop = *in_pos + (*coder).index_remaining as size_t;
                    }
                    decode_index(coder, allocator, input, in_pos, in_stop, true)
                };
                match ret {
                    LZMA_OK => {
                        if (*coder).index_remaining == 0 {
                            return LZMA_DATA_ERROR;
                        }
                        return LZMA_OK;
                    }
                    LZMA_STREAM_END => {
                        if (*coder).index_remaining != 0 {
                            return LZMA_DATA_ERROR;
                        }
                    }
                    _ => return ret,
                }
                let seek_amount: u64 = lzma_index_total_size(&*(*coder).this_index) as u64
                    + LZMA_STREAM_HEADER_SIZE as u64;
                if (*coder).file_target_pos < seek_amount {
                    return LZMA_DATA_ERROR;
                }
                (*coder).file_target_pos -= seek_amount;
                if (*coder).file_target_pos == 0 {
                    (*coder).header_flags = (*coder).first_header_flags;
                    (*coder).sequence = SEQ_HEADER_COMPARE;
                    continue;
                }
                (*coder).sequence = SEQ_HEADER_DECODE;
                (*coder).file_target_pos += LZMA_STREAM_HEADER_SIZE as u64;
                if (*coder).temp_size != 0
                    && (*coder).temp_size as lzma_vli - (*coder).footer_flags.backward_size
                        >= seek_amount
                {
                    (*coder).temp_pos = ((*coder).temp_size as lzma_vli
                        - (*coder).footer_flags.backward_size
                        - seek_amount as lzma_vli
                        + LZMA_STREAM_HEADER_SIZE as lzma_vli)
                        as size_t;
                    (*coder).temp_size = (*coder).temp_pos;
                } else {
                    let ret_seek: lzma_ret = reverse_seek(coder, in_start, in_pos, in_size);
                    if ret_seek != LZMA_OK {
                        return ret_seek;
                    }
                }
            }
            SEQ_HEADER_DECODE => {
                if fill_temp(coder, input, in_pos, in_size) {
                    return LZMA_OK;
                }
                (*coder).file_target_pos -= LZMA_STREAM_HEADER_SIZE as u64;
                (*coder).temp_size -= LZMA_STREAM_HEADER_SIZE as size_t;
                (*coder).temp_pos = (*coder).temp_size;
                let Some(header) = (*coder)
                    .temp
                    .get((*coder).temp_size..)
                    .and_then(|t| t.first_chunk())
                else {
                    return LZMA_PROG_ERROR;
                };
                let ret: lzma_ret = hide_format_error(lzma_stream_header_decode(
                    &mut (*coder).header_flags,
                    header,
                ));
                if ret != LZMA_OK {
                    return ret;
                }
                (*coder).sequence = SEQ_HEADER_COMPARE;
            }
            SEQ_HEADER_COMPARE => {
                let ret: lzma_ret =
                    lzma_stream_flags_compare(&(*coder).header_flags, &(*coder).footer_flags);
                if ret != LZMA_OK {
                    return ret;
                }
                if lzma_index_stream_flags(
                    (*coder).this_index,
                    ::core::ptr::addr_of_mut!((*coder).footer_flags),
                ) != LZMA_OK
                {
                    return LZMA_PROG_ERROR;
                }
                if lzma_index_stream_padding((*coder).this_index, (*coder).stream_padding)
                    != LZMA_OK
                {
                    return LZMA_PROG_ERROR;
                }
                (*coder).stream_padding = 0;
                if !(*coder).combined_index.is_null() {
                    let ret: lzma_ret =
                        lzma_index_cat((*coder).this_index, (*coder).combined_index, allocator);
                    if ret != LZMA_OK {
                        return ret;
                    }
                }
                (*coder).combined_index = (*coder).this_index;
                (*coder).this_index = core::ptr::null_mut();
                if (*coder).file_target_pos == 0 {
                    *(*coder).dest_index = (*coder).combined_index;
                    (*coder).combined_index = core::ptr::null_mut();
                    *in_pos = in_size;
                    return LZMA_STREAM_END;
                }
                (*coder).sequence = (if (*coder).temp_size > 0 {
                    SEQ_PADDING_DECODE
                } else {
                    SEQ_PADDING_SEEK
                }) as file_info_seq;
            }
            _ => return LZMA_PROG_ERROR,
        }
    }
}
unsafe fn file_info_decoder_memconfig(
    coder_ptr: *mut c_void,
    memusage: *mut u64,
    old_memlimit: *mut u64,
    new_memlimit: u64,
) -> lzma_ret {
    let coder: *mut lzma_file_info_coder = coder_ptr as *mut lzma_file_info_coder;
    let mut combined_index_memusage: u64 = 0;
    let mut this_index_memusage: u64 = 0;
    if !(*coder).combined_index.is_null() {
        combined_index_memusage = lzma_index_memused(&*(*coder).combined_index);
    }
    if !(*coder).this_index.is_null() {
        this_index_memusage = lzma_index_memused(&*(*coder).this_index);
    } else if (*coder).sequence == SEQ_INDEX_DECODE {
        let mut dummy: u64 = 0;
        let Some(memconfig) = (*coder).index_decoder.memconfig else {
            return LZMA_PROG_ERROR;
        };
        if memconfig(
            (*coder).index_decoder.coder,
            ::core::ptr::addr_of_mut!(this_index_memusage),
            ::core::ptr::addr_of_mut!(dummy),
            0,
        ) != LZMA_OK
        {
            return LZMA_PROG_ERROR;
        }
    }
    *memusage = combined_index_memusage + this_index_memusage;
    if *memusage == 0 {
        *memusage = lzma_index_memusage(1, 0);
    }
    *old_memlimit = (*coder).memlimit;
    if new_memlimit != 0 {
        if new_memlimit < *memusage {
            return LZMA_MEMLIMIT_ERROR;
        }
        if (*coder).this_index.is_null() && (*coder).sequence == SEQ_INDEX_DECODE {
            let idec_new_memlimit: u64 = new_memlimit - combined_index_memusage;
            let mut dummy1: u64 = 0;
            let mut dummy2: u64 = 0;
            let Some(memconfig) = (*coder).index_decoder.memconfig else {
                return LZMA_PROG_ERROR;
            };
            if memconfig(
                (*coder).index_decoder.coder,
                ::core::ptr::addr_of_mut!(dummy1),
                ::core::ptr::addr_of_mut!(dummy2),
                idec_new_memlimit,
            ) != LZMA_OK
            {
                return LZMA_PROG_ERROR;
            }
        }
        (*coder).memlimit = new_memlimit;
    }
    LZMA_OK
}
unsafe fn file_info_decoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_file_info_coder = coder_ptr as *mut lzma_file_info_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).index_decoder), allocator);
    lzma_index_end((*coder).this_index, allocator);
    lzma_index_end((*coder).combined_index, allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn lzma_file_info_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    seek_pos: *mut u64,
    dest_index: *mut *mut lzma_index,
    memlimit: u64,
    file_size: u64,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<
            unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *mut u64,
                *mut *mut lzma_index,
                u64,
                u64,
            ) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        lzma_file_info_decoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *mut u64,
                *mut *mut lzma_index,
                u64,
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
                *mut u64,
                *mut *mut lzma_index,
                u64,
                u64,
            ) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        lzma_file_info_decoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *mut u64,
                *mut *mut lzma_index,
                u64,
                u64,
            ) -> lzma_ret,
    ));
    if dest_index.is_null() {
        return LZMA_PROG_ERROR;
    }
    let mut coder: *mut lzma_file_info_coder = (*next).coder as *mut lzma_file_info_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_file_info_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            file_info_decode
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
            Some(file_info_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*next).memconfig = Some(
            file_info_decoder_memconfig
                as unsafe fn(*mut c_void, *mut u64, *mut u64, u64) -> lzma_ret,
        );
        (*coder).index_decoder = lzma_next_coder_s {
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
        (*coder).this_index = core::ptr::null_mut();
        (*coder).combined_index = core::ptr::null_mut();
    }
    (*coder).sequence = SEQ_MAGIC_BYTES;
    (*coder).file_cur_pos = 0;
    (*coder).file_target_pos = 0;
    (*coder).file_size = file_size;
    lzma_index_end((*coder).this_index, allocator);
    (*coder).this_index = core::ptr::null_mut();
    lzma_index_end((*coder).combined_index, allocator);
    (*coder).combined_index = core::ptr::null_mut();
    (*coder).stream_padding = 0;
    (*coder).dest_index = dest_index;
    (*coder).external_seek_pos = seek_pos;
    (*coder).memlimit = if 1 > memlimit { 1 } else { memlimit };
    (*coder).temp_pos = 0;
    (*coder).temp_size = LZMA_STREAM_HEADER_SIZE as size_t;
    LZMA_OK
}
pub unsafe fn lzma_file_info_decoder(
    strm: *mut lzma_stream,
    dest_index: *mut *mut lzma_index,
    memlimit: u64,
    file_size: u64,
) -> lzma_ret {
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = lzma_file_info_decoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        ::core::ptr::addr_of_mut!((*strm).seek_pos),
        dest_index,
        memlimit,
        file_size,
    );
    if ret__0 != LZMA_OK {
        lzma_end(strm);
        return ret__0;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
