use crate::types::*;
pub unsafe fn lzma_raw_buffer_decode(
    filters: *const lzma_filter,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    if input.is_null()
        || in_pos.is_null()
        || *in_pos > in_size
        || out.is_null()
        || out_pos.is_null()
        || *out_pos > out_size
    {
        return LZMA_PROG_ERROR;
    }
    let mut next: lzma_next_coder = LZMA_NEXT_CODER_INIT;
    let ret: lzma_ret = lzma_raw_decoder_init(::core::ptr::addr_of_mut!(next), allocator, filters);
    if ret != LZMA_OK {
        return ret;
    }
    let in_start: size_t = *in_pos;
    let out_start: size_t = *out_pos;
    let mut ret: lzma_ret = next.code(
        allocator,
        input,
        in_pos,
        in_size,
        out,
        out_pos,
        out_size,
        LZMA_FINISH,
    );
    if ret == LZMA_STREAM_END {
        ret = LZMA_OK;
    } else {
        if ret == LZMA_OK {
            if *in_pos != in_size {
                ret = LZMA_BUF_ERROR;
            } else if *out_pos != out_size {
                ret = LZMA_DATA_ERROR;
            } else {
                let mut tmp: [u8; 1] = [0; 1];
                let mut tmp_pos: size_t = 0;
                next.code(
                    allocator,
                    input,
                    in_pos,
                    in_size,
                    ::core::ptr::addr_of_mut!(tmp) as *mut u8,
                    ::core::ptr::addr_of_mut!(tmp_pos),
                    1,
                    LZMA_FINISH,
                );
                if tmp_pos == 1 {
                    ret = LZMA_BUF_ERROR;
                } else {
                    ret = LZMA_DATA_ERROR;
                }
            }
        }
        *in_pos = in_start;
        *out_pos = out_start;
    }
    lzma_next_end(::core::ptr::addr_of_mut!(next), allocator);
    ret
}
