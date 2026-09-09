use crate::common::filter_flags_encoder::{lzma_filter_flags_encode, lzma_filter_flags_size};
use crate::types::*;
/// # Safety
/// `block.filters` must be a NULL-or-`LZMA_VLI_UNKNOWN`-terminated array of at
/// most `LZMA_FILTERS_MAX` entries; it is walked as a bare pointer.
pub unsafe fn lzma_block_header_size(block: &mut lzma_block) -> lzma_ret {
    if block.version > 1 {
        return LZMA_OPTIONS_ERROR;
    }
    let mut size: u32 = (1 + 1 + 4) as u32;
    if block.compressed_size != LZMA_VLI_UNKNOWN {
        let add: u32 = lzma_vli_size(block.compressed_size) as u32;
        if add == 0 || block.compressed_size == 0 {
            return LZMA_PROG_ERROR;
        }
        size += add;
    }
    if block.uncompressed_size != LZMA_VLI_UNKNOWN {
        let add_0: u32 = lzma_vli_size(block.uncompressed_size) as u32;
        if add_0 == 0 {
            return LZMA_PROG_ERROR;
        }
        size += add_0;
    }
    if block.filters.is_null() || (*block.filters).id == LZMA_VLI_UNKNOWN {
        return LZMA_PROG_ERROR;
    }
    let mut i: size_t = 0;
    while (*block.filters.add(i)).id != LZMA_VLI_UNKNOWN {
        if i == LZMA_FILTERS_MAX as size_t {
            return LZMA_PROG_ERROR;
        }
        let mut add_1: u32 = 0;
        let ret_: lzma_ret = lzma_filter_flags_size(&mut add_1, &*block.filters.add(i));
        if ret_ != LZMA_OK {
            return ret_;
        }
        size += add_1;
        i += 1;
    }
    block.header_size = (size + 3) & !(3);
    LZMA_OK
}
/// Encode the Block Header into the first `block.header_size` bytes of `out`.
/// Call [`lzma_block_header_size`] first; a shorter `out` is rejected rather
/// than overrun.
///
/// # Safety
/// Same filter-array contract as [`lzma_block_header_size`].
pub unsafe fn lzma_block_header_encode(block: &lzma_block, out: &mut [u8]) -> lzma_ret {
    if lzma_block_unpadded_size(block) == 0
        || !(block.uncompressed_size <= LZMA_VLI_MAX || block.uncompressed_size == LZMA_VLI_UNKNOWN)
    {
        return LZMA_PROG_ERROR;
    }
    let header_size = block.header_size as size_t;
    let out_size: size_t = header_size - 4;
    let Some(out) = out.get_mut(..header_size) else {
        return LZMA_PROG_ERROR;
    };
    let (out, crc_out) = out.split_at_mut(out_size);
    let Some(crc_out) = crc_out.first_chunk_mut::<4>() else {
        return LZMA_PROG_ERROR;
    };
    out[0] = (out_size / 4) as u8;
    out[1] = 0;
    let mut out_pos: size_t = 2;
    if block.compressed_size != LZMA_VLI_UNKNOWN {
        let ret_: lzma_ret = lzma_vli_encode(block.compressed_size, None, out, &mut out_pos);
        if ret_ != LZMA_OK {
            return ret_;
        }
        out[1] |= 0x40;
    }
    if block.uncompressed_size != LZMA_VLI_UNKNOWN {
        let ret__0: lzma_ret = lzma_vli_encode(block.uncompressed_size, None, out, &mut out_pos);
        if ret__0 != LZMA_OK {
            return ret__0;
        }
        out[1] |= 0x80;
    }
    if block.filters.is_null() || (*block.filters).id == LZMA_VLI_UNKNOWN {
        return LZMA_PROG_ERROR;
    }
    let mut filter_count: size_t = 0;
    loop {
        if filter_count == LZMA_FILTERS_MAX as size_t {
            return LZMA_PROG_ERROR;
        }
        let ret__1: lzma_ret =
            lzma_filter_flags_encode(&*block.filters.add(filter_count), out, &mut out_pos);
        if ret__1 != LZMA_OK {
            return ret__1;
        }
        filter_count += 1;
        if (*block.filters.add(filter_count)).id == LZMA_VLI_UNKNOWN {
            break;
        }
    }
    out[1] |= (filter_count - 1) as u8;
    out[out_pos..].fill(0);
    write32le(crc_out, crc32(out, 0));
    LZMA_OK
}
