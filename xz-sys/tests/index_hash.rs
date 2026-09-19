#![cfg(not(target_family = "wasm"))]

// Port of vendor/xz/tests/test_index_hash.c
//
// No test included for lzma_index_hash_end since it would be trivial
// unless tested for memory leaks with something like valgrind.

use std::ptr;

use xz_core::types::{INDEX_INDICATOR, UNPADDED_SIZE_MAX, UNPADDED_SIZE_MIN, vli_ceil4};
use xz_sys::{
    LZMA_BUF_ERROR, LZMA_DATA_ERROR, LZMA_OK, LZMA_PROG_ERROR, LZMA_STREAM_END, LZMA_VLI_MAX,
    lzma_crc32, lzma_index_hash, lzma_index_hash_append, lzma_index_hash_decode,
    lzma_index_hash_end, lzma_index_hash_init, lzma_index_hash_size, lzma_vli, lzma_vli_encode,
};

#[test]
fn test_lzma_index_hash_init() {
    unsafe {
        // First test with NULL index_hash.
        // This should create a fresh index_hash.
        let index_hash = lzma_index_hash_init(ptr::null_mut(), ptr::null());
        assert!(!index_hash.is_null());

        // Next test with non-NULL index_hash.
        let second_hash = lzma_index_hash_init(index_hash, ptr::null());

        // It should not create a new index_hash pointer.
        // Instead it must just re-init the first index_hash.
        assert_eq!(index_hash, second_hash);

        lzma_index_hash_end(index_hash, ptr::null());
    }
}

#[test]
fn test_lzma_index_hash_append() {
    unsafe {
        // Test all invalid parameters
        assert_eq!(
            lzma_index_hash_append(ptr::null_mut(), 0, 0),
            LZMA_PROG_ERROR
        );

        // Test NULL index_hash
        assert_eq!(
            lzma_index_hash_append(ptr::null_mut(), UNPADDED_SIZE_MIN, LZMA_VLI_MAX),
            LZMA_PROG_ERROR
        );

        // Test with invalid Unpadded Size
        let index_hash = lzma_index_hash_init(ptr::null_mut(), ptr::null());
        assert!(!index_hash.is_null());
        assert_eq!(
            lzma_index_hash_append(index_hash, UNPADDED_SIZE_MIN - 1, LZMA_VLI_MAX),
            LZMA_PROG_ERROR
        );

        // Test with invalid Uncompressed Size
        assert_eq!(
            lzma_index_hash_append(index_hash, UNPADDED_SIZE_MIN, LZMA_VLI_MAX + 1),
            LZMA_PROG_ERROR
        );

        // First append a Record describing a small Block.
        // This should succeed.
        assert_eq!(
            lzma_index_hash_append(index_hash, UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );

        // Append another small Record.
        assert_eq!(
            lzma_index_hash_append(index_hash, UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );

        // Append a Record that would cause the compressed size to grow
        // too big
        assert_eq!(
            lzma_index_hash_append(index_hash, UNPADDED_SIZE_MAX, 1),
            LZMA_DATA_ERROR
        );

        lzma_index_hash_end(index_hash, ptr::null());
    }
}

// Fill an index_hash with unpadded and uncompressed VLIs
// by calling lzma_index_hash_append
unsafe fn fill_index_hash(
    index_hash: *mut lzma_index_hash,
    unpadded_sizes: &[lzma_vli],
    uncomp_sizes: &[lzma_vli],
    block_count: u32,
) {
    for i in 0..block_count as usize {
        assert_eq!(
            lzma_index_hash_append(index_hash, unpadded_sizes[i], uncomp_sizes[i]),
            LZMA_OK
        );
    }
}

// Set the contents of buf to the expected Index based on the
// .xz specification. This needs the unpadded and uncompressed VLIs
// to correctly create the Index.
unsafe fn generate_index(
    buf: &mut [u8],
    unpadded_sizes: &[lzma_vli],
    uncomp_sizes: &[lzma_vli],
    block_count: u32,
    index_max_size: usize,
) {
    let mut in_pos: usize = 0;
    let mut out_pos: usize = 0;

    // First set Index Indicator
    buf[out_pos] = INDEX_INDICATOR;
    out_pos += 1;

    // Next write out Number of Records
    assert_eq!(
        lzma_vli_encode(
            lzma_vli::from(block_count),
            &mut in_pos,
            buf.as_mut_ptr(),
            &mut out_pos,
            index_max_size,
        ),
        LZMA_STREAM_END
    );

    // Next write out each Record.
    // A Record consists of Unpadded Size and Uncompressed Size
    // written next to each other as VLIs.
    for i in 0..block_count as usize {
        in_pos = 0;
        assert_eq!(
            lzma_vli_encode(
                unpadded_sizes[i],
                &mut in_pos,
                buf.as_mut_ptr(),
                &mut out_pos,
                index_max_size,
            ),
            LZMA_STREAM_END
        );
        in_pos = 0;
        assert_eq!(
            lzma_vli_encode(
                uncomp_sizes[i],
                &mut in_pos,
                buf.as_mut_ptr(),
                &mut out_pos,
                index_max_size,
            ),
            LZMA_STREAM_END
        );
    }

    // Add Index Padding
    let rounded_out_pos = vli_ceil4(out_pos as lzma_vli) as usize;
    buf[out_pos..rounded_out_pos].fill(0);
    out_pos = rounded_out_pos;

    // Add the CRC32
    let crc = lzma_crc32(buf.as_ptr(), out_pos, 0);
    buf[out_pos..out_pos + 4].copy_from_slice(&crc.to_le_bytes());
    out_pos += 4;

    assert_eq!(out_pos, index_max_size);
}

#[test]
fn test_lzma_index_hash_decode() {
    unsafe {
        let mut index_hash = lzma_index_hash_init(ptr::null_mut(), ptr::null());
        assert!(!index_hash.is_null());

        let mut in_pos: usize;

        // Six valid values for the Unpadded Size fields in an Index
        let unpadded_sizes: [lzma_vli; 6] = [UNPADDED_SIZE_MIN, 1000, 4000, 8000, 16000, 32000];

        // Six valid values for the Uncompressed Size fields in an Index
        let uncomp_sizes: [lzma_vli; 6] = [1, 500, 8000, 20, 1, 500];

        // Add two Records to an index_hash
        fill_index_hash(index_hash, &unpadded_sizes, &uncomp_sizes, 2);

        let size_two_records = lzma_index_hash_size(index_hash) as usize;
        assert!(size_two_records > 0);
        let mut index_two_records = vec![0u8; size_two_records];

        generate_index(
            &mut index_two_records,
            &unpadded_sizes,
            &uncomp_sizes,
            2,
            size_two_records,
        );

        // First test for basic buffer size error
        in_pos = size_two_records + 1;
        assert_eq!(
            lzma_index_hash_decode(
                index_hash,
                index_two_records.as_ptr(),
                &mut in_pos,
                size_two_records,
            ),
            LZMA_BUF_ERROR
        );

        // Next test for invalid Index Indicator
        in_pos = 0;
        index_two_records[0] ^= 1;
        assert_eq!(
            lzma_index_hash_decode(
                index_hash,
                index_two_records.as_ptr(),
                &mut in_pos,
                size_two_records,
            ),
            LZMA_DATA_ERROR
        );
        index_two_records[0] ^= 1;

        // Next verify the index_hash as expected
        in_pos = 0;
        assert_eq!(
            lzma_index_hash_decode(
                index_hash,
                index_two_records.as_ptr(),
                &mut in_pos,
                size_two_records,
            ),
            LZMA_STREAM_END
        );

        // Next test an index_hash with three Records
        index_hash = lzma_index_hash_init(index_hash, ptr::null());
        fill_index_hash(index_hash, &unpadded_sizes, &uncomp_sizes, 3);

        let size_three_records = lzma_index_hash_size(index_hash) as usize;
        assert!(size_three_records > 0);
        let mut index_three_records = vec![0u8; size_three_records];

        generate_index(
            &mut index_three_records,
            &unpadded_sizes,
            &uncomp_sizes,
            3,
            size_three_records,
        );

        in_pos = 0;
        assert_eq!(
            lzma_index_hash_decode(
                index_hash,
                index_three_records.as_ptr(),
                &mut in_pos,
                size_three_records,
            ),
            LZMA_STREAM_END
        );

        // Next test an index_hash with five Records
        index_hash = lzma_index_hash_init(index_hash, ptr::null());
        fill_index_hash(index_hash, &unpadded_sizes, &uncomp_sizes, 5);

        let size_five_records = lzma_index_hash_size(index_hash) as usize;
        assert!(size_five_records > 0);
        let mut index_five_records = vec![0u8; size_five_records];

        generate_index(
            &mut index_five_records,
            &unpadded_sizes,
            &uncomp_sizes,
            5,
            size_five_records,
        );

        // Instead of testing all input at once, give input
        // one byte at a time
        in_pos = 0;
        for _ in 0..size_five_records - 1 {
            assert_eq!(
                lzma_index_hash_decode(
                    index_hash,
                    index_five_records.as_ptr(),
                    &mut in_pos,
                    in_pos + 1,
                ),
                LZMA_OK
            );
        }

        // Last byte should return LZMA_STREAM_END
        assert_eq!(
            lzma_index_hash_decode(
                index_hash,
                index_five_records.as_ptr(),
                &mut in_pos,
                in_pos + 1,
            ),
            LZMA_STREAM_END
        );

        // Next test if the index_hash is given an incorrect Unpadded
        // Size. Should detect and report LZMA_DATA_ERROR
        index_hash = lzma_index_hash_init(index_hash, ptr::null());
        fill_index_hash(index_hash, &unpadded_sizes, &uncomp_sizes, 5);
        // The sixth Record will have an invalid Unpadded Size
        assert_eq!(
            lzma_index_hash_append(index_hash, unpadded_sizes[5] + 1, uncomp_sizes[5]),
            LZMA_OK
        );

        let size_six_records = lzma_index_hash_size(index_hash) as usize;
        assert!(size_six_records > 0);
        let mut index_six_records = vec![0u8; size_six_records];

        generate_index(
            &mut index_six_records,
            &unpadded_sizes,
            &uncomp_sizes,
            6,
            size_six_records,
        );
        in_pos = 0;
        assert_eq!(
            lzma_index_hash_decode(
                index_hash,
                index_six_records.as_ptr(),
                &mut in_pos,
                size_six_records,
            ),
            LZMA_DATA_ERROR
        );

        // Next test if the Index is corrupt (invalid CRC32).
        // Should detect and report LZMA_DATA_ERROR
        index_hash = lzma_index_hash_init(index_hash, ptr::null());
        fill_index_hash(index_hash, &unpadded_sizes, &uncomp_sizes, 2);

        index_two_records[size_two_records - 1] ^= 1;

        in_pos = 0;
        assert_eq!(
            lzma_index_hash_decode(
                index_hash,
                index_two_records.as_ptr(),
                &mut in_pos,
                size_two_records,
            ),
            LZMA_DATA_ERROR
        );

        // Next test with Index and index_hash struct not matching
        // a Record
        index_hash = lzma_index_hash_init(index_hash, ptr::null());
        fill_index_hash(index_hash, &unpadded_sizes, &uncomp_sizes, 2);
        // Recalculate Index with invalid Unpadded Size
        let unpadded_sizes_invalid: [lzma_vli; 2] = [unpadded_sizes[0], unpadded_sizes[1] + 1];

        generate_index(
            &mut index_two_records,
            &unpadded_sizes_invalid,
            &uncomp_sizes,
            2,
            size_two_records,
        );

        in_pos = 0;
        assert_eq!(
            lzma_index_hash_decode(
                index_hash,
                index_two_records.as_ptr(),
                &mut in_pos,
                size_two_records,
            ),
            LZMA_DATA_ERROR
        );

        lzma_index_hash_end(index_hash, ptr::null());
    }
}

#[test]
fn test_lzma_index_hash_size() {
    unsafe {
        let index_hash = lzma_index_hash_init(ptr::null_mut(), ptr::null());
        assert!(!index_hash.is_null());

        // First test empty index_hash
        // Expected size should be:
        // Index Indicator - 1 byte
        // Number of Records - 1 byte
        // List of Records - 0 bytes
        // Index Padding - 2 bytes
        // CRC32 - 4 bytes
        // Total - 8 bytes
        assert_eq!(lzma_index_hash_size(index_hash), 8);

        // Append a Record describing a small Block to the index_hash
        assert_eq!(
            lzma_index_hash_append(index_hash, UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );

        // Expected size should be:
        // Index Indicator - 1 byte
        // Number of Records - 1 byte
        // List of Records - 2 bytes
        // Index Padding - 0 bytes
        // CRC32 - 4 bytes
        // Total - 8 bytes
        let mut expected_size: lzma_vli = 8;
        assert_eq!(lzma_index_hash_size(index_hash), expected_size);

        // Append additional small Record
        assert_eq!(
            lzma_index_hash_append(index_hash, UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );

        // Expected size should be:
        // Index Indicator - 1 byte
        // Number of Records - 1 byte
        // List of Records - 4 bytes
        // Index Padding - 2 bytes
        // CRC32 - 4 bytes
        // Total - 12 bytes
        expected_size = 12;
        assert_eq!(lzma_index_hash_size(index_hash), expected_size);

        // Append a larger Record to the index_hash (3 bytes for each VLI)
        let three_byte_vli: lzma_vli = 0x10000;
        assert_eq!(
            lzma_index_hash_append(index_hash, three_byte_vli, three_byte_vli),
            LZMA_OK
        );

        // Expected size should be:
        // Index Indicator - 1 byte
        // Number of Records - 1 byte
        // List of Records - 10 bytes
        // Index Padding - 0 bytes
        // CRC32 - 4 bytes
        // Total - 16 bytes
        expected_size = 16;
        assert_eq!(lzma_index_hash_size(index_hash), expected_size);

        lzma_index_hash_end(index_hash, ptr::null());
    }
}
