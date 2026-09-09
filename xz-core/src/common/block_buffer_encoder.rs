use crate::types::*;
pub const HEADERS_BOUND: u32 =
    1 + 1 + 2 * LZMA_VLI_BYTES_MAX + 3 + 4 + LZMA_CHECK_SIZE_MAX + 3 & !(3);
fn lzma2_bound(uncompressed_size: u64) -> u64 {
    if uncompressed_size > COMPRESSED_SIZE_MAX as u64 {
        return 0;
    }
    let overhead: u64 = uncompressed_size
        .wrapping_add(LZMA2_CHUNK_MAX as u64)
        .wrapping_sub(1)
        .wrapping_div(LZMA2_CHUNK_MAX as u64)
        .wrapping_mul(LZMA2_HEADER_UNCOMPRESSED as u64)
        .wrapping_add(1);
    if (COMPRESSED_SIZE_MAX as u64).wrapping_sub(overhead) < uncompressed_size {
        return 0;
    }
    uncompressed_size + overhead
}
pub fn lzma_block_buffer_bound64(uncompressed_size: u64) -> u64 {
    let mut lzma2_size: u64 = lzma2_bound(uncompressed_size);
    if lzma2_size == 0 {
        return 0;
    }
    lzma2_size = (lzma2_size + 3) & !(3);
    HEADERS_BOUND as u64 + lzma2_size
}
fn size_t_bound_or_zero_with_max(ret: u64, size_max: u64) -> size_t {
    if ret > size_max { 0 } else { ret as size_t }
}
fn size_t_bound_or_zero(ret: u64) -> size_t {
    size_t_bound_or_zero_with_max(ret, size_t::MAX as u64)
}
pub fn lzma_block_buffer_bound(uncompressed_size: size_t) -> size_t {
    let ret: u64 = lzma_block_buffer_bound64(uncompressed_size as u64);
    size_t_bound_or_zero(ret)
}
unsafe fn block_encode_uncompressed(
    block: *mut lzma_block,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    let mut lzma2: lzma_options_lzma = lzma_options_lzma {
        dict_size: LZMA_DICT_SIZE_MIN as u32,
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
        ext_size_low: 0,
        ext_size_high: 0,
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
    let mut filters: [lzma_filter; 2] = [lzma_filter {
        id: 0,
        options: core::ptr::null_mut(),
    }; 2];
    filters[0].id = LZMA_FILTER_LZMA2;
    filters[0].options = ::core::ptr::addr_of_mut!(lzma2) as *mut c_void;
    filters[1].id = LZMA_VLI_UNKNOWN;
    let filters_orig: *mut lzma_filter = (*block).filters;
    (*block).filters = ::core::ptr::addr_of_mut!(filters) as *mut lzma_filter;
    if lzma_block_header_size(&mut *block) != LZMA_OK {
        (*block).filters = filters_orig;
        return LZMA_PROG_ERROR;
    }
    if ((out_size - *out_pos) as lzma_vli)
        < (*block).header_size as lzma_vli + (*block).compressed_size
    {
        (*block).filters = filters_orig;
        return LZMA_BUF_ERROR;
    }
    if lzma_block_header_encode(&*block, c_slice_mut(out.add(*out_pos), out_size - *out_pos))
        != LZMA_OK
    {
        (*block).filters = filters_orig;
        return LZMA_PROG_ERROR;
    }
    (*block).filters = filters_orig;
    *out_pos += (*block).header_size as size_t;
    let mut in_pos: size_t = 0;
    let mut control: u8 = 0x1 as u8;
    while in_pos < in_size {
        *out.add(*out_pos) = control;
        *out_pos += 1;
        control = 0x2 as u8;
        let copy_size: size_t = if in_size - in_pos < (1u32 << 16) as size_t {
            in_size - in_pos
        } else {
            (1u32 << 16) as size_t
        };
        *out.add(*out_pos) = ((copy_size - 1) >> 8) as u8;
        *out_pos += 1;
        *out.add(*out_pos) = ((copy_size - 1) & 0xff) as u8;
        *out_pos += 1;
        core::ptr::copy_nonoverlapping(
            input.add(in_pos) as *const u8,
            out.add(*out_pos) as *mut u8,
            copy_size,
        );
        in_pos += copy_size;
        *out_pos += copy_size;
    }
    *out.add(*out_pos) = 0;
    *out_pos += 1;
    LZMA_OK
}
unsafe fn block_encode_normal(
    block: *mut lzma_block,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    mut out_size: size_t,
) -> lzma_ret {
    let ret_: lzma_ret = lzma_block_header_size(&mut *block);
    if ret_ != LZMA_OK {
        return ret_;
    }
    if out_size - *out_pos <= (*block).header_size as size_t {
        return LZMA_BUF_ERROR;
    }
    let out_start: size_t = *out_pos;
    *out_pos += (*block).header_size as size_t;
    if (out_size - *out_pos) as lzma_vli > (*block).compressed_size {
        out_size = (*out_pos as lzma_vli + (*block).compressed_size) as size_t;
    }
    let mut raw_encoder: lzma_next_coder = lzma_next_coder_s {
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
    let mut ret: lzma_ret = lzma_raw_encoder_init(
        ::core::ptr::addr_of_mut!(raw_encoder),
        allocator,
        (*block).filters,
    );
    if ret == LZMA_OK {
        let mut in_pos: size_t = 0;
        ret = match raw_encoder.code {
            Some(code) => code(
                raw_encoder.coder,
                allocator,
                input,
                ::core::ptr::addr_of_mut!(in_pos),
                in_size,
                out,
                out_pos,
                out_size,
                LZMA_FINISH,
            ),
            None => LZMA_PROG_ERROR,
        };
    }
    lzma_next_end(::core::ptr::addr_of_mut!(raw_encoder), allocator);
    if ret == LZMA_STREAM_END {
        (*block).compressed_size =
            (*out_pos - (out_start + (*block).header_size as size_t)) as lzma_vli;
        ret = lzma_block_header_encode(
            &*block,
            c_slice_mut(out.add(out_start), out_size - out_start),
        );
        if ret != LZMA_OK {
            ret = LZMA_PROG_ERROR;
        }
    } else if ret == LZMA_OK {
        ret = LZMA_BUF_ERROR;
    }
    if ret != LZMA_OK {
        *out_pos = out_start;
    }
    ret
}
unsafe fn block_buffer_encode(
    block: *mut lzma_block,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    mut out_size: size_t,
    try_to_compress: bool,
) -> lzma_ret {
    if block.is_null()
        || input.is_null() && in_size != 0
        || out.is_null()
        || out_pos.is_null()
        || *out_pos > out_size
    {
        return LZMA_PROG_ERROR;
    }
    if (*block).version > 1 {
        return LZMA_OPTIONS_ERROR;
    }
    if (*block).check as c_uint > LZMA_CHECK_ID_MAX as c_uint
        || try_to_compress && (*block).filters.is_null()
    {
        return LZMA_PROG_ERROR;
    }
    if lzma_check_is_supported((*block).check) == 0 {
        return LZMA_UNSUPPORTED_CHECK;
    }
    out_size -= (out_size - *out_pos) & 3;
    let check_size: size_t = lzma_check_size((*block).check) as size_t;
    if out_size - *out_pos <= check_size {
        return LZMA_BUF_ERROR;
    }
    out_size -= check_size;
    (*block).uncompressed_size = in_size as lzma_vli;
    (*block).compressed_size = lzma2_bound(in_size as u64) as lzma_vli;
    if (*block).compressed_size == 0 {
        return LZMA_DATA_ERROR;
    }
    let mut ret: lzma_ret = LZMA_BUF_ERROR;
    if try_to_compress {
        ret = block_encode_normal(block, allocator, input, in_size, out, out_pos, out_size);
    }
    if ret != LZMA_OK {
        if ret != LZMA_BUF_ERROR {
            return ret;
        }
        let ret_: lzma_ret =
            block_encode_uncompressed(block, input, in_size, out, out_pos, out_size);
        if ret_ != LZMA_OK {
            return ret_;
        }
    }
    let mut i: size_t = (*block).compressed_size as size_t;
    while i & 3 != 0 {
        *out.add(*out_pos) = 0;
        *out_pos += 1;
        i += 1;
    }
    if check_size > 0 {
        let mut check: lzma_check_state = lzma_check_state {
            buffer: lzma_check_state_buffer { u8_0: [0; 64] },
            state: lzma_check_state_inner { crc32: 0 },
        };
        lzma_check_init(::core::ptr::addr_of_mut!(check), (*block).check);
        lzma_check_update(
            ::core::ptr::addr_of_mut!(check),
            (*block).check,
            input,
            in_size,
        );
        lzma_check_finish(::core::ptr::addr_of_mut!(check), (*block).check);
        core::ptr::copy_nonoverlapping(
            ::core::ptr::addr_of_mut!(check.buffer.u8_0) as *const u8,
            ::core::ptr::addr_of_mut!((*block).raw_check) as *mut u8,
            check_size,
        );
        core::ptr::copy_nonoverlapping(
            ::core::ptr::addr_of_mut!(check.buffer.u8_0) as *const u8,
            out.add(*out_pos) as *mut u8,
            check_size,
        );
        *out_pos += check_size;
    }
    LZMA_OK
}
pub unsafe fn lzma_block_buffer_encode(
    block: *mut lzma_block,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    block_buffer_encode(
        block, allocator, input, in_size, out, out_pos, out_size, true,
    )
}
pub unsafe fn lzma_block_uncomp_encode(
    block: *mut lzma_block,
    input: *const u8,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    block_buffer_encode(
        block,
        core::ptr::null(),
        input,
        in_size,
        out,
        out_pos,
        out_size,
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::{size_t_bound_or_zero, size_t_bound_or_zero_with_max};
    use crate::types::{
        COMPRESSED_SIZE_MAX, LZMA_BLOCK_HEADER_SIZE_MAX, LZMA_CHECK_SIZE_MAX, LZMA_VLI_MAX, size_t,
    };

    #[test]
    fn compressed_size_max_matches_c_masking() {
        assert_eq!(
            COMPRESSED_SIZE_MAX,
            (LZMA_VLI_MAX - LZMA_BLOCK_HEADER_SIZE_MAX as u64 - LZMA_CHECK_SIZE_MAX as u64) & !3u64
        );
        assert_eq!(COMPRESSED_SIZE_MAX & 3, 0);
    }

    #[test]
    fn block_buffer_bound_rejects_values_over_size_t_max() {
        assert_eq!(size_t_bound_or_zero(123), 123 as size_t);

        let simulated_32_bit_size_max = u32::MAX as u64;
        assert_eq!(
            size_t_bound_or_zero_with_max(simulated_32_bit_size_max + 1, simulated_32_bit_size_max),
            0
        );
    }
}
