use crate::types::*;
/// The two delta coders share one state and differ only in `code`, so each
/// wraps the state in its own type.
pub(crate) trait DeltaCoder: NextCoder + 'static {
    fn wrap(coder: lzma_delta_coder) -> Self;
    fn delta(&mut self) -> &mut lzma_delta_coder;
}
/// Ends the chain and frees the coder. `T` is the wrapper `coder` was
/// allocated as.
pub(crate) unsafe fn delta_coder_end<T: DeltaCoder>(
    coder: &mut T,
    allocator: *const lzma_allocator,
) {
    lzma_next_end(::core::ptr::addr_of_mut!(coder.delta().next), allocator);
    crate::alloc::internal_free(coder as *mut T, allocator);
}
pub(crate) unsafe fn lzma_delta_coder_init<T: DeltaCoder>(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
) -> lzma_ret {
    let mut wrapper: *mut T = (*next).coder_as::<T>();
    if wrapper.is_null() {
        wrapper = crate::alloc::internal_alloc_object::<T>(allocator);
        if wrapper.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).set_coder(wrapper);
        wrapper.write(T::wrap(lzma_delta_coder {
            next: LZMA_NEXT_CODER_INIT,
            distance: 0,
            pos: 0,
            history: [0; LZMA_DELTA_DIST_MAX as usize],
        }));
    }
    let coder: *mut lzma_delta_coder = (*wrapper).delta();
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
