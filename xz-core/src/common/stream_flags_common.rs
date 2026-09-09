use crate::types::*;
pub static lzma_header_magic: [u8; 6] = [
    0xfd as u8, 0x37 as u8, 0x7a as u8, 0x58 as u8, 0x5a as u8, 0,
];
pub static lzma_footer_magic: [u8; 2] = [0x59 as u8, 0x5a as u8];
pub fn lzma_stream_flags_compare(a: &lzma_stream_flags, b: &lzma_stream_flags) -> lzma_ret {
    if a.version != 0 || b.version != 0 {
        return LZMA_OPTIONS_ERROR;
    }
    if a.check as c_uint > LZMA_CHECK_ID_MAX as c_uint
        || b.check as c_uint > LZMA_CHECK_ID_MAX as c_uint
    {
        return LZMA_PROG_ERROR;
    }
    if a.check != b.check {
        return LZMA_DATA_ERROR;
    }
    if a.backward_size != LZMA_VLI_UNKNOWN && b.backward_size != LZMA_VLI_UNKNOWN {
        if !is_backward_size_valid(a) || !is_backward_size_valid(b) {
            return LZMA_PROG_ERROR;
        }
        if a.backward_size != b.backward_size {
            return LZMA_DATA_ERROR;
        }
    }
    LZMA_OK
}
