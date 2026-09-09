use crate::common::stream_encoder::lzma_stream_encoder;
use crate::types::*;
use core::mem::MaybeUninit;
pub unsafe fn lzma_easy_encoder(
    strm: *mut lzma_stream,
    preset: u32,
    check: lzma_check,
) -> lzma_ret {
    // Zeroed rather than uninit: lzma_easy_preset takes a reference, and a
    // reference is only valid if every field is, including the reserved ones
    // the preset never writes.
    let mut opt_easy = MaybeUninit::<lzma_options_easy>::zeroed().assume_init();
    if lzma_easy_preset(&mut opt_easy, preset) {
        return LZMA_OPTIONS_ERROR;
    }
    lzma_stream_encoder(
        strm,
        ::core::ptr::addr_of_mut!(opt_easy.filters) as *mut lzma_filter,
        check,
    )
}
