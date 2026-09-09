use crate::common::filter_encoder::{lzma_properties_encode, lzma_properties_size};
use crate::types::*;
/// # Safety
/// Same `filter.options` contract as [`lzma_properties_size`].
pub unsafe fn lzma_filter_flags_size(size: &mut u32, filter: &lzma_filter) -> lzma_ret {
    if filter.id >= LZMA_FILTER_RESERVED_START {
        return LZMA_PROG_ERROR;
    }
    let ret_: lzma_ret = lzma_properties_size(size, filter);
    if ret_ != LZMA_OK {
        return ret_;
    }
    *size += lzma_vli_size(filter.id) + lzma_vli_size(*size as lzma_vli);
    LZMA_OK
}
/// # Safety
/// Same `filter.options` contract as [`lzma_properties_size`].
pub unsafe fn lzma_filter_flags_encode(
    filter: &lzma_filter,
    out: &mut [u8],
    out_pos: &mut size_t,
) -> lzma_ret {
    if filter.id >= LZMA_FILTER_RESERVED_START {
        return LZMA_PROG_ERROR;
    }
    let ret_: lzma_ret = lzma_vli_encode(filter.id, None, out, out_pos);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let mut props_size: u32 = 0;
    let ret__0: lzma_ret = lzma_properties_size(&mut props_size, filter);
    if ret__0 != LZMA_OK {
        return ret__0;
    }
    let ret__1: lzma_ret = lzma_vli_encode(props_size as lzma_vli, None, out, out_pos);
    if ret__1 != LZMA_OK {
        return ret__1;
    }
    let props_size = props_size as size_t;
    if out.len() - *out_pos < props_size {
        return LZMA_PROG_ERROR;
    }
    let ret__2: lzma_ret = lzma_properties_encode(filter, &mut out[*out_pos..][..props_size]);
    if ret__2 != LZMA_OK {
        return ret__2;
    }
    *out_pos += props_size;
    LZMA_OK
}
