#![cfg(not(target_family = "wasm"))]

use std::ptr;

use xz_sys::{
    LZMA_DATA_ERROR, LZMA_FILTERS_MAX, lzma_block, lzma_block_header_decode, lzma_filter,
};

/// A four-byte Block Header leaves nothing for the CRC to cover, and the CRC of
/// an empty buffer is zero, so `[0, 0, 0, 0]` gets past the checksum test. The
/// flags byte is then read from inside what the CRC occupies. C reads it there
/// and goes on to fail on the filter, and so must this.
#[test]
fn four_byte_header_is_a_data_error_not_a_crash() {
    unsafe {
        let mut filters = [lzma_filter {
            id: 0,
            options: ptr::null_mut(),
        }; LZMA_FILTERS_MAX as usize + 1];
        let mut block: lzma_block = std::mem::zeroed();
        block.version = 1;
        block.header_size = 4;
        block.filters = filters.as_mut_ptr();

        let input = [0u8; 4];
        assert_eq!(
            lzma_block_header_decode(&mut block, ptr::null(), input.as_ptr()),
            LZMA_DATA_ERROR
        );

        // Same input through the C library, to show the code is not invented.
        let mut c_filters = [liblzma_sys::lzma_filter {
            id: 0,
            options: ptr::null_mut(),
        }; LZMA_FILTERS_MAX as usize + 1];
        let mut c_block: liblzma_sys::lzma_block = std::mem::zeroed();
        c_block.version = 1;
        c_block.header_size = 4;
        c_block.filters = c_filters.as_mut_ptr();
        assert_eq!(
            liblzma_sys::lzma_block_header_decode(&mut c_block, ptr::null(), input.as_ptr()),
            liblzma_sys::LZMA_DATA_ERROR
        );
    }
}
