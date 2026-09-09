use crate::common::filter_decoder::lzma_properties_decode;
use crate::types::*;
/// # Safety
/// On success `filter.options` owns an allocation from `allocator`.
pub unsafe fn lzma_filter_flags_decode(
    filter: &mut lzma_filter,
    allocator: *const lzma_allocator,
    input: &[u8],
    in_pos: &mut size_t,
) -> lzma_ret {
    filter.options = core::ptr::null_mut();
    let ret: lzma_ret = lzma_vli_decode(&mut filter.id, None, input, in_pos);
    if ret != LZMA_OK {
        return ret;
    }
    if filter.id >= LZMA_FILTER_RESERVED_START {
        return LZMA_DATA_ERROR;
    }
    let mut props_size: lzma_vli = 0;
    let ret: lzma_ret = lzma_vli_decode(&mut props_size, None, input, in_pos);
    if ret != LZMA_OK {
        return ret;
    }
    if ((input.len() - *in_pos) as lzma_vli) < props_size {
        return LZMA_DATA_ERROR;
    }
    let props_size = props_size as size_t;
    let ret: lzma_ret = lzma_properties_decode(filter, allocator, &input[*in_pos..][..props_size]);
    *in_pos += props_size;
    ret
}
