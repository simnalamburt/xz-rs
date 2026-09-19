use crate::types::*;
unsafe fn delta_coder_end(coder: &mut lzma_delta_coder, allocator: *const lzma_allocator) {
    lzma_next_end(::core::ptr::addr_of_mut!(coder.next), allocator);
    crate::alloc::internal_free(coder as *mut lzma_delta_coder, allocator);
}
pub unsafe fn lzma_delta_coder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
) -> lzma_ret {
    let mut coder: *mut lzma_delta_coder = (*next).coder as *mut lzma_delta_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_delta_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).end = coder_end_fn!(delta_coder_end, lzma_delta_coder);
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
    if lzma_delta_coder_memusage((*filters).options) == UINT64_MAX {
        return LZMA_OPTIONS_ERROR;
    }
    let opt: *const lzma_options_delta = (*filters).options as *const lzma_options_delta;
    (*coder).distance = (*opt).dist as size_t;
    (*coder).pos = 0;
    core::ptr::write_bytes(
        ::core::ptr::addr_of_mut!((*coder).history) as *mut u8,
        0 as u8,
        256,
    );
    lzma_next_filter_init(
        ::core::ptr::addr_of_mut!((*coder).next),
        allocator,
        filters.offset(1),
    )
}
pub(crate) unsafe fn lzma_delta_coder_memusage(options: *const c_void) -> u64 {
    let opt: *const lzma_options_delta = options as *const lzma_options_delta;
    if opt.is_null()
        || (*opt).type_ != LZMA_DELTA_TYPE_BYTE
        || (*opt).dist < LZMA_DELTA_DIST_MIN
        || (*opt).dist > LZMA_DELTA_DIST_MAX
    {
        return UINT64_MAX;
    }
    core::mem::size_of::<lzma_delta_coder>() as u64
}
