use crate::types::*;
pub unsafe fn lzma_block_buffer_decode(
    block: &mut lzma_block,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    if in_pos.is_null()
        || input.is_null() && *in_pos != in_size
        || *in_pos > in_size
        || out_pos.is_null()
        || out.is_null() && *out_pos != out_size
        || *out_pos > out_size
    {
        return LZMA_PROG_ERROR;
    }
    let mut block_decoder: lzma_next_coder = LZMA_NEXT_CODER_INIT;
    let mut ret: lzma_ret =
        lzma_block_decoder_init(::core::ptr::addr_of_mut!(block_decoder), allocator, block);
    if ret == LZMA_OK {
        let in_start: size_t = *in_pos;
        let out_start: size_t = *out_pos;
        ret = block_decoder.code(
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
                if *in_pos == in_size {
                    ret = LZMA_DATA_ERROR;
                } else {
                    ret = LZMA_BUF_ERROR;
                }
            }
            *in_pos = in_start;
            *out_pos = out_start;
        }
    }
    lzma_next_end(::core::ptr::addr_of_mut!(block_decoder), allocator);
    ret
}
