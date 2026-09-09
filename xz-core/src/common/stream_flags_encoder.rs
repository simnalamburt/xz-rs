use crate::common::stream_flags_common::{lzma_footer_magic, lzma_header_magic};
use crate::types::*;

/// Stream Header: 6 magic, 2 Stream Flags, 4 CRC32.
const HEADER_MAGIC_SIZE: usize = 6;
/// Stream Footer: 4 CRC32, 4 Backward Size, 2 Stream Flags, 2 magic.
const FOOTER_CRC_SIZE: usize = 4;
const FOOTER_BACKWARD_SIZE: usize = 4;
const FOOTER_FLAGS_OFFSET: usize = FOOTER_CRC_SIZE + FOOTER_BACKWARD_SIZE;

const SIZE: usize = LZMA_STREAM_HEADER_SIZE as usize;
const FLAGS: usize = LZMA_STREAM_FLAGS_SIZE as usize;

fn stream_flags_encode(options: &lzma_stream_flags, out: &mut [u8; FLAGS]) -> bool {
    if options.check as c_uint > LZMA_CHECK_ID_MAX as c_uint {
        return true;
    }
    out[0] = 0;
    out[1] = options.check as u8;
    false
}

pub fn lzma_stream_header_encode(options: &lzma_stream_flags, out: &mut [u8; SIZE]) -> lzma_ret {
    if options.version != 0 {
        return LZMA_OPTIONS_ERROR;
    }
    *out.subarray_mut::<0, HEADER_MAGIC_SIZE>() = lzma_header_magic;
    if stream_flags_encode(options, out.subarray_mut::<HEADER_MAGIC_SIZE, FLAGS>()) {
        return LZMA_PROG_ERROR;
    }
    let crc = crc32(out.subarray::<HEADER_MAGIC_SIZE, FLAGS>(), 0);
    write32le(out.subarray_mut::<{ HEADER_MAGIC_SIZE + FLAGS }, 4>(), crc);
    LZMA_OK
}

pub fn lzma_stream_footer_encode(options: &lzma_stream_flags, out: &mut [u8; SIZE]) -> lzma_ret {
    if options.version != 0 {
        return LZMA_OPTIONS_ERROR;
    }
    if !is_backward_size_valid(options) {
        return LZMA_PROG_ERROR;
    }
    write32le(
        out.subarray_mut::<FOOTER_CRC_SIZE, 4>(),
        options.backward_size.wrapping_div(4).wrapping_sub(1) as u32,
    );
    if stream_flags_encode(options, out.subarray_mut::<FOOTER_FLAGS_OFFSET, FLAGS>()) {
        return LZMA_PROG_ERROR;
    }
    let crc = crc32(
        out.subarray::<FOOTER_CRC_SIZE, { FOOTER_BACKWARD_SIZE + FLAGS }>(),
        0,
    );
    write32le(out.subarray_mut::<0, 4>(), crc);
    *out.subarray_mut::<{ FOOTER_FLAGS_OFFSET + FLAGS }, 2>() = lzma_footer_magic;
    LZMA_OK
}
