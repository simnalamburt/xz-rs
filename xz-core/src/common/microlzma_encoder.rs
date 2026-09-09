use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_microlzma_coder {
    pub lzma: lzma_next_coder,
    pub props: u8,
}
unsafe fn microlzma_encode(
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
    let coder: *mut lzma_microlzma_coder = coder_ptr as *mut lzma_microlzma_coder;
    let out_start: size_t = *out_pos;
    let in_start: size_t = *in_pos;
    let mut uncomp_size: u64 = 0;
    debug_assert!((*coder).lzma.set_out_limit.is_some());
    let set_out_limit = (*coder).lzma.set_out_limit.unwrap_unchecked();
    debug_assert!((*coder).lzma.code.is_some());
    let code = (*coder).lzma.code.unwrap_unchecked();
    if set_out_limit(
        (*coder).lzma.coder,
        ::core::ptr::addr_of_mut!(uncomp_size),
        (out_size - *out_pos) as u64,
    ) != LZMA_OK
    {
        return LZMA_PROG_ERROR;
    }
    let ret: lzma_ret = code(
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
    if ret != LZMA_STREAM_END {
        if ret == LZMA_OK {
            return LZMA_PROG_ERROR;
        }
        return ret;
    }
    *out.add(out_start) = !(*coder).props;
    *in_pos = in_start + uncomp_size as size_t;
    ret
}
unsafe fn microlzma_encoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_microlzma_coder = coder_ptr as *mut lzma_microlzma_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).lzma), allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn microlzma_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    options: *const lzma_options_lzma,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<
            unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *const lzma_options_lzma,
            ) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        microlzma_encoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *const lzma_options_lzma,
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
                *const lzma_options_lzma,
            ) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        microlzma_encoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                *const lzma_options_lzma,
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
            microlzma_encode
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
            Some(microlzma_encoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
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
    if lzma_lzma_lclppb_encode(options, ::core::ptr::addr_of_mut!((*coder).props)) {
        return LZMA_OPTIONS_ERROR;
    }
    let filters: [lzma_filter_info; 2] = [
        lzma_filter_info_s {
            id: LZMA_FILTER_LZMA1,
            init: Some(
                lzma_lzma_encoder_init
                    as unsafe fn(
                        *mut lzma_next_coder,
                        *const lzma_allocator,
                        *const lzma_filter_info,
                    ) -> lzma_ret,
            ),
            options: options as *mut c_void,
        },
        lzma_filter_info_s {
            id: 0,
            init: None,
            options: core::ptr::null_mut(),
        },
    ];
    lzma_next_filter_init(
        ::core::ptr::addr_of_mut!((*coder).lzma),
        allocator,
        ::core::ptr::addr_of!(filters) as *const lzma_filter_info,
    )
}
pub unsafe fn lzma_microlzma_encoder(
    strm: *mut lzma_stream,
    options: *const lzma_options_lzma,
) -> lzma_ret {
    let ret: lzma_ret = lzma_strm_init(strm);
    if ret != LZMA_OK {
        return ret;
    }
    let ret: lzma_ret = microlzma_encoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        options,
    );
    if ret != LZMA_OK {
        lzma_end(strm);
        return ret;
    }
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
