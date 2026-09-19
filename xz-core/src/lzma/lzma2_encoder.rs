use crate::lz::lz_encoder::{lzma_lz_encoder_init, lzma_lz_options};
use crate::lzma::lzma_encoder::{
    lzma_lzma_encode, lzma_lzma_encoder_create, lzma_lzma_encoder_reset,
};
use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_lzma2_coder {
    pub sequence: lzma2_encoder_seq,
    pub lzma: *mut c_void,
    pub opt_cur: lzma_options_lzma,
    pub need_properties: bool,
    pub need_state_reset: bool,
    pub need_dictionary_reset: bool,
    pub uncompressed_size: size_t,
    pub compressed_size: size_t,
    pub buf_pos: size_t,
    pub buf: [u8; 65542],
}
pub type lzma2_encoder_seq = c_uint;
pub const SEQ_UNCOMPRESSED_COPY: lzma2_encoder_seq = 4;
pub const SEQ_UNCOMPRESSED_HEADER: lzma2_encoder_seq = 3;
pub const SEQ_LZMA_COPY: lzma2_encoder_seq = 2;
pub const SEQ_LZMA_ENCODE: lzma2_encoder_seq = 1;
pub const SEQ_INIT: lzma2_encoder_seq = 0;
#[inline]
unsafe fn mf_unencoded(mf: *const lzma_mf) -> u32 {
    (*mf).write_pos - (*mf).read_pos + (*mf).read_ahead
}
#[inline]
unsafe fn mf_read(
    mf: *mut lzma_mf,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    left: *mut size_t,
) {
    let out_avail: size_t = out_size - *out_pos;
    let copy_size: size_t = if out_avail < *left { out_avail } else { *left };
    core::ptr::copy_nonoverlapping(
        (*mf)
            .buffer
            .offset((*mf).read_pos as isize)
            .offset(-(*left as isize)) as *const u8,
        out.add(*out_pos) as *mut u8,
        copy_size,
    );
    *out_pos += copy_size;
    *left -= copy_size;
}
pub const LZMA2_UNCOMPRESSED_MAX: c_uint = 1u32 << 21;
pub const LZMA2_HEADER_MAX: u32 = 6;
unsafe fn lzma2_header_lzma(coder: *mut lzma_lzma2_coder) {
    let mut pos: size_t = 0;
    if (*coder).need_properties {
        pos = 0;
        if (*coder).need_dictionary_reset {
            (*coder).buf[pos as usize] = (0x80 + ((3) << 5)) as u8;
        } else {
            (*coder).buf[pos as usize] = (0x80 + ((2) << 5)) as u8;
        }
    } else {
        pos = 1;
        if (*coder).need_state_reset {
            (*coder).buf[pos as usize] = (0x80 + (1 << 5)) as u8;
        } else {
            (*coder).buf[pos as usize] = 0x80 as u8;
        }
    }
    (*coder).buf_pos = pos;
    let mut size: size_t = (*coder).uncompressed_size - 1;
    (*coder).buf[pos as usize] = ((*coder).buf[pos as usize] as size_t + (size >> 16)) as u8;
    pos += 1;
    (*coder).buf[pos as usize] = (size >> 8 & 0xff) as u8;
    pos += 1;
    (*coder).buf[pos as usize] = (size & 0xff) as u8;
    pos += 1;
    size = (*coder).compressed_size - 1;
    (*coder).buf[pos as usize] = (size >> 8) as u8;
    pos += 1;
    (*coder).buf[pos as usize] = (size & 0xff) as u8;
    pos += 1;
    if (*coder).need_properties {
        lzma_lzma_lclppb_encode(
            ::core::ptr::addr_of_mut!((*coder).opt_cur),
            (::core::ptr::addr_of_mut!((*coder).buf) as *mut u8).add(pos),
        );
    }
    (*coder).need_properties = false;
    (*coder).need_state_reset = false;
    (*coder).need_dictionary_reset = false;
    (*coder).compressed_size += LZMA2_HEADER_MAX as size_t;
}
unsafe fn lzma2_header_uncompressed(coder: *mut lzma_lzma2_coder) {
    if (*coder).need_dictionary_reset {
        (*coder).buf[0] = 1;
    } else {
        (*coder).buf[0] = 2;
    }
    (*coder).need_dictionary_reset = false;
    (*coder).buf[1] = ((*coder).uncompressed_size - 1 >> 8) as u8;
    (*coder).buf[2] = ((*coder).uncompressed_size - 1 & 0xff) as u8;
    (*coder).buf_pos = 0;
}
unsafe fn lzma2_encode(
    coder_ptr: *mut c_void,
    mf: *mut lzma_mf,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    let coder: *mut lzma_lzma2_coder = coder_ptr as *mut lzma_lzma2_coder;
    while *out_pos < out_size {
        match (*coder).sequence {
            0 => {
                if mf_unencoded(mf) == 0 {
                    if (*mf).action == LZMA_FINISH {
                        *out.add(*out_pos) = 0;
                        *out_pos += 1;
                    }
                    return if (*mf).action == LZMA_RUN {
                        LZMA_OK
                    } else {
                        LZMA_STREAM_END
                    };
                }
                if (*coder).need_state_reset {
                    let ret_: lzma_ret = lzma_lzma_encoder_reset(
                        (*coder).lzma as *mut lzma_lzma1_encoder,
                        ::core::ptr::addr_of_mut!((*coder).opt_cur),
                    );
                    if ret_ != LZMA_OK {
                        return ret_;
                    }
                }
                (*coder).uncompressed_size = 0;
                (*coder).compressed_size = 0;
                (*coder).sequence = SEQ_LZMA_ENCODE;
            }
            1 | 2 => {}
            3 => {
                lzma_bufcpy(
                    ::core::ptr::addr_of_mut!((*coder).buf) as *mut u8,
                    ::core::ptr::addr_of_mut!((*coder).buf_pos),
                    LZMA2_HEADER_UNCOMPRESSED as size_t,
                    out,
                    out_pos,
                    out_size,
                );
                if (*coder).buf_pos != LZMA2_HEADER_UNCOMPRESSED as size_t {
                    return LZMA_OK;
                }
                (*coder).sequence = SEQ_UNCOMPRESSED_COPY;
            }
            4 | 5 => {}
            _ => return LZMA_PROG_ERROR,
        }
        if (*coder).sequence == SEQ_UNCOMPRESSED_COPY {
            mf_read(
                mf,
                out,
                out_pos,
                out_size,
                ::core::ptr::addr_of_mut!((*coder).uncompressed_size),
            );
            if (*coder).uncompressed_size != 0 {
                return LZMA_OK;
            }
            (*coder).sequence = SEQ_INIT;
            continue;
        }
        if (*coder).sequence == SEQ_LZMA_ENCODE {
            let left: u32 =
                ((LZMA2_UNCOMPRESSED_MAX as size_t) - (*coder).uncompressed_size) as u32;
            let limit: u32 = if left < (*mf).match_len_max {
                0
            } else {
                (*mf).read_pos - (*mf).read_ahead + left - (*mf).match_len_max
            };
            let read_start: u32 = (*mf).read_pos - (*mf).read_ahead;
            let ret: lzma_ret = lzma_lzma_encode(
                (*coder).lzma as *mut lzma_lzma1_encoder,
                mf,
                (::core::ptr::addr_of_mut!((*coder).buf) as *mut u8)
                    .offset(LZMA2_HEADER_MAX as isize),
                ::core::ptr::addr_of_mut!((*coder).compressed_size),
                LZMA2_CHUNK_MAX as size_t,
                limit,
            );
            (*coder).uncompressed_size +=
                ((*mf).read_pos - (*mf).read_ahead - read_start) as size_t;
            if ret != LZMA_STREAM_END {
                return LZMA_OK;
            }
            if (*coder).compressed_size >= (*coder).uncompressed_size {
                (*coder).uncompressed_size += (*mf).read_ahead as size_t;
                (*mf).read_ahead = 0;
                lzma2_header_uncompressed(coder);
                (*coder).need_state_reset = true;
                (*coder).sequence = SEQ_UNCOMPRESSED_HEADER;
                continue;
            }
            lzma2_header_lzma(coder);
            (*coder).sequence = SEQ_LZMA_COPY;
        }
        if (*coder).sequence == SEQ_LZMA_COPY {
            lzma_bufcpy(
                ::core::ptr::addr_of_mut!((*coder).buf) as *mut u8,
                ::core::ptr::addr_of_mut!((*coder).buf_pos),
                (*coder).compressed_size,
                out,
                out_pos,
                out_size,
            );
            if (*coder).buf_pos != (*coder).compressed_size {
                return LZMA_OK;
            }
            (*coder).sequence = SEQ_INIT;
        }
    }
    LZMA_OK
}
unsafe fn lzma2_encoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_lzma2_coder = coder_ptr as *mut lzma_lzma2_coder;
    crate::alloc::internal_free((*coder).lzma as *mut lzma_lzma1_encoder, allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn lzma2_encoder_options_update(
    coder_ptr: *mut c_void,
    filter: *const lzma_filter,
) -> lzma_ret {
    let coder: *mut lzma_lzma2_coder = coder_ptr as *mut lzma_lzma2_coder;
    if (*filter).options.is_null() || (*coder).sequence != SEQ_INIT {
        return LZMA_PROG_ERROR;
    }
    let opt: *const lzma_options_lzma = (*filter).options as *const lzma_options_lzma;
    if (*coder).opt_cur.lc != (*opt).lc
        || (*coder).opt_cur.lp != (*opt).lp
        || (*coder).opt_cur.pb != (*opt).pb
    {
        if (*opt).lc > LZMA_LCLP_MAX
            || (*opt).lp > LZMA_LCLP_MAX
            || (*opt).lc + (*opt).lp > LZMA_LCLP_MAX
            || (*opt).pb > LZMA_PB_MAX
        {
            return LZMA_OPTIONS_ERROR;
        }
        (*coder).opt_cur.lc = (*opt).lc;
        (*coder).opt_cur.lp = (*opt).lp;
        (*coder).opt_cur.pb = (*opt).pb;
        (*coder).need_properties = true;
        (*coder).need_state_reset = true;
    }
    LZMA_OK
}
unsafe fn lzma2_encoder_init(
    lz: *mut lzma_lz_encoder,
    allocator: *const lzma_allocator,
    _id: lzma_vli,
    options: *const c_void,
    lz_options: *mut lzma_lz_options,
) -> lzma_ret {
    if options.is_null() {
        return LZMA_PROG_ERROR;
    }
    let mut coder: *mut lzma_lzma2_coder = (*lz).coder as *mut lzma_lzma2_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_lzma2_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*lz).coder = coder as *mut c_void;
        (*lz).code = lzma2_encode as lzma_lz_encoder_code_function;
        (*lz).end = Some(lzma2_encoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*lz).options_update = Some(
            lzma2_encoder_options_update as unsafe fn(*mut c_void, *const lzma_filter) -> lzma_ret,
        );
        (*coder).lzma = core::ptr::null_mut();
    }
    (*coder).opt_cur = *(options as *const lzma_options_lzma);
    (*coder).sequence = SEQ_INIT;
    (*coder).need_properties = true;
    (*coder).need_state_reset = false;
    (*coder).need_dictionary_reset =
        (*coder).opt_cur.preset_dict.is_null() || (*coder).opt_cur.preset_dict_size == 0;
    let ret_: lzma_ret = lzma_lzma_encoder_create(
        ::core::ptr::addr_of_mut!((*coder).lzma),
        allocator,
        0x21,
        ::core::ptr::addr_of_mut!((*coder).opt_cur),
        lz_options,
    );
    if ret_ != LZMA_OK {
        return ret_;
    }
    if (*lz_options).before_size + (*lz_options).dict_size < LZMA2_CHUNK_MAX as size_t {
        (*lz_options).before_size = (LZMA2_CHUNK_MAX as size_t) - (*lz_options).dict_size;
    }
    LZMA_OK
}
pub(crate) unsafe fn lzma_lzma2_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
) -> lzma_ret {
    lzma_lz_encoder_init(
        next,
        allocator,
        filters,
        lzma2_encoder_init
            as unsafe fn(
                *mut lzma_lz_encoder,
                *const lzma_allocator,
                lzma_vli,
                *const c_void,
                *mut lzma_lz_options,
            ) -> lzma_ret,
    )
}
pub(crate) unsafe fn lzma_lzma2_encoder_memusage(options: *const c_void) -> u64 {
    let lzma_mem: u64 = lzma_lzma_encoder_memusage(options) as u64;
    if lzma_mem == UINT64_MAX {
        return UINT64_MAX;
    }
    (core::mem::size_of::<lzma_lzma2_coder>() as u64) + lzma_mem
}
pub(crate) unsafe fn lzma_lzma2_props_encode(options: *const c_void, out: *mut u8) -> lzma_ret {
    if options.is_null() {
        return LZMA_PROG_ERROR;
    }
    let opt: *const lzma_options_lzma = options as *const lzma_options_lzma;
    let mut d: u32 = if (*opt).dict_size > 4096 {
        (*opt).dict_size
    } else {
        4096
    };
    d -= 1;
    d |= d >> 2;
    d |= d >> 3;
    d |= d >> 4;
    d |= d >> 8;
    d |= d >> 16;
    if d == UINT32_MAX {
        *out = 40;
    } else {
        *out = (get_dist_slot(d + 1) - 24) as u8;
    }
    LZMA_OK
}
pub(crate) unsafe fn lzma_lzma2_block_size(options: *const c_void) -> u64 {
    let opt: *const lzma_options_lzma = options as *const lzma_options_lzma;
    if (*opt).dict_size < LZMA_DICT_SIZE_MIN as u32 || (*opt).dict_size > (1u32 << 30) + (1 << 29) {
        return UINT64_MAX;
    }
    if ((*opt).dict_size as u64) * 3 > 1 << 20 {
        ((*opt).dict_size as u64) * 3
    } else {
        1 << 20
    }
}
