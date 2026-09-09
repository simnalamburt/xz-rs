use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_alone_coder {
    pub next: lzma_next_coder,
    pub sequence: alone_decoder_seq,
    pub picky: bool,
    pub pos: size_t,
    pub uncompressed_size: lzma_vli,
    pub memlimit: u64,
    pub memusage: u64,
    pub options: lzma_options_lzma,
}
pub type alone_decoder_seq = c_uint;
pub const SEQ_CODE: alone_decoder_seq = 4;
pub const SEQ_CODER_INIT: alone_decoder_seq = 3;
pub const SEQ_UNCOMPRESSED_SIZE: alone_decoder_seq = 2;
pub const SEQ_DICTIONARY_SIZE: alone_decoder_seq = 1;
pub const SEQ_PROPERTIES: alone_decoder_seq = 0;
unsafe fn alone_decode(
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
    let coder: *mut lzma_alone_coder = coder_ptr as *mut lzma_alone_coder;
    while *out_pos < out_size && ((*coder).sequence == SEQ_CODE || *in_pos < in_size) {
        match (*coder).sequence {
            0 => {
                if lzma_lzma_lclppb_decode(
                    ::core::ptr::addr_of_mut!((*coder).options),
                    *input.add(*in_pos),
                ) {
                    return LZMA_FORMAT_ERROR;
                }
                (*coder).sequence = SEQ_DICTIONARY_SIZE;
                *in_pos += 1;
            }
            1 => {
                (*coder).options.dict_size = ((*coder).options.dict_size as size_t
                    | (*input.add(*in_pos) as size_t) << ((*coder).pos * 8))
                    as u32;
                (*coder).pos += 1;
                if (*coder).pos == 4 {
                    if (*coder).picky && (*coder).options.dict_size != UINT32_MAX {
                        let mut d: u32 = (*coder).options.dict_size.wrapping_sub(1);
                        d |= d >> 2;
                        d |= d >> 3;
                        d |= d >> 4;
                        d |= d >> 8;
                        d |= d >> 16;
                        d = d.wrapping_add(1);
                        if d != (*coder).options.dict_size {
                            return LZMA_FORMAT_ERROR;
                        }
                    }
                    (*coder).pos = 0;
                    (*coder).sequence = SEQ_UNCOMPRESSED_SIZE;
                }
                *in_pos += 1;
            }
            2 => {
                (*coder).uncompressed_size |=
                    (*input.add(*in_pos) as lzma_vli) << ((*coder).pos * 8);
                *in_pos += 1;
                (*coder).pos += 1;
                if (*coder).pos >= 8 {
                    if (*coder).picky
                        && (*coder).uncompressed_size != LZMA_VLI_UNKNOWN
                        && (*coder).uncompressed_size >= 1 << 38
                    {
                        return LZMA_FORMAT_ERROR;
                    }
                    (*coder).options.ext_flags = LZMA_LZMA1EXT_ALLOW_EOPM as u32;
                    (*coder).options.ext_size_low = (*coder).uncompressed_size as u32;
                    (*coder).options.ext_size_high = ((*coder).uncompressed_size >> 32) as u32;
                    (*coder).memusage = lzma_lzma_decoder_memusage_nocheck(
                        ::core::ptr::addr_of_mut!((*coder).options) as *const c_void,
                    )
                    .wrapping_add(LZMA_MEMUSAGE_BASE);
                    (*coder).pos = 0;
                    (*coder).sequence = SEQ_CODER_INIT;
                }
            }
            3 => {}
            4 => {
                debug_assert!((*coder).next.code.is_some());
                let code = (*coder).next.code.unwrap_unchecked();
                return code(
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
            }
            _ => return LZMA_PROG_ERROR,
        }
        if (*coder).sequence == SEQ_CODER_INIT {
            if (*coder).memusage > (*coder).memlimit {
                return LZMA_MEMLIMIT_ERROR;
            }
            let mut filters: [lzma_filter_info; 2] = [
                lzma_filter_info_s {
                    id: LZMA_FILTER_LZMA1EXT,
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
                ::core::ptr::addr_of_mut!((*coder).next),
                allocator,
                ::core::ptr::addr_of_mut!(filters) as *mut lzma_filter_info,
            );
            if ret_ != LZMA_OK {
                return ret_;
            }
            (*coder).sequence = SEQ_CODE;
        }
    }
    LZMA_OK
}
unsafe fn alone_decoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_alone_coder = coder_ptr as *mut lzma_alone_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).next), allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn alone_decoder_memconfig(
    coder_ptr: *mut c_void,
    memusage: *mut u64,
    old_memlimit: *mut u64,
    new_memlimit: u64,
) -> lzma_ret {
    let coder: *mut lzma_alone_coder = coder_ptr as *mut lzma_alone_coder;
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
pub(crate) unsafe fn lzma_alone_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    memlimit: u64,
    picky: bool,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, bool) -> lzma_ret>,
        uintptr_t,
    >(Some(
        lzma_alone_decoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, bool) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, bool) -> lzma_ret>,
        uintptr_t,
    >(Some(
        lzma_alone_decoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, bool) -> lzma_ret,
    ));
    let mut coder: *mut lzma_alone_coder = (*next).coder as *mut lzma_alone_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_alone_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            alone_decode
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
            Some(alone_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*next).memconfig = Some(
            alone_decoder_memconfig as unsafe fn(*mut c_void, *mut u64, *mut u64, u64) -> lzma_ret,
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
    (*coder).sequence = SEQ_PROPERTIES;
    (*coder).picky = picky;
    (*coder).pos = 0;
    (*coder).options.dict_size = 0;
    (*coder).options.preset_dict = core::ptr::null();
    (*coder).options.preset_dict_size = 0;
    (*coder).uncompressed_size = 0;
    (*coder).memlimit = if 1 > memlimit { 1 } else { memlimit };
    (*coder).memusage = LZMA_MEMUSAGE_BASE;
    LZMA_OK
}
pub unsafe fn lzma_alone_decoder(strm: *mut lzma_stream, memlimit: u64) -> lzma_ret {
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = lzma_alone_decoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        memlimit,
        false,
    );
    if ret__0 != LZMA_OK {
        lzma_end(strm);
        return ret__0;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
