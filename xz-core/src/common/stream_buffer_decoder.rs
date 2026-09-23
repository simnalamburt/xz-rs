use crate::types::*;
pub unsafe fn lzma_stream_buffer_decode(
    memlimit: *mut u64,
    flags: u32,
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
    if flags & LZMA_TELL_ANY_CHECK as u32 != 0 {
        return LZMA_PROG_ERROR;
    }
    let mut stream_decoder: lzma_next_coder = LZMA_NEXT_CODER_INIT;
    let mut ret: lzma_ret = lzma_stream_decoder_init(
        ::core::ptr::addr_of_mut!(stream_decoder),
        allocator,
        *memlimit,
        flags,
    );
    if ret == LZMA_OK {
        let in_start: size_t = *in_pos;
        let out_start: size_t = *out_pos;
        ret = stream_decoder.code(
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
            *in_pos = in_start;
            *out_pos = out_start;
            if ret == LZMA_OK {
                if *in_pos == in_size {
                    ret = LZMA_DATA_ERROR;
                } else {
                    ret = LZMA_BUF_ERROR;
                }
            } else if ret == LZMA_MEMLIMIT_ERROR {
                let mut memusage: u64 = 0;
                stream_decoder.memconfig(memlimit, ::core::ptr::addr_of_mut!(memusage), 0);
            }
        }
    }
    lzma_next_end(::core::ptr::addr_of_mut!(stream_decoder), allocator);
    ret
}
