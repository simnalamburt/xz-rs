use crate::common::filter_flags_decoder::lzma_filter_flags_decode;
use crate::types::*;

/// Decode the Block Header from the first `block.header_size` bytes of
/// `input`. `input[0]` must agree with `block.header_size`, which the caller
/// normally learns from `lzma_block_header_size_decode`.
///
/// # Safety
/// `block.filters` must point at an array of `LZMA_FILTERS_MAX + 1` entries;
/// it is filled through a bare pointer. On success each entry's `options`
/// owns an allocation from `allocator`.
pub unsafe fn lzma_block_header_decode(
    block: &mut lzma_block,
    allocator: *const lzma_allocator,
    input: &[u8],
) -> lzma_ret {
    if block.filters.is_null() {
        return LZMA_PROG_ERROR;
    }
    let mut i: size_t = 0;
    while i <= LZMA_FILTERS_MAX as size_t {
        (*block.filters.add(i)).id = LZMA_VLI_UNKNOWN;
        (*block.filters.add(i)).options = core::ptr::null_mut();
        i += 1;
    }
    if block.version > 1 {
        block.version = 1;
    }
    block.ignore_check = false as lzma_bool;
    let Some(&first) = input.first() else {
        return LZMA_PROG_ERROR;
    };
    if (first as u32 + 1) * 4 != block.header_size
        || block.check as c_uint > LZMA_CHECK_ID_MAX as c_uint
    {
        return LZMA_PROG_ERROR;
    }
    let header_size: size_t = block.header_size as size_t;
    let Some(header) = input.get(..header_size) else {
        return LZMA_PROG_ERROR;
    };
    // `header_size` is (input[0] + 1) * 4, so it is at least 4 and the split
    // leaves exactly the four CRC bytes.
    let (input, crc_in) = header.split_at(header_size - 4);
    let Some(crc_in) = crc_in.first_chunk::<4>() else {
        return LZMA_PROG_ERROR;
    };
    if crc32(input, 0) != read32le(crc_in) {
        return LZMA_DATA_ERROR;
    }
    // The flags byte is read from the whole header, not from the part the CRC
    // covers: a four-byte header leaves that part empty, and C still reads
    // here.
    let flags = header[1];
    if flags & 0x3c != 0 {
        return LZMA_OPTIONS_ERROR;
    }
    let mut in_pos: size_t = 2;
    if flags & 0x40 != 0 {
        let ret: lzma_ret = lzma_vli_decode(&mut block.compressed_size, None, input, &mut in_pos);
        if ret != LZMA_OK {
            return ret;
        }
        if lzma_block_unpadded_size(block) == 0 {
            return LZMA_DATA_ERROR;
        }
    } else {
        block.compressed_size = LZMA_VLI_UNKNOWN;
    }
    if flags & 0x80 != 0 {
        let ret: lzma_ret = lzma_vli_decode(&mut block.uncompressed_size, None, input, &mut in_pos);
        if ret != LZMA_OK {
            return ret;
        }
    } else {
        block.uncompressed_size = LZMA_VLI_UNKNOWN;
    }
    let filter_count: size_t = ((u32::from(flags) & 3) + 1) as size_t;
    let mut i_0: size_t = 0;
    while i_0 < filter_count {
        let ret: lzma_ret =
            lzma_filter_flags_decode(&mut *block.filters.add(i_0), allocator, input, &mut in_pos);
        if ret != LZMA_OK {
            lzma_filters_free(block.filters, allocator);
            return ret;
        }
        i_0 += 1;
    }
    if input[in_pos..].iter().any(|&b| b != 0) {
        lzma_filters_free(block.filters, allocator);
        return LZMA_OPTIONS_ERROR;
    }
    LZMA_OK
}
