use crate::types::*;
use core::mem::MaybeUninit;
pub fn lzma_easy_decoder_memusage(preset: u32) -> u64 {
    // See lzma_easy_encoder for why this is zeroed.
    let mut opt_easy = unsafe { MaybeUninit::<lzma_options_easy>::zeroed().assume_init() };
    if lzma_easy_preset(&mut opt_easy, preset) {
        return UINT32_MAX as u64;
    }
    unsafe {
        lzma_raw_decoder_memusage(::core::ptr::addr_of_mut!(opt_easy.filters) as *mut lzma_filter)
    }
}
