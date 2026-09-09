use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_lzip_coder {
    pub sequence: lzip_decoder_seq,
    pub version: u32,
    pub crc32: u32,
    pub uncompressed_size: u64,
    pub member_size: u64,
    pub memlimit: u64,
    pub memusage: u64,
    pub tell_any_check: bool,
    pub ignore_check: bool,
    pub concatenated: bool,
    pub first_member: bool,
    pub pos: size_t,
    pub buffer: [u8; 20],
    pub options: lzma_options_lzma,
    pub lzma_decoder: lzma_next_coder,
}
pub type lzip_decoder_seq = c_uint;
pub const SEQ_MEMBER_FOOTER: lzip_decoder_seq = 5;
pub const SEQ_LZMA_STREAM: lzip_decoder_seq = 4;
pub const SEQ_CODER_INIT: lzip_decoder_seq = 3;
pub const SEQ_DICT_SIZE: lzip_decoder_seq = 2;
pub const SEQ_VERSION: lzip_decoder_seq = 1;
pub const SEQ_ID_STRING: lzip_decoder_seq = 0;
#[inline]
fn read64le(buf: &[u8; 8]) -> u64 {
    let mut num: u64 = buf[0] as u64;
    num |= (buf[1] as u64) << 8;
    num |= (buf[2] as u64) << 16;
    num |= (buf[3] as u64) << 24;
    num |= (buf[4] as u64) << 32;
    num |= (buf[5] as u64) << 40;
    num |= (buf[6] as u64) << 48;
    num |= (buf[7] as u64) << 56;
    num
}
pub const LZIP_V0_FOOTER_SIZE: u32 = 12;
pub const LZIP_V1_FOOTER_SIZE: u32 = 20;
pub const LZIP_LC: u32 = 3;
pub const LZIP_LP: u32 = 0;
pub const LZIP_PB: u32 = 2;
type LzipBlockState = u8;
const LZIP_BLOCK_VERSION: LzipBlockState = 0;
const LZIP_BLOCK_DICT_SIZE: LzipBlockState = 1;
const LZIP_BLOCK_CODER_INIT: LzipBlockState = 2;
const LZIP_BLOCK_LZMA_STREAM: LzipBlockState = 3;
const LZIP_BLOCK_MEMBER_FOOTER: LzipBlockState = 4;
unsafe fn lzip_decode(
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
    let coder: *mut lzma_lzip_coder = coder_ptr as *mut lzma_lzip_coder;
    loop {
        let mut block_state: LzipBlockState;
        match (*coder).sequence {
            0 => {
                let lzip_id_string: [u8; 4] = [0x4c as u8, 0x5a as u8, 0x49 as u8, 0x50 as u8];
                while (*coder).pos < core::mem::size_of::<[u8; 4]>() {
                    if *in_pos >= in_size {
                        return if !(*coder).first_member && action == LZMA_FINISH {
                            LZMA_STREAM_END
                        } else {
                            LZMA_OK
                        };
                    }
                    if *input.add(*in_pos) != lzip_id_string[(*coder).pos as usize] {
                        return if !(*coder).first_member {
                            LZMA_STREAM_END
                        } else {
                            LZMA_FORMAT_ERROR
                        };
                    }
                    *in_pos += 1;
                    (*coder).pos += 1;
                }
                (*coder).pos = 0;
                (*coder).crc32 = 0;
                (*coder).uncompressed_size = 0;
                (*coder).member_size = core::mem::size_of::<[u8; 4]>() as u64;
                (*coder).sequence = SEQ_VERSION;
                block_state = LZIP_BLOCK_VERSION;
            }
            1 => {
                block_state = LZIP_BLOCK_VERSION;
            }
            2 => {
                block_state = LZIP_BLOCK_DICT_SIZE;
            }
            3 => {
                block_state = LZIP_BLOCK_CODER_INIT;
            }
            4 => {
                block_state = LZIP_BLOCK_LZMA_STREAM;
            }
            5 => {
                block_state = LZIP_BLOCK_MEMBER_FOOTER;
            }
            _ => return LZMA_PROG_ERROR,
        }
        match block_state {
            LZIP_BLOCK_VERSION => {
                if *in_pos >= in_size {
                    return LZMA_OK;
                }
                (*coder).version = *input.add(*in_pos) as u32;
                *in_pos += 1;
                if (*coder).version > 1 {
                    return LZMA_OPTIONS_ERROR;
                }
                (*coder).member_size += 1;
                (*coder).sequence = SEQ_DICT_SIZE;
                if (*coder).tell_any_check {
                    return LZMA_GET_CHECK;
                }
                block_state = LZIP_BLOCK_DICT_SIZE;
            }
            _ => {}
        }
        match block_state {
            LZIP_BLOCK_DICT_SIZE => {
                if *in_pos >= in_size {
                    return LZMA_OK;
                }
                let ds: u32 = *input.add(*in_pos) as u32;
                *in_pos += 1;
                (*coder).member_size += 1;
                let b2log: u32 = ds & 0x1f;
                let fracnum: u32 = ds >> 5;
                if b2log < 12 || b2log > 29 || b2log == 12 && fracnum > 0 {
                    return LZMA_DATA_ERROR;
                }
                (*coder).options.dict_size = (1u32 << b2log) - (fracnum << (b2log - 4));
                (*coder).options.preset_dict = core::ptr::null();
                (*coder).options.lc = LZIP_LC;
                (*coder).options.lp = LZIP_LP;
                (*coder).options.pb = LZIP_PB;
                (*coder).memusage = lzma_lzma_decoder_memusage_nocheck(::core::ptr::addr_of_mut!(
                    (*coder).options
                )
                    as *const c_void)
                .wrapping_add(LZMA_MEMUSAGE_BASE);
                (*coder).sequence = SEQ_CODER_INIT;
                block_state = LZIP_BLOCK_CODER_INIT;
            }
            _ => {}
        }
        match block_state {
            LZIP_BLOCK_CODER_INIT => {
                if (*coder).memusage > (*coder).memlimit {
                    return LZMA_MEMLIMIT_ERROR;
                }
                let filters: [lzma_filter_info; 2] = [
                    lzma_filter_info_s {
                        id: LZMA_FILTER_LZMA1,
                        init: Some(
                            lzma_lzma_decoder_init
                                as unsafe fn(
                                    *mut lzma_next_coder,
                                    *const lzma_allocator,
                                    *const lzma_filter_info,
                                ) -> lzma_ret,
                        ),
                        options: ::core::ptr::addr_of_mut!((*coder).options) as *mut c_void,
                    },
                    lzma_filter_info_s {
                        id: 0,
                        init: None,
                        options: core::ptr::null_mut(),
                    },
                ];
                let ret_: lzma_ret = lzma_next_filter_init(
                    ::core::ptr::addr_of_mut!((*coder).lzma_decoder),
                    allocator,
                    ::core::ptr::addr_of!(filters) as *const lzma_filter_info,
                );
                if ret_ != LZMA_OK {
                    return ret_;
                }
                (*coder).crc32 = 0;
                (*coder).sequence = SEQ_LZMA_STREAM;
                block_state = LZIP_BLOCK_LZMA_STREAM;
            }
            _ => {}
        }
        match block_state {
            LZIP_BLOCK_LZMA_STREAM => {
                let in_start: size_t = *in_pos;
                let out_start: size_t = *out_pos;
                let Some(code) = (*coder).lzma_decoder.code else {
                    return LZMA_PROG_ERROR;
                };
                let ret: lzma_ret = code(
                    (*coder).lzma_decoder.coder,
                    allocator,
                    input,
                    in_pos,
                    in_size,
                    out,
                    out_pos,
                    out_size,
                    action,
                );
                let out_used: size_t = *out_pos - out_start;
                (*coder).member_size += (*in_pos - in_start) as u64;
                (*coder).uncompressed_size += out_used as u64;
                if !(*coder).ignore_check && out_used > 0 {
                    (*coder).crc32 = lzma_crc32(out.add(out_start), out_used, (*coder).crc32);
                }
                if ret != LZMA_STREAM_END {
                    return ret;
                }
                (*coder).sequence = SEQ_MEMBER_FOOTER;
            }
            _ => {}
        }
        let footer_size: size_t = (if (*coder).version == 0 {
            LZIP_V0_FOOTER_SIZE
        } else {
            LZIP_V1_FOOTER_SIZE
        }) as size_t;
        lzma_bufcpy(
            input,
            in_pos,
            in_size,
            ::core::ptr::addr_of_mut!((*coder).buffer) as *mut u8,
            ::core::ptr::addr_of_mut!((*coder).pos),
            footer_size,
        );
        if (*coder).pos < footer_size {
            return LZMA_OK;
        }
        (*coder).pos = 0;
        (*coder).member_size += footer_size as u64;
        let footer = ::core::ptr::addr_of!((*coder).buffer) as *const u8;
        if !(*coder).ignore_check && (*coder).crc32 != read32le(&*footer.cast::<[u8; 4]>()) {
            return LZMA_DATA_ERROR;
        }
        if (*coder).uncompressed_size != read64le(&*footer.add(4).cast::<[u8; 8]>()) {
            return LZMA_DATA_ERROR;
        }
        if (*coder).version > 0 {
            if (*coder).member_size != read64le(&*footer.add(12).cast::<[u8; 8]>()) {
                return LZMA_DATA_ERROR;
            }
        }
        if !(*coder).concatenated {
            return LZMA_STREAM_END;
        }
        (*coder).first_member = false;
        (*coder).sequence = SEQ_ID_STRING;
    }
}
unsafe fn lzip_decoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_lzip_coder = coder_ptr as *mut lzma_lzip_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).lzma_decoder), allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn lzip_decoder_get_check(_coder_ptr: *const c_void) -> lzma_check {
    LZMA_CHECK_CRC32
}
unsafe fn lzip_decoder_memconfig(
    coder_ptr: *mut c_void,
    memusage: *mut u64,
    old_memlimit: *mut u64,
    new_memlimit: u64,
) -> lzma_ret {
    let coder: *mut lzma_lzip_coder = coder_ptr as *mut lzma_lzip_coder;
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
pub(crate) unsafe fn lzma_lzip_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    memlimit: u64,
    flags: u32,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret>,
        uintptr_t,
    >(Some(
        lzma_lzip_decoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret>,
        uintptr_t,
    >(Some(
        lzma_lzip_decoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret,
    ));
    if flags & !(LZMA_SUPPORTED_FLAGS as u32) != 0 {
        return LZMA_OPTIONS_ERROR;
    }
    let mut coder: *mut lzma_lzip_coder = (*next).coder as *mut lzma_lzip_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_lzip_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            lzip_decode
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
        (*next).end = Some(lzip_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*next).get_check = Some(lzip_decoder_get_check as unsafe fn(*const c_void) -> lzma_check);
        (*next).memconfig = Some(
            lzip_decoder_memconfig as unsafe fn(*mut c_void, *mut u64, *mut u64, u64) -> lzma_ret,
        );
        (*coder).lzma_decoder = lzma_next_coder_s {
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
    (*coder).sequence = SEQ_ID_STRING;
    (*coder).memlimit = if 1 > memlimit { 1 } else { memlimit };
    (*coder).memusage = LZMA_MEMUSAGE_BASE;
    (*coder).tell_any_check = flags & LZMA_TELL_ANY_CHECK as u32 != 0;
    (*coder).ignore_check = flags & LZMA_IGNORE_CHECK as u32 != 0;
    (*coder).concatenated = flags & LZMA_CONCATENATED as u32 != 0;
    (*coder).first_member = true;
    (*coder).pos = 0;
    LZMA_OK
}
pub unsafe fn lzma_lzip_decoder(strm: *mut lzma_stream, memlimit: u64, flags: u32) -> lzma_ret {
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = lzma_lzip_decoder_init(
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
