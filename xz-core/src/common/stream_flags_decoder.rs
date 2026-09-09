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

fn stream_flags_decode(options: &mut lzma_stream_flags, input: &[u8; FLAGS]) -> bool {
    if input[0] != 0 || input[1] & 0xf0 != 0 {
        return true;
    }
    options.version = 0;
    options.check = (input[1] & 0xf) as lzma_check;
    false
}

pub fn lzma_stream_header_decode(options: &mut lzma_stream_flags, input: &[u8; SIZE]) -> lzma_ret {
    if *input.subarray::<0, HEADER_MAGIC_SIZE>() != lzma_header_magic {
        return LZMA_FORMAT_ERROR;
    }
    let crc = crc32(input.subarray::<HEADER_MAGIC_SIZE, FLAGS>(), 0);
    if crc != read32le(input.subarray::<{ HEADER_MAGIC_SIZE + FLAGS }, 4>()) {
        return LZMA_DATA_ERROR;
    }
    if stream_flags_decode(options, input.subarray::<HEADER_MAGIC_SIZE, FLAGS>()) {
        return LZMA_OPTIONS_ERROR;
    }
    options.backward_size = LZMA_VLI_UNKNOWN;
    LZMA_OK
}

pub fn lzma_stream_footer_decode(options: &mut lzma_stream_flags, input: &[u8; SIZE]) -> lzma_ret {
    if *input.subarray::<{ FOOTER_FLAGS_OFFSET + FLAGS }, 2>() != lzma_footer_magic {
        return LZMA_FORMAT_ERROR;
    }
    let crc = crc32(
        input.subarray::<FOOTER_CRC_SIZE, { FOOTER_BACKWARD_SIZE + FLAGS }>(),
        0,
    );
    if crc != read32le(input.subarray::<0, 4>()) {
        return LZMA_DATA_ERROR;
    }
    if stream_flags_decode(options, input.subarray::<FOOTER_FLAGS_OFFSET, FLAGS>()) {
        return LZMA_OPTIONS_ERROR;
    }
    options.backward_size = read32le(input.subarray::<FOOTER_CRC_SIZE, 4>()) as lzma_vli;
    options.backward_size = (options.backward_size + 1) * 4;
    LZMA_OK
}
