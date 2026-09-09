use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_microlzma_coder {
    pub lzma: lzma_next_coder,
    pub comp_size: u64,
    pub uncomp_size: lzma_vli,
    pub dict_size: u32,
    pub uncomp_size_is_exact: bool,
    pub props_decoded: bool,
}
unsafe fn microlzma_decode(
    coder_ptr: *mut c_void,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    mut in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    mut out_size: size_t,
    action: lzma_action,
) -> lzma_ret {
    let coder: *mut lzma_microlzma_coder = coder_ptr as *mut lzma_microlzma_coder;
    let in_start: size_t = *in_pos;
    let out_start: size_t = *out_pos;
    if (in_size - *in_pos) as u64 > (*coder).comp_size {
        in_size = *in_pos + (*coder).comp_size as size_t;
    }
    if !(*coder).uncomp_size_is_exact && (out_size - *out_pos) as lzma_vli > (*coder).uncomp_size {
        out_size = *out_pos + (*coder).uncomp_size as size_t;
    }
    if !(*coder).props_decoded {
        if *in_pos >= in_size {
            return LZMA_OK;
        }
        let mut options: lzma_options_lzma = lzma_options_lzma {
            dict_size: (*coder).dict_size,
            preset_dict: core::ptr::null(),
            preset_dict_size: 0,
            lc: 0,
            lp: 0,
            pb: 0,
            mode: 0,
            nice_len: 0,
            mf: 0,
            depth: 0,
            ext_flags: 0,
            ext_size_low: UINT32_MAX,
            ext_size_high: UINT32_MAX,
            reserved_int4: 0,
            reserved_int5: 0,
            reserved_int6: 0,
            reserved_int7: 0,
            reserved_int8: 0,
            reserved_enum1: LZMA_RESERVED_ENUM,
            reserved_enum2: LZMA_RESERVED_ENUM,
            reserved_enum3: LZMA_RESERVED_ENUM,
            reserved_enum4: LZMA_RESERVED_ENUM,
            reserved_ptr1: core::ptr::null_mut(),
            reserved_ptr2: core::ptr::null_mut(),
        };
        if (*coder).uncomp_size_is_exact {
            options.ext_size_low = (*coder).uncomp_size as u32;
            options.ext_size_high = ((*coder).uncomp_size >> 32) as u32;
        }
        if lzma_lzma_lclppb_decode(::core::ptr::addr_of_mut!(options), !*input.add(*in_pos)) {
            return LZMA_OPTIONS_ERROR;
        }
        *in_pos += 1;
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
                options: ::core::ptr::addr_of_mut!(options) as *mut c_void,
            },
            lzma_filter_info_s {
                id: 0,
                init: None,
                options: core::ptr::null_mut(),
            },
        ];
        let ret: lzma_ret = lzma_next_filter_init(
            ::core::ptr::addr_of_mut!((*coder).lzma),
            allocator,
            ::core::ptr::addr_of_mut!(filters) as *mut lzma_filter_info,
        );
        if ret != LZMA_OK {
            return ret;
        }
        let dummy_in: u8 = 0;
        let mut dummy_in_pos: size_t = 0;
        let Some(code) = (*coder).lzma.code else {
            return LZMA_PROG_ERROR;
        };
        if code(
            (*coder).lzma.coder,
            allocator,
            ::core::ptr::addr_of!(dummy_in),
            ::core::ptr::addr_of_mut!(dummy_in_pos),
            1,
            out,
            out_pos,
            out_size,
            LZMA_RUN,
        ) != LZMA_OK
        {
            return LZMA_PROG_ERROR;
        }
        (*coder).props_decoded = true;
    }
    let Some(code) = (*coder).lzma.code else {
        return LZMA_PROG_ERROR;
    };
    let mut ret: lzma_ret = code(
        (*coder).lzma.coder,
        allocator,
        input,
        in_pos,
        in_size,
        out,
        out_pos,
        out_size,
        action,
    );
    (*coder).comp_size -= (*in_pos - in_start) as u64;
    if (*coder).uncomp_size_is_exact {
        if ret == LZMA_STREAM_END && (*coder).comp_size != 0 {
            ret = LZMA_DATA_ERROR;
        }
    } else {
        (*coder).uncomp_size -= (*out_pos - out_start) as lzma_vli;
        if ret == LZMA_STREAM_END {
            ret = LZMA_DATA_ERROR;
        } else if (*coder).uncomp_size == 0 {
            ret = LZMA_STREAM_END;
        }
    }
    ret
}
unsafe fn microlzma_decoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_microlzma_coder = coder_ptr as *mut lzma_microlzma_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).lzma), allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn microlzma_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    comp_size: u64,
    uncomp_size: u64,
    uncomp_size_is_exact: bool,
    dict_size: u32,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<
            unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u64, bool, u32) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        microlzma_decoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                u64,
                u64,
                bool,
                u32,
            ) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<
            unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u64, bool, u32) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        microlzma_decoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                u64,
                u64,
                bool,
                u32,
            ) -> lzma_ret,
    ));
    let mut coder: *mut lzma_microlzma_coder = (*next).coder as *mut lzma_microlzma_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_microlzma_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            microlzma_decode
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
            Some(microlzma_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*coder).lzma = lzma_next_coder_s {
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
    if uncomp_size > LZMA_VLI_MAX as u64 {
        return LZMA_OPTIONS_ERROR;
    }
    (*coder).comp_size = comp_size;
    (*coder).uncomp_size = uncomp_size as lzma_vli;
    (*coder).uncomp_size_is_exact = uncomp_size_is_exact;
    (*coder).dict_size = dict_size;
    (*coder).props_decoded = false;
    LZMA_OK
}
pub unsafe fn lzma_microlzma_decoder(
    strm: *mut lzma_stream,
    comp_size: u64,
    uncomp_size: u64,
    uncomp_size_is_exact: lzma_bool,
    dict_size: u32,
) -> lzma_ret {
    let ret: lzma_ret = lzma_strm_init(strm);
    if ret != LZMA_OK {
        return ret;
    }
    let ret: lzma_ret = microlzma_decoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        comp_size,
        uncomp_size,
        uncomp_size_is_exact != 0,
        dict_size,
    );
    if ret != LZMA_OK {
        lzma_end(strm);
        return ret;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
