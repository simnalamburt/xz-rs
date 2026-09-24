use crate::types::*;
pub unsafe fn lzma_raw_buffer_encode(
    filters: *const lzma_filter,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    if input.is_null() && in_size != 0 || out.is_null() || out_pos.is_null() || *out_pos > out_size
    {
        return LZMA_PROG_ERROR;
    }
    let mut next: lzma_next_coder = LZMA_NEXT_CODER_INIT;
    let ret: lzma_ret = lzma_raw_encoder_init(::core::ptr::addr_of_mut!(next), allocator, filters);
    if ret != LZMA_OK {
        return ret;
    }
    let out_start: size_t = *out_pos;
    let mut in_pos: size_t = 0;
    let mut ret: lzma_ret = next.code(
        allocator,
        input,
        ::core::ptr::addr_of_mut!(in_pos),
        in_size,
        out,
        out_pos,
        out_size,
        LZMA_FINISH,
    );
    lzma_next_end(::core::ptr::addr_of_mut!(next), allocator);
    if ret == LZMA_STREAM_END {
        ret = LZMA_OK;
    } else {
        if ret == LZMA_OK {
            ret = LZMA_BUF_ERROR;
        }
        *out_pos = out_start;
    }
    ret
}
