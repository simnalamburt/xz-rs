use crate::types::*;
pub const LZMA_BLOCK_HEADER_SIZE_MIN: u32 = 8;
pub unsafe fn lzma_block_compressed_size(
    block: &mut lzma_block,
    unpadded_size: lzma_vli,
) -> lzma_ret {
    if lzma_block_unpadded_size(block) == 0 {
        return LZMA_PROG_ERROR;
    }
    let container_size: u32 = block.header_size + lzma_check_size(block.check) as u32;
    if unpadded_size <= container_size as lzma_vli {
        return LZMA_DATA_ERROR;
    }
    let compressed_size: lzma_vli = unpadded_size - container_size as lzma_vli;
    if block.compressed_size != LZMA_VLI_UNKNOWN && block.compressed_size != compressed_size {
        return LZMA_DATA_ERROR;
    }
    block.compressed_size = compressed_size;
    LZMA_OK
}
pub unsafe fn lzma_block_unpadded_size(block: &lzma_block) -> lzma_vli {
    if block.version > 1
        || block.header_size < LZMA_BLOCK_HEADER_SIZE_MIN
        || block.header_size > LZMA_BLOCK_HEADER_SIZE_MAX
        || block.header_size & 3 != 0
        || !(block.compressed_size <= LZMA_VLI_MAX || block.compressed_size == LZMA_VLI_UNKNOWN)
        || block.compressed_size == 0
        || block.check as c_uint > LZMA_CHECK_ID_MAX as c_uint
    {
        return 0;
    }
    if block.compressed_size == LZMA_VLI_UNKNOWN {
        return LZMA_VLI_UNKNOWN;
    }
    let unpadded_size: lzma_vli = block.compressed_size
        + block.header_size as lzma_vli
        + lzma_check_size(block.check) as lzma_vli;
    if unpadded_size > UNPADDED_SIZE_MAX {
        return 0;
    }
    unpadded_size
}
pub unsafe fn lzma_block_total_size(block: &lzma_block) -> lzma_vli {
    let mut unpadded_size: lzma_vli = lzma_block_unpadded_size(block);
    if unpadded_size != LZMA_VLI_UNKNOWN {
        unpadded_size = vli_ceil4(unpadded_size);
    }
    unpadded_size
}
