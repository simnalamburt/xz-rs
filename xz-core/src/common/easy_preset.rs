use crate::types::*;
pub fn lzma_easy_preset(opt_easy: &mut lzma_options_easy, preset: u32) -> bool {
    if lzma_lzma_preset(&mut opt_easy.opt_lzma, preset) != 0 {
        return true;
    }
    // The filter chain points back into the same struct, so the caller must
    // keep `opt_easy` in place for as long as the chain is used.
    let opt_lzma = (&raw mut opt_easy.opt_lzma).cast::<c_void>();
    opt_easy.filters[0].id = LZMA_FILTER_LZMA2;
    opt_easy.filters[0].options = opt_lzma;
    opt_easy.filters[1].id = LZMA_VLI_UNKNOWN;
    false
}
