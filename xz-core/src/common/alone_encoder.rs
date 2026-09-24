use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_alone_coder {
    pub next: lzma_next_coder,
    pub sequence: alone_encoder_seq,
    pub header_pos: size_t,
    pub header: [u8; 13],
}
pub type alone_encoder_seq = c_uint;
pub const SEQ_CODE: alone_encoder_seq = 1;
pub const SEQ_HEADER: alone_encoder_seq = 0;
pub const ALONE_HEADER_SIZE: u32 = 1 + 4 + 8;
unsafe fn alone_encode(
    coder: &mut lzma_alone_coder,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    action: lzma_action,
) -> lzma_ret {
    while *out_pos < out_size {
        match coder.sequence {
            0 => {
                lzma_bufcpy(
                    ::core::ptr::addr_of_mut!(coder.header) as *mut u8,
                    ::core::ptr::addr_of_mut!(coder.header_pos),
                    ALONE_HEADER_SIZE as size_t,
                    out,
                    out_pos,
                    out_size,
                );
                if coder.header_pos < ALONE_HEADER_SIZE as size_t {
                    return LZMA_OK;
                }
                coder.sequence = SEQ_CODE;
            }
            1 => {
                return coder.next.code(
                    allocator, input, in_pos, in_size, out, out_pos, out_size, action,
                );
            }
            _ => return LZMA_PROG_ERROR,
        }
    }
    LZMA_OK
}
unsafe fn alone_encoder_end(coder: &mut lzma_alone_coder, allocator: *const lzma_allocator) {
    lzma_next_end(::core::ptr::addr_of_mut!(coder.next), allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn alone_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    options: &lzma_options_lzma,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<
            unsafe fn(*mut lzma_next_coder, *const lzma_allocator, &lzma_options_lzma) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        alone_encoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                &lzma_options_lzma,
            ) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<
            unsafe fn(*mut lzma_next_coder, *const lzma_allocator, &lzma_options_lzma) -> lzma_ret,
        >,
        uintptr_t,
    >(Some(
        alone_encoder_init
            as unsafe fn(
                *mut lzma_next_coder,
                *const lzma_allocator,
                &lzma_options_lzma,
            ) -> lzma_ret,
    ));
    let mut coder: *mut lzma_alone_coder = (*next).coder_as::<lzma_alone_coder>();
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_alone_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).set_coder(coder);
        (*coder).next = LZMA_NEXT_CODER_INIT;
    }
    (*coder).sequence = SEQ_HEADER;
    (*coder).header_pos = 0;
    if lzma_lzma_lclppb_encode(
        options,
        ::core::ptr::addr_of_mut!((*coder).header) as *mut u8,
    ) {
        return LZMA_OPTIONS_ERROR;
    }
    if options.dict_size < LZMA_DICT_SIZE_MIN as u32 {
        return LZMA_OPTIONS_ERROR;
    }
    let mut d: u32 = options.dict_size - 1;
    d |= d >> 2;
    d |= d >> 3;
    d |= d >> 4;
    d |= d >> 8;
    d |= d >> 16;
    if d != UINT32_MAX {
        d += 1;
    }
    write32le(
        &mut *((::core::ptr::addr_of_mut!((*coder).header) as *mut u8)
            .add(1)
            .cast::<[u8; 4]>()),
        d,
    );
    core::ptr::write_bytes(
        (::core::ptr::addr_of_mut!((*coder).header) as *mut u8)
            .offset(1)
            .offset(4) as *mut u8,
        0xff as u8,
        8,
    );
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
            options: options as *const lzma_options_lzma as *mut c_void,
        },
        lzma_filter_info_s {
            id: 0,
            init: None,
            options: core::ptr::null_mut(),
        },
    ];
    lzma_next_filter_init(
        ::core::ptr::addr_of_mut!((*coder).next),
        allocator,
        ::core::ptr::addr_of!(filters) as *const lzma_filter_info,
    )
}
impl NextCoder for lzma_alone_coder {
    unsafe fn code(
        &mut self,
        allocator: *const lzma_allocator,
        input: *const u8,
        in_pos: *mut size_t,
        in_size: size_t,
        out: *mut u8,
        out_pos: *mut size_t,
        out_size: size_t,
        action: lzma_action,
    ) -> lzma_ret {
        alone_encode(
            self, allocator, input, in_pos, in_size, out, out_pos, out_size, action,
        )
    }
    unsafe fn end(&mut self, allocator: *const lzma_allocator) {
        alone_encoder_end(self, allocator)
    }
}
pub unsafe fn lzma_alone_encoder(strm: &mut lzma_stream, options: &lzma_options_lzma) -> lzma_ret {
    let ret: lzma_ret = lzma_strm_init(strm);
    if ret != LZMA_OK {
        return ret;
    }
    let ret: lzma_ret = alone_encoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        options,
    );
    if ret != LZMA_OK {
        lzma_end(strm);
        return ret;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
