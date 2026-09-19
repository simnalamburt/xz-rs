#![cfg(not(target_family = "wasm"))]

use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

use xz_core::types::{
    LZMA_INDEX_ITER_ANY, LZMA_INDEX_ITER_BLOCK, LZMA_INDEX_ITER_NONEMPTY_BLOCK,
    LZMA_INDEX_ITER_STREAM, UINT32_MAX, UINT64_MAX, UNPADDED_SIZE_MAX, UNPADDED_SIZE_MIN,
    vli_ceil4,
};
use xz_sys::{
    LZMA_BACKWARD_SIZE_MIN, LZMA_BUF_ERROR, LZMA_CHECK_CRC32, LZMA_CHECK_CRC64, LZMA_CHECK_NONE,
    LZMA_CHECK_SHA256, LZMA_DATA_ERROR, LZMA_FINISH, LZMA_MEMLIMIT_ERROR, LZMA_OK, LZMA_PROG_ERROR,
    LZMA_STREAM_END, LZMA_STREAM_HEADER_SIZE, LZMA_VLI_MAX, lzma_allocator, lzma_code, lzma_crc32,
    lzma_end, lzma_index, lzma_index_append, lzma_index_block_count, lzma_index_buffer_decode,
    lzma_index_buffer_encode, lzma_index_cat, lzma_index_checks, lzma_index_decoder,
    lzma_index_dup, lzma_index_encoder, lzma_index_end, lzma_index_file_size, lzma_index_init,
    lzma_index_iter, lzma_index_iter_init, lzma_index_iter_locate, lzma_index_iter_next,
    lzma_index_iter_rewind, lzma_index_memusage, lzma_index_memused, lzma_index_size,
    lzma_index_stream_count, lzma_index_stream_flags, lzma_index_stream_padding,
    lzma_index_stream_size, lzma_index_total_size, lzma_index_uncompressed_size, lzma_ret,
    lzma_stream, lzma_stream_flags, lzma_vli, lzma_vli_decode,
};

const MEMLIMIT: u64 = 1 << 20;

const LZMA_INDEX_CHECK_MASK_NONE: u32 = 1 << LZMA_CHECK_NONE;
const LZMA_INDEX_CHECK_MASK_CRC32: u32 = 1 << LZMA_CHECK_CRC32;
const LZMA_INDEX_CHECK_MASK_CRC64: u32 = 1 << LZMA_CHECK_CRC64;
const LZMA_INDEX_CHECK_MASK_SHA256: u32 = 1 << LZMA_CHECK_SHA256;

const STREAM_HEADER: lzma_vli = LZMA_STREAM_HEADER_SIZE as lzma_vli;

fn stream_flags_with_check(check: u32) -> lzma_stream_flags {
    let mut flags: lzma_stream_flags = unsafe { MaybeUninit::zeroed().assume_init() };
    flags.version = 0;
    flags.backward_size = LZMA_BACKWARD_SIZE_MIN;
    flags.check = check;
    flags
}

unsafe fn new_iter(idx: *const lzma_index) -> lzma_index_iter {
    let mut iter = MaybeUninit::<lzma_index_iter>::zeroed().assume_init();
    lzma_index_iter_init(&mut iter, idx);
    iter
}

fn new_stream() -> lzma_stream {
    unsafe { MaybeUninit::zeroed().assume_init() }
}

#[test]
fn test_lzma_index_memusage() {
    // The return value from lzma_index_memusage is an approximation
    // of the amount of memory needed for lzma_index for a given
    // amount of Streams and Blocks. It will be an upperbound,
    // so this test will mostly sanity check and error check the
    // function.

    // The maximum number of Streams should be UINT32_MAX in the
    // current implementation even though the parameter is lzma_vli.
    assert_eq!(
        lzma_index_memusage(UINT32_MAX as lzma_vli + 1, 1),
        UINT64_MAX
    );

    // While the number of blocks is lzma_vli, the real maximum value is
    // much smaller than LZMA_VLI_MAX. Just check that it fails with a
    // huge but valid VLI and that it succeeds with a smaller one.
    assert_eq!(lzma_index_memusage(1, LZMA_VLI_MAX / 5), UINT64_MAX);
    assert!(lzma_index_memusage(1, LZMA_VLI_MAX / 11) < UINT64_MAX);

    // Number of Streams must be non-zero
    assert_eq!(lzma_index_memusage(0, 1), UINT64_MAX);

    // Number of Blocks CAN be zero
    assert_ne!(lzma_index_memusage(1, 0), UINT64_MAX);

    // Arbitrary values for Stream and Block should work without error
    // and should always increase
    let mut previous: u64 = 1;
    let mut streams: lzma_vli = 1;
    let mut blocks: lzma_vli = 1;

    // Test 100 different increasing values for Streams and Block
    for _ in 0..100 {
        let current = lzma_index_memusage(streams, blocks);
        assert!(current > previous);
        previous = current;
        streams += 29;
        blocks += 107;
    }

    // Force integer overflow in calculation (should result in an error)
    assert_eq!(
        lzma_index_memusage(UINT32_MAX as lzma_vli, LZMA_VLI_MAX),
        UINT64_MAX
    );
}

#[test]
fn test_lzma_index_memused() {
    unsafe {
        // Very similar to test_lzma_index_memusage above since
        // lzma_index_memused is essentially a wrapper for
        // lzma_index_memusage
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        // Test with empty Index
        assert!(lzma_index_memused(idx) < UINT64_MAX);

        // Append small Blocks and then test again (should pass).
        for _ in 0..10 {
            assert_eq!(
                lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, 1),
                LZMA_OK
            );
        }

        assert!(lzma_index_memused(idx) < UINT64_MAX);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_append() {
    unsafe {
        // Basic input-output test done here.
        // Less trivial tests for this function are done throughout
        // other tests.

        // First test with NULL lzma_index
        assert_eq!(
            lzma_index_append(ptr::null_mut(), ptr::null(), UNPADDED_SIZE_MIN, 1),
            LZMA_PROG_ERROR
        );

        let mut idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        // Test with invalid Unpadded Size
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN - 1, 1),
            LZMA_PROG_ERROR
        );
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MAX + 1, 1),
            LZMA_PROG_ERROR
        );

        // Test with invalid Uncompressed Size
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MAX, LZMA_VLI_MAX + 1),
            LZMA_PROG_ERROR
        );

        // Test expected successful Block appends
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN * 2, 2),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN * 3, 3),
            LZMA_OK
        );

        lzma_index_end(idx, ptr::null());

        // Test compressed .xz file size growing too large.
        // Should result in LZMA_DATA_ERROR.
        idx = lzma_index_init(ptr::null());

        // The calculation for maximum unpadded size is to make room for the
        // second stream when lzma_index_cat() is called. The
        // 4 * LZMA_STREAM_HEADER_SIZE is for the header and footer of
        // both streams. The extra 24 bytes are for the size of the indexes
        // for both streams. This allows us to maximize the unpadded sum
        // during the lzma_index_append() call after the indexes have been
        // concatenated.
        assert_eq!(
            lzma_index_append(
                idx,
                ptr::null(),
                UNPADDED_SIZE_MAX - ((4 * STREAM_HEADER) + 24),
                1
            ),
            LZMA_OK
        );

        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        assert_eq!(lzma_index_cat(second, idx, ptr::null()), LZMA_OK);

        assert_eq!(
            lzma_index_append(second, ptr::null(), UNPADDED_SIZE_MAX, 1),
            LZMA_DATA_ERROR
        );

        lzma_index_end(second, ptr::null());

        // Test uncompressed size growing too large.
        // Should result in LZMA_DATA_ERROR.
        idx = lzma_index_init(ptr::null());

        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, LZMA_VLI_MAX),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, 1),
            LZMA_DATA_ERROR
        );

        lzma_index_end(idx, ptr::null());

        // Currently not testing for error case when the size of the Index
        // grows too large to be stored. This was not practical to test for
        // since too many Blocks needed to be created to cause this.
    }
}

#[test]
fn test_lzma_index_stream_flags() {
    unsafe {
        // Only trivial tests done here testing for basic functionality.
        // More in-depth testing for this function will be done in
        // test_lzma_index_checks.

        // Testing for NULL inputs
        assert_eq!(
            lzma_index_stream_flags(ptr::null_mut(), ptr::null()),
            LZMA_PROG_ERROR
        );

        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        assert_eq!(lzma_index_stream_flags(idx, ptr::null()), LZMA_PROG_ERROR);

        let stream_flags = stream_flags_with_check(LZMA_CHECK_CRC32);

        assert_eq!(lzma_index_stream_flags(idx, &stream_flags), LZMA_OK);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_checks() {
    unsafe {
        // Tests should still pass, even if some of the check types
        // are disabled.
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        let mut stream_flags = stream_flags_with_check(LZMA_CHECK_NONE);

        // First set the check type to None
        assert_eq!(lzma_index_stream_flags(idx, &stream_flags), LZMA_OK);
        assert_eq!(lzma_index_checks(idx), LZMA_INDEX_CHECK_MASK_NONE);

        // Set the check type to CRC32 and repeat
        stream_flags.check = LZMA_CHECK_CRC32;
        assert_eq!(lzma_index_stream_flags(idx, &stream_flags), LZMA_OK);
        assert_eq!(lzma_index_checks(idx), LZMA_INDEX_CHECK_MASK_CRC32);

        // Set the check type to CRC64 and repeat
        stream_flags.check = LZMA_CHECK_CRC64;
        assert_eq!(lzma_index_stream_flags(idx, &stream_flags), LZMA_OK);
        assert_eq!(lzma_index_checks(idx), LZMA_INDEX_CHECK_MASK_CRC64);

        // Set the check type to SHA256 and repeat
        stream_flags.check = LZMA_CHECK_SHA256;
        assert_eq!(lzma_index_stream_flags(idx, &stream_flags), LZMA_OK);
        assert_eq!(lzma_index_checks(idx), LZMA_INDEX_CHECK_MASK_SHA256);

        // Create second lzma_index and cat to first
        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        // Set the check type to CRC32 for the second lzma_index
        stream_flags.check = LZMA_CHECK_CRC32;
        assert_eq!(lzma_index_stream_flags(second, &stream_flags), LZMA_OK);

        assert_eq!(lzma_index_checks(second), LZMA_INDEX_CHECK_MASK_CRC32);

        assert_eq!(lzma_index_cat(idx, second, ptr::null()), LZMA_OK);

        // Index should now have both CRC32 and SHA256
        assert_eq!(
            lzma_index_checks(idx),
            LZMA_INDEX_CHECK_MASK_CRC32 | LZMA_INDEX_CHECK_MASK_SHA256
        );

        // Change the check type of the second Stream to SHA256
        stream_flags.check = LZMA_CHECK_SHA256;
        assert_eq!(lzma_index_stream_flags(idx, &stream_flags), LZMA_OK);

        // Index should now have only SHA256
        assert_eq!(lzma_index_checks(idx), LZMA_INDEX_CHECK_MASK_SHA256);

        // Test with a third Stream
        let third = lzma_index_init(ptr::null());
        assert!(!third.is_null());

        stream_flags.check = LZMA_CHECK_CRC64;
        assert_eq!(lzma_index_stream_flags(third, &stream_flags), LZMA_OK);

        assert_eq!(lzma_index_checks(third), LZMA_INDEX_CHECK_MASK_CRC64);

        assert_eq!(lzma_index_cat(idx, third, ptr::null()), LZMA_OK);

        // Index should now have CRC64 and SHA256
        assert_eq!(
            lzma_index_checks(idx),
            LZMA_INDEX_CHECK_MASK_CRC64 | LZMA_INDEX_CHECK_MASK_SHA256
        );

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_stream_padding() {
    unsafe {
        // Test NULL lzma_index
        assert_eq!(
            lzma_index_stream_padding(ptr::null_mut(), 0),
            LZMA_PROG_ERROR
        );

        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        // Test Stream Padding not a multiple of 4
        assert_eq!(lzma_index_stream_padding(idx, 3), LZMA_PROG_ERROR);

        // Test Stream Padding too large
        assert_eq!(
            lzma_index_stream_padding(idx, LZMA_VLI_MAX - 3),
            LZMA_DATA_ERROR
        );

        // Test Stream Padding valid
        assert_eq!(lzma_index_stream_padding(idx, 0x1000), LZMA_OK);
        assert_eq!(lzma_index_stream_padding(idx, 4), LZMA_OK);
        assert_eq!(lzma_index_stream_padding(idx, 0), LZMA_OK);

        // Test Stream Padding causing the file size to grow too large
        assert_eq!(
            lzma_index_append(idx, ptr::null(), LZMA_VLI_MAX - 0x1000, 1),
            LZMA_OK
        );
        assert_eq!(lzma_index_stream_padding(idx, 0x1000), LZMA_DATA_ERROR);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_stream_count() {
    unsafe {
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        assert_eq!(lzma_index_stream_count(idx), 1);

        // Appending Blocks should not change the Stream count value
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );

        assert_eq!(lzma_index_stream_count(idx), 1);

        // Test with multiple Streams
        for i in 0..100u64 {
            let idx_cat = lzma_index_init(ptr::null());
            assert!(!idx_cat.is_null());
            assert_eq!(lzma_index_cat(idx, idx_cat, ptr::null()), LZMA_OK);
            assert_eq!(lzma_index_stream_count(idx), i + 2);
        }

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_block_count() {
    unsafe {
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        assert_eq!(lzma_index_block_count(idx), 0);

        let iterations: u64 = 0x1000;
        for i in 0..iterations {
            assert_eq!(
                lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, 1),
                LZMA_OK
            );
            assert_eq!(lzma_index_block_count(idx), i + 1);
        }

        // Create new lzma_index with a few Blocks
        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        assert_eq!(
            lzma_index_append(second, ptr::null(), UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(second, ptr::null(), UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(second, ptr::null(), UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );

        assert_eq!(lzma_index_block_count(second), 3);

        // Concatenate the lzma_indexes together and the result should have
        // the sum of the two individual counts.
        assert_eq!(lzma_index_cat(idx, second, ptr::null()), LZMA_OK);
        assert_eq!(lzma_index_block_count(idx), iterations + 3);

        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );

        assert_eq!(lzma_index_block_count(idx), iterations + 4);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_size() {
    unsafe {
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        // Base size should be:
        // 1 byte Index Indicator
        // 1 byte Number of Records
        // 0 bytes Records
        // 2 bytes Index Padding
        // 4 bytes CRC32
        // Total: 8 bytes
        assert_eq!(lzma_index_size(idx), 8);

        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );

        // New size should be:
        // 1 byte Index Indicator
        // 1 byte Number of Records
        // 2 bytes Records
        // 0 bytes Index Padding
        // 4 bytes CRC32
        // Total: 8 bytes
        assert_eq!(lzma_index_size(idx), 8);

        assert_eq!(
            lzma_index_append(idx, ptr::null(), LZMA_VLI_MAX / 4, LZMA_VLI_MAX / 4),
            LZMA_OK
        );

        // New size should be:
        // 1 byte Index Indicator
        // 1 byte Number of Records
        // 20 bytes Records
        // 2 bytes Index Padding
        // 4 bytes CRC32
        // Total: 28 bytes
        assert_eq!(lzma_index_size(idx), 28);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_stream_size() {
    unsafe {
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        // Stream size calculated by:
        // Size of Stream Header (12 bytes)
        // Size of all Blocks
        // Size of the Index
        // Size of the Stream Footer (12 bytes)

        // First test with empty Index
        // Stream size should be:
        // Size of Stream Header - 12 bytes
        // Size of all Blocks - 0 bytes
        // Size of Index - 8 bytes
        // Size of Stream Footer - 12 bytes
        // Total: 32 bytes
        assert_eq!(lzma_index_stream_size(idx), 32);

        // Next, append a few Blocks and retest
        assert_eq!(lzma_index_append(idx, ptr::null(), 1000, 1), LZMA_OK);
        assert_eq!(lzma_index_append(idx, ptr::null(), 999, 1), LZMA_OK);
        assert_eq!(lzma_index_append(idx, ptr::null(), 997, 1), LZMA_OK);

        // Stream size should be:
        // Size of Stream Header - 12 bytes
        // Size of all Blocks - 3000 bytes [*]
        // Size of Index - 16 bytes
        // Size of Stream Footer - 12 bytes
        // Total: 3040 bytes
        //
        // [*] Block size is a multiple of 4 bytes so 999 and 997 get
        //     rounded up to 1000 bytes.
        assert_eq!(lzma_index_stream_size(idx), 3040);

        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        assert_eq!(lzma_index_stream_size(second), 32);
        assert_eq!(lzma_index_append(second, ptr::null(), 1000, 1), LZMA_OK);

        // Stream size should be:
        // Size of Stream Header - 12 bytes
        // Size of all Blocks - 1000 bytes
        // Size of Index - 12 bytes
        // Size of Stream Footer - 12 bytes
        // Total: 1036 bytes
        assert_eq!(lzma_index_stream_size(second), 1036);

        assert_eq!(lzma_index_cat(idx, second, ptr::null()), LZMA_OK);

        // Stream size should be:
        // Size of Stream Header - 12 bytes
        // Size of all Blocks - 4000 bytes
        // Size of Index - 20 bytes
        // Size of Stream Footer - 12 bytes
        // Total: 4044 bytes
        assert_eq!(lzma_index_stream_size(idx), 4044);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_total_size() {
    unsafe {
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        // First test empty lzma_index.
        // Result should be 0 since no Blocks have been added.
        assert_eq!(lzma_index_total_size(idx), 0);

        // Add a few Blocks and retest after each append
        assert_eq!(lzma_index_append(idx, ptr::null(), 1000, 1), LZMA_OK);
        assert_eq!(lzma_index_total_size(idx), 1000);

        assert_eq!(lzma_index_append(idx, ptr::null(), 999, 1), LZMA_OK);
        assert_eq!(lzma_index_total_size(idx), 2000);

        assert_eq!(lzma_index_append(idx, ptr::null(), 997, 1), LZMA_OK);
        assert_eq!(lzma_index_total_size(idx), 3000);

        // Create second lzma_index and append Blocks to it.
        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        assert_eq!(lzma_index_total_size(second), 0);

        assert_eq!(lzma_index_append(second, ptr::null(), 100, 1), LZMA_OK);
        assert_eq!(lzma_index_total_size(second), 100);

        assert_eq!(lzma_index_append(second, ptr::null(), 100, 1), LZMA_OK);
        assert_eq!(lzma_index_total_size(second), 200);

        // Concatenate the Streams together
        assert_eq!(lzma_index_cat(idx, second, ptr::null()), LZMA_OK);

        // The resulting total size should be the size of all Blocks
        // from both Streams
        assert_eq!(lzma_index_total_size(idx), 3200);

        // Test sizes that aren't multiples of four bytes
        assert_eq!(lzma_index_append(idx, ptr::null(), 11, 1), LZMA_OK);
        assert_eq!(lzma_index_total_size(idx), 3212);

        assert_eq!(lzma_index_append(idx, ptr::null(), 11, 1), LZMA_OK);
        assert_eq!(lzma_index_total_size(idx), 3224);

        assert_eq!(lzma_index_append(idx, ptr::null(), 9, 1), LZMA_OK);
        assert_eq!(lzma_index_total_size(idx), 3236);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_file_size() {
    unsafe {
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        // Should be the same as test_lzma_index_stream_size with
        // only one Stream and no Stream Padding.
        assert_eq!(lzma_index_file_size(idx), 32);

        assert_eq!(lzma_index_append(idx, ptr::null(), 1000, 1), LZMA_OK);
        assert_eq!(lzma_index_append(idx, ptr::null(), 999, 1), LZMA_OK);
        assert_eq!(lzma_index_append(idx, ptr::null(), 997, 1), LZMA_OK);

        assert_eq!(lzma_index_file_size(idx), 3040);

        // Next add Stream Padding
        assert_eq!(lzma_index_stream_padding(idx, 1000), LZMA_OK);

        assert_eq!(lzma_index_file_size(idx), 4040);

        // Create second lzma_index.
        // Very similar to test_lzma_index_stream_size, but
        // the values should include the headers of the second Stream.
        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        assert_eq!(lzma_index_append(second, ptr::null(), 1000, 1), LZMA_OK);
        assert_eq!(lzma_index_stream_size(second), 1036);

        assert_eq!(lzma_index_cat(idx, second, ptr::null()), LZMA_OK);

        // .xz file size should be:
        // Size of 2 Stream Headers - 12 * 2 bytes
        // Size of all Blocks - 3000 + 1000 bytes
        // Size of 2 Indexes - 16 + 12 bytes
        // Size of Stream Padding - 1000 bytes
        // Size of 2 Stream Footers - 12 * 2 bytes
        // Total: 5076 bytes
        assert_eq!(lzma_index_file_size(idx), 5076);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_uncompressed_size() {
    unsafe {
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        // Empty lzma_index should have 0 uncompressed .xz file size.
        assert_eq!(lzma_index_uncompressed_size(idx), 0);

        // Append a few small Blocks
        assert_eq!(lzma_index_append(idx, ptr::null(), 1000, 1), LZMA_OK);
        assert_eq!(lzma_index_append(idx, ptr::null(), 1000, 10), LZMA_OK);
        assert_eq!(lzma_index_append(idx, ptr::null(), 1000, 100), LZMA_OK);

        assert_eq!(lzma_index_uncompressed_size(idx), 111);

        // Create another lzma_index
        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        // Append a few small Blocks
        assert_eq!(lzma_index_append(second, ptr::null(), 1000, 2), LZMA_OK);
        assert_eq!(lzma_index_append(second, ptr::null(), 1000, 20), LZMA_OK);
        assert_eq!(lzma_index_append(second, ptr::null(), 1000, 200), LZMA_OK);

        assert_eq!(lzma_index_uncompressed_size(second), 222);

        // Concatenate second lzma_index to first
        assert_eq!(lzma_index_cat(idx, second, ptr::null()), LZMA_OK);

        // New uncompressed .xz file size should be the sum of the two Streams
        assert_eq!(lzma_index_uncompressed_size(idx), 333);

        // Append one more Block to the lzma_index and ensure that
        // it is properly updated
        assert_eq!(lzma_index_append(idx, ptr::null(), 1000, 111), LZMA_OK);
        assert_eq!(lzma_index_uncompressed_size(idx), 444);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_iter_init() {
    unsafe {
        // Testing basic init functionality.
        // The init function should call rewind on the iterator.
        let first = lzma_index_init(ptr::null());
        assert!(!first.is_null());

        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        let third = lzma_index_init(ptr::null());
        assert!(!third.is_null());

        assert_eq!(lzma_index_cat(first, second, ptr::null()), LZMA_OK);
        assert_eq!(lzma_index_cat(first, third, ptr::null()), LZMA_OK);

        let mut iter = new_iter(first);

        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);
        assert_eq!(iter.stream.number, 1);
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);
        assert_eq!(iter.stream.number, 2);

        lzma_index_iter_init(&mut iter, first);

        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);
        assert_eq!(iter.stream.number, 3);

        lzma_index_end(first, ptr::null());
    }
}

#[test]
fn test_lzma_index_iter_rewind() {
    unsafe {
        let first = lzma_index_init(ptr::null());
        assert!(!first.is_null());

        let mut iter = new_iter(first);

        // Append 3 Blocks and iterate over each. This is to test
        // the LZMA_INDEX_ITER_BLOCK mode.
        for i in 0..3u64 {
            assert_eq!(
                lzma_index_append(first, ptr::null(), UNPADDED_SIZE_MIN, 1),
                LZMA_OK
            );
            assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_BLOCK), 0);
            assert_eq!(iter.block.number_in_file, i + 1);
            assert_eq!(iter.block.number_in_stream, i + 1);
        }

        // Rewind back to the beginning and iterate over the Blocks again
        lzma_index_iter_rewind(&mut iter);

        // Should be able to re-iterate over the Blocks again.
        for i in 0..3u64 {
            assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_BLOCK), 0);
            assert_eq!(iter.block.number_in_file, i + 1);
            assert_eq!(iter.block.number_in_stream, i + 1);
        }

        // Next concatenate two more lzma_indexes, iterate over them,
        // rewind, and iterate over them again. This is to test
        // the LZMA_INDEX_ITER_STREAM mode.
        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        let third = lzma_index_init(ptr::null());
        assert!(!third.is_null());

        assert_eq!(lzma_index_cat(first, second, ptr::null()), LZMA_OK);
        assert_eq!(lzma_index_cat(first, third, ptr::null()), LZMA_OK);

        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);

        assert_eq!(iter.stream.number, 3);

        lzma_index_iter_rewind(&mut iter);

        for i in 0..3u64 {
            assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);
            assert_eq!(iter.stream.number, i + 1);
        }

        lzma_index_end(first, ptr::null());
    }
}

#[test]
fn test_lzma_index_iter_next() {
    unsafe {
        let first = lzma_index_init(ptr::null());
        assert!(!first.is_null());

        let mut iter = new_iter(first);

        // First test bad mode values
        for i in (LZMA_INDEX_ITER_NONEMPTY_BLOCK + 1)..100 {
            assert_ne!(lzma_index_iter_next(&mut iter, i), 0);
        }

        // Test iterating over Blocks
        assert_eq!(
            lzma_index_append(first, ptr::null(), UNPADDED_SIZE_MIN, 1),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(first, ptr::null(), UNPADDED_SIZE_MIN * 2, 10),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(first, ptr::null(), UNPADDED_SIZE_MIN * 3, 100),
            LZMA_OK
        );

        // For Blocks, need to verify:
        // - number_in_file (overall Block number)
        // - compressed_file_offset
        // - uncompressed_file_offset
        // - number_in_stream (Block number relative to current Stream)
        // - compressed_stream_offset
        // - uncompressed_stream_offset
        // - uncompressed_size
        // - unpadded_size
        // - total_size

        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_BLOCK), 0);

        // Verify Block data stored correctly
        assert_eq!(iter.block.number_in_file, 1);

        // Should start right after the Stream Header
        assert_eq!(iter.block.compressed_file_offset, STREAM_HEADER);
        assert_eq!(iter.block.uncompressed_file_offset, 0);
        assert_eq!(iter.block.number_in_stream, 1);
        assert_eq!(iter.block.compressed_stream_offset, STREAM_HEADER);
        assert_eq!(iter.block.uncompressed_stream_offset, 0);
        assert_eq!(iter.block.unpadded_size, UNPADDED_SIZE_MIN);
        assert_eq!(iter.block.total_size, vli_ceil4(UNPADDED_SIZE_MIN));

        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_BLOCK), 0);

        // Verify Block data stored correctly
        assert_eq!(iter.block.number_in_file, 2);
        assert_eq!(
            iter.block.compressed_file_offset,
            STREAM_HEADER + vli_ceil4(UNPADDED_SIZE_MIN)
        );
        assert_eq!(iter.block.uncompressed_file_offset, 1);
        assert_eq!(iter.block.number_in_stream, 2);
        assert_eq!(
            iter.block.compressed_stream_offset,
            STREAM_HEADER + vli_ceil4(UNPADDED_SIZE_MIN)
        );
        assert_eq!(iter.block.uncompressed_stream_offset, 1);
        assert_eq!(iter.block.unpadded_size, UNPADDED_SIZE_MIN * 2);
        assert_eq!(iter.block.total_size, vli_ceil4(UNPADDED_SIZE_MIN * 2));

        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_BLOCK), 0);

        // Verify Block data stored correctly
        assert_eq!(iter.block.number_in_file, 3);
        assert_eq!(
            iter.block.compressed_file_offset,
            STREAM_HEADER + vli_ceil4(UNPADDED_SIZE_MIN) + vli_ceil4(UNPADDED_SIZE_MIN * 2)
        );
        assert_eq!(iter.block.uncompressed_file_offset, 11);
        assert_eq!(iter.block.number_in_stream, 3);
        assert_eq!(
            iter.block.compressed_stream_offset,
            STREAM_HEADER + vli_ceil4(UNPADDED_SIZE_MIN) + vli_ceil4(UNPADDED_SIZE_MIN * 2)
        );
        assert_eq!(iter.block.uncompressed_stream_offset, 11);
        assert_eq!(iter.block.unpadded_size, UNPADDED_SIZE_MIN * 3);
        assert_eq!(iter.block.total_size, vli_ceil4(UNPADDED_SIZE_MIN * 3));

        // Only three Blocks were added, so this should return true
        assert_ne!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_BLOCK), 0);

        let second_stream_compressed_start = STREAM_HEADER * 2
            + vli_ceil4(UNPADDED_SIZE_MIN)
            + vli_ceil4(UNPADDED_SIZE_MIN * 2)
            + vli_ceil4(UNPADDED_SIZE_MIN * 3)
            + lzma_index_size(first);
        let second_stream_uncompressed_start: lzma_vli = 1 + 10 + 100;

        // Test iterating over Streams.
        // The second Stream will have 0 Blocks
        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        // Set Stream Flags for Stream 2
        let flags = stream_flags_with_check(LZMA_CHECK_CRC32);

        assert_eq!(lzma_index_stream_flags(second, &flags), LZMA_OK);

        // The Second stream will have 8 bytes of Stream Padding
        assert_eq!(lzma_index_stream_padding(second, 8), LZMA_OK);

        let second_stream_index_size = lzma_index_size(second);

        // The third Stream will have 2 Blocks
        let third = lzma_index_init(ptr::null());
        assert!(!third.is_null());

        assert_eq!(lzma_index_append(third, ptr::null(), 32, 20), LZMA_OK);
        assert_eq!(lzma_index_append(third, ptr::null(), 64, 40), LZMA_OK);

        let third_stream_index_size = lzma_index_size(third);

        assert_eq!(lzma_index_cat(first, second, ptr::null()), LZMA_OK);
        assert_eq!(lzma_index_cat(first, third, ptr::null()), LZMA_OK);

        // For Streams, need to verify:
        // - flags (Stream Flags)
        // - number (Stream count)
        // - block_count
        // - compressed_offset
        // - uncompressed_offset
        // - compressed_size
        // - uncompressed_size
        // - padding (Stream Padding)
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);

        // Verify Stream
        assert_eq!((*iter.stream.flags).backward_size, LZMA_BACKWARD_SIZE_MIN);
        assert_eq!((*iter.stream.flags).check, LZMA_CHECK_CRC32);
        assert_eq!(iter.stream.number, 2);
        assert_eq!(iter.stream.block_count, 0);
        assert_eq!(
            iter.stream.compressed_offset,
            second_stream_compressed_start
        );
        assert_eq!(
            iter.stream.uncompressed_offset,
            second_stream_uncompressed_start
        );
        assert_eq!(
            iter.stream.compressed_size,
            STREAM_HEADER * 2 + second_stream_index_size
        );
        assert_eq!(iter.stream.uncompressed_size, 0);
        assert_eq!(iter.stream.padding, 8);

        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);

        // Verify Stream
        let third_stream_compressed_start = second_stream_compressed_start
            + STREAM_HEADER * 2
            + 8 // Stream padding
            + second_stream_index_size;
        let third_stream_uncompressed_start = second_stream_uncompressed_start;

        assert_eq!(iter.stream.number, 3);
        assert_eq!(iter.stream.block_count, 2);
        assert_eq!(iter.stream.compressed_offset, third_stream_compressed_start);
        assert_eq!(
            iter.stream.uncompressed_offset,
            third_stream_uncompressed_start
        );
        assert_eq!(
            iter.stream.compressed_size,
            STREAM_HEADER * 2
                + 96 // Total compressed size
                + third_stream_index_size
        );
        assert_eq!(iter.stream.uncompressed_size, 60);
        assert_eq!(iter.stream.padding, 0);

        assert_ne!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_STREAM), 0);

        // Even after a failing call to next with ITER_STREAM mode,
        // should still be able to iterate over the 2 Blocks in
        // Stream 3.
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_BLOCK), 0);

        // Verify both Blocks

        // Next call to iterate Block should return true because the
        // first Block can already be read from the earlier *successful*
        // LZMA_INDEX_ITER_STREAM call; the previous failed call doesn't
        // modify the iterator.
        assert_ne!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_BLOCK), 0);

        // Rewind to test LZMA_INDEX_ITER_ANY
        lzma_index_iter_rewind(&mut iter);

        // Iterate past the first three Blocks
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_ANY), 0);
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_ANY), 0);
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_ANY), 0);

        // Iterate past the next Stream
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_ANY), 0);

        // Iterate past the next Stream
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_ANY), 0);
        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_ANY), 0);

        // Last call should fail
        assert_ne!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_ANY), 0);

        // Rewind to test LZMA_INDEX_ITER_NONEMPTY_BLOCK
        lzma_index_iter_rewind(&mut iter);

        // Iterate past the first three Blocks
        assert_eq!(
            lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_NONEMPTY_BLOCK),
            0
        );
        assert_eq!(
            lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_NONEMPTY_BLOCK),
            0
        );
        assert_eq!(
            lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_NONEMPTY_BLOCK),
            0
        );

        // Skip past the next Stream which has no Blocks.
        // We will get to the first Block of the third Stream.
        assert_eq!(
            lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_NONEMPTY_BLOCK),
            0
        );

        // Iterate past the second (the last) Block in the third Stream
        assert_eq!(
            lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_NONEMPTY_BLOCK),
            0
        );

        // Last call should fail since there is nothing left to iterate over.
        assert_ne!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_ANY), 0);

        lzma_index_end(first, ptr::null());
    }
}

#[test]
fn test_lzma_index_iter_locate() {
    unsafe {
        let mut idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        let mut iter = new_iter(idx);

        // Cannot locate anything from an empty Index.
        assert_ne!(lzma_index_iter_locate(&mut iter, 0), 0);
        assert_ne!(lzma_index_iter_locate(&mut iter, 555), 0);

        // One empty Record: nothing is found since there's no uncompressed
        // data.
        assert_eq!(lzma_index_append(idx, ptr::null(), 16, 0), LZMA_OK);
        assert_ne!(lzma_index_iter_locate(&mut iter, 0), 0);

        // Non-empty Record and we can find something.
        assert_eq!(lzma_index_append(idx, ptr::null(), 32, 5), LZMA_OK);
        assert_eq!(lzma_index_iter_locate(&mut iter, 0), 0);
        assert_eq!(iter.block.total_size, 32);
        assert_eq!(iter.block.uncompressed_size, 5);
        assert_eq!(iter.block.compressed_file_offset, STREAM_HEADER + 16);
        assert_eq!(iter.block.uncompressed_file_offset, 0);

        // Still cannot find anything past the end.
        assert_ne!(lzma_index_iter_locate(&mut iter, 5), 0);

        // Add the third Record.
        assert_eq!(lzma_index_append(idx, ptr::null(), 40, 11), LZMA_OK);

        assert_eq!(lzma_index_iter_locate(&mut iter, 0), 0);
        assert_eq!(iter.block.total_size, 32);
        assert_eq!(iter.block.uncompressed_size, 5);
        assert_eq!(iter.block.compressed_file_offset, STREAM_HEADER + 16);
        assert_eq!(iter.block.uncompressed_file_offset, 0);

        assert_eq!(lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_BLOCK), 0);
        assert_eq!(iter.block.total_size, 40);
        assert_eq!(iter.block.uncompressed_size, 11);
        assert_eq!(iter.block.compressed_file_offset, STREAM_HEADER + 16 + 32);
        assert_eq!(iter.block.uncompressed_file_offset, 5);

        assert_eq!(lzma_index_iter_locate(&mut iter, 2), 0);
        assert_eq!(iter.block.total_size, 32);
        assert_eq!(iter.block.uncompressed_size, 5);
        assert_eq!(iter.block.compressed_file_offset, STREAM_HEADER + 16);
        assert_eq!(iter.block.uncompressed_file_offset, 0);

        assert_eq!(lzma_index_iter_locate(&mut iter, 5), 0);
        assert_eq!(iter.block.total_size, 40);
        assert_eq!(iter.block.uncompressed_size, 11);
        assert_eq!(iter.block.compressed_file_offset, STREAM_HEADER + 16 + 32);
        assert_eq!(iter.block.uncompressed_file_offset, 5);

        assert_eq!(lzma_index_iter_locate(&mut iter, 5 + 11 - 1), 0);
        assert_eq!(iter.block.total_size, 40);
        assert_eq!(iter.block.uncompressed_size, 11);
        assert_eq!(iter.block.compressed_file_offset, STREAM_HEADER + 16 + 32);
        assert_eq!(iter.block.uncompressed_file_offset, 5);

        assert_ne!(lzma_index_iter_locate(&mut iter, 5 + 11), 0);
        assert_ne!(lzma_index_iter_locate(&mut iter, 5 + 15), 0);

        // Large Index
        lzma_index_end(idx, ptr::null());
        idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());
        lzma_index_iter_init(&mut iter, idx);

        let mut n: u32 = 4;
        while n <= 4 * 5555 {
            assert_eq!(
                lzma_index_append(idx, ptr::null(), (n + 7) as lzma_vli, n as lzma_vli),
                LZMA_OK
            );
            n += 4;
        }

        assert_eq!(lzma_index_block_count(idx), 5555);

        // First Record
        assert_eq!(lzma_index_iter_locate(&mut iter, 0), 0);
        assert_eq!(iter.block.unpadded_size, 4 + 7);
        assert_eq!(iter.block.total_size, 4 + 8);
        assert_eq!(iter.block.uncompressed_size, 4);
        assert_eq!(iter.block.compressed_file_offset, STREAM_HEADER);
        assert_eq!(iter.block.uncompressed_file_offset, 0);

        assert_eq!(lzma_index_iter_locate(&mut iter, 3), 0);
        assert_eq!(iter.block.unpadded_size, 4 + 7);
        assert_eq!(iter.block.total_size, 4 + 8);
        assert_eq!(iter.block.uncompressed_size, 4);
        assert_eq!(iter.block.compressed_file_offset, STREAM_HEADER);
        assert_eq!(iter.block.uncompressed_file_offset, 0);

        // Second Record
        assert_eq!(lzma_index_iter_locate(&mut iter, 4), 0);
        assert_eq!(iter.block.unpadded_size, 2 * 4 + 7);
        assert_eq!(iter.block.total_size, 2 * 4 + 8);
        assert_eq!(iter.block.uncompressed_size, 2 * 4);
        assert_eq!(iter.block.compressed_file_offset, STREAM_HEADER + 4 + 8);
        assert_eq!(iter.block.uncompressed_file_offset, 4);

        // Last Record
        assert_eq!(
            lzma_index_iter_locate(&mut iter, lzma_index_uncompressed_size(idx) - 1),
            0
        );
        assert_eq!(iter.block.unpadded_size, 4 * 5555 + 7);
        assert_eq!(iter.block.total_size, 4 * 5555 + 8);
        assert_eq!(iter.block.uncompressed_size, 4 * 5555);
        assert_eq!(
            iter.block.compressed_file_offset,
            lzma_index_total_size(idx) + STREAM_HEADER - 4 * 5555 - 8
        );
        assert_eq!(
            iter.block.uncompressed_file_offset,
            lzma_index_uncompressed_size(idx) - 4 * 5555
        );

        // Allocation chunk boundaries. See INDEX_GROUP_SIZE in
        // common/index.rs.
        let group_multiple: u32 = 256 * 4;
        let radius: u32 = 8;
        let start: u32 = group_multiple - radius;
        let mut ubase: lzma_vli = 0;
        let mut tbase: lzma_vli = 0;
        let mut n: u32 = 1;
        while n < start {
            ubase += n as lzma_vli * 4;
            tbase += n as lzma_vli * 4 + 8;
            n += 1;
        }

        while n < start + 2 * radius {
            assert_eq!(
                lzma_index_iter_locate(&mut iter, ubase + n as lzma_vli * 4),
                0
            );

            assert_eq!(
                iter.block.compressed_file_offset,
                tbase + n as lzma_vli * 4 + 8 + STREAM_HEADER
            );
            assert_eq!(
                iter.block.uncompressed_file_offset,
                ubase + n as lzma_vli * 4
            );

            tbase += n as lzma_vli * 4 + 8;
            ubase += n as lzma_vli * 4;
            n += 1;

            assert_eq!(iter.block.total_size, n as lzma_vli * 4 + 8);
            assert_eq!(iter.block.uncompressed_size, n as lzma_vli * 4);
        }

        // Do it also backwards.
        while n > start {
            assert_eq!(
                lzma_index_iter_locate(&mut iter, ubase + (n - 1) as lzma_vli * 4),
                0
            );

            assert_eq!(iter.block.total_size, n as lzma_vli * 4 + 8);
            assert_eq!(iter.block.uncompressed_size, n as lzma_vli * 4);

            n -= 1;
            tbase -= n as lzma_vli * 4 + 8;
            ubase -= n as lzma_vli * 4;

            assert_eq!(
                iter.block.compressed_file_offset,
                tbase + n as lzma_vli * 4 + 8 + STREAM_HEADER
            );
            assert_eq!(
                iter.block.uncompressed_file_offset,
                ubase + n as lzma_vli * 4
            );
        }

        // Test locating in concatenated Index.
        lzma_index_end(idx, ptr::null());
        idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());
        lzma_index_iter_init(&mut iter, idx);
        for _ in 0..group_multiple {
            assert_eq!(lzma_index_append(idx, ptr::null(), 8, 0), LZMA_OK);
        }

        assert_eq!(lzma_index_append(idx, ptr::null(), 16, 1), LZMA_OK);
        assert_eq!(lzma_index_iter_locate(&mut iter, 0), 0);
        assert_eq!(iter.block.total_size, 16);
        assert_eq!(iter.block.uncompressed_size, 1);
        assert_eq!(
            iter.block.compressed_file_offset,
            STREAM_HEADER + group_multiple as lzma_vli * 8
        );
        assert_eq!(iter.block.uncompressed_file_offset, 0);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_cat() {
    unsafe {
        // Most complex tests for this function are done in other tests.
        // This will mostly test basic functionality.

        let mut dest = lzma_index_init(ptr::null());
        assert!(!dest.is_null());

        let mut src = lzma_index_init(ptr::null());
        assert!(!src.is_null());

        // First test NULL dest or src
        assert_eq!(
            lzma_index_cat(ptr::null_mut(), ptr::null_mut(), ptr::null()),
            LZMA_PROG_ERROR
        );
        assert_eq!(
            lzma_index_cat(dest, ptr::null_mut(), ptr::null()),
            LZMA_PROG_ERROR
        );
        assert_eq!(
            lzma_index_cat(ptr::null_mut(), src, ptr::null()),
            LZMA_PROG_ERROR
        );

        // Check for compressed size overflow
        assert_eq!(
            lzma_index_append(dest, ptr::null(), (UNPADDED_SIZE_MAX / 2) + 1, 1),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(src, ptr::null(), (UNPADDED_SIZE_MAX / 2) + 1, 1),
            LZMA_OK
        );
        assert_eq!(lzma_index_cat(dest, src, ptr::null()), LZMA_DATA_ERROR);

        lzma_index_end(src, ptr::null());
        lzma_index_end(dest, ptr::null());

        // Check for uncompressed size overflow
        dest = lzma_index_init(ptr::null());
        assert!(!dest.is_null());

        src = lzma_index_init(ptr::null());
        assert!(!src.is_null());

        assert_eq!(
            lzma_index_append(dest, ptr::null(), UNPADDED_SIZE_MIN, (LZMA_VLI_MAX / 2) + 1),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(src, ptr::null(), UNPADDED_SIZE_MIN, (LZMA_VLI_MAX / 2) + 1),
            LZMA_OK
        );
        assert_eq!(lzma_index_cat(dest, src, ptr::null()), LZMA_DATA_ERROR);

        lzma_index_end(dest, ptr::null());
        lzma_index_end(src, ptr::null());
    }
}

// Helper function for test_lzma_index_dup().
unsafe fn index_is_equal(a: *const lzma_index, b: *const lzma_index) -> bool {
    // Compare only the Stream and Block sizes and offsets.
    let mut ra = new_iter(a);
    let mut rb = new_iter(b);

    loop {
        let reta = lzma_index_iter_next(&mut ra, LZMA_INDEX_ITER_ANY) != 0;
        let retb = lzma_index_iter_next(&mut rb, LZMA_INDEX_ITER_ANY) != 0;

        // If both iterators finish at the same time, then the Indexes
        // are identical.
        if reta {
            return retb;
        }

        if ra.stream.number != rb.stream.number
            || ra.stream.block_count != rb.stream.block_count
            || ra.stream.compressed_offset != rb.stream.compressed_offset
            || ra.stream.uncompressed_offset != rb.stream.uncompressed_offset
            || ra.stream.compressed_size != rb.stream.compressed_size
            || ra.stream.uncompressed_size != rb.stream.uncompressed_size
            || ra.stream.padding != rb.stream.padding
        {
            return false;
        }

        if ra.stream.block_count == 0 {
            continue;
        }

        if ra.block.number_in_file != rb.block.number_in_file
            || ra.block.compressed_file_offset != rb.block.compressed_file_offset
            || ra.block.uncompressed_file_offset != rb.block.uncompressed_file_offset
            || ra.block.number_in_stream != rb.block.number_in_stream
            || ra.block.compressed_stream_offset != rb.block.compressed_stream_offset
            || ra.block.uncompressed_stream_offset != rb.block.uncompressed_stream_offset
            || ra.block.uncompressed_size != rb.block.uncompressed_size
            || ra.block.unpadded_size != rb.block.unpadded_size
            || ra.block.total_size != rb.block.total_size
        {
            return false;
        }
    }
}

// Allocator that succeeds for the first two allocations but fails the rest.
static MY_ALLOC_COUNT: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" fn my_alloc(opaque: *mut c_void, a: usize, b: usize) -> *mut c_void {
    assert!(usize::MAX / a >= b);

    if MY_ALLOC_COUNT.load(Ordering::Relaxed) >= 2 {
        return ptr::null_mut();
    }

    MY_ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
    xz_core::alloc::lzma_c_alloc(opaque, a, b)
}

const TEST_INDEX_DUP_ALLOC: lzma_allocator = lzma_allocator {
    alloc: Some(my_alloc),
    free: None,
    opaque: ptr::null_mut(),
};

#[test]
fn test_lzma_index_dup() {
    unsafe {
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        // Test for the bug fix 21515d79d778b8730a434f151b07202d52a04611:
        // Fix lzma_index_dup() for empty Streams.
        assert_eq!(lzma_index_stream_padding(idx, 4), LZMA_OK);
        let mut copy = lzma_index_dup(idx, ptr::null());
        assert!(!copy.is_null());
        assert!(index_is_equal(idx, copy));
        lzma_index_end(copy, ptr::null());

        // Test for the bug fix 3bf857edfef51374f6f3fffae3d817f57d3264a0:
        // Fix a memory leak in error path of lzma_index_dup().
        // Use Valgrind to see that there are no leaks.
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, 10),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN * 2, 100),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN * 3, 1000),
            LZMA_OK
        );

        assert!(lzma_index_dup(idx, &TEST_INDEX_DUP_ALLOC).is_null());

        // Test a few streams and blocks
        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        assert_eq!(lzma_index_stream_padding(second, 16), LZMA_OK);

        let third = lzma_index_init(ptr::null());
        assert!(!third.is_null());

        assert_eq!(
            lzma_index_append(third, ptr::null(), UNPADDED_SIZE_MIN * 10, 40),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(third, ptr::null(), UNPADDED_SIZE_MIN * 20, 400),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(third, ptr::null(), UNPADDED_SIZE_MIN * 30, 4000),
            LZMA_OK
        );

        assert_eq!(lzma_index_cat(idx, second, ptr::null()), LZMA_OK);
        assert_eq!(lzma_index_cat(idx, third, ptr::null()), LZMA_OK);

        copy = lzma_index_dup(idx, ptr::null());
        assert!(!copy.is_null());
        assert!(index_is_equal(idx, copy));

        lzma_index_end(copy, ptr::null());
        lzma_index_end(idx, ptr::null());
    }
}

unsafe fn verify_index_buffer(idx: *const lzma_index, buffer: &[u8]) {
    let buffer_size = buffer.len();
    let mut iter = new_iter(idx);

    let mut buffer_pos: usize = 0;

    // Verify Index Indicator
    assert_eq!(buffer[buffer_pos], 0);
    buffer_pos += 1;

    // Get Number of Records
    let mut number_of_records: lzma_vli = 0;
    let mut block_count: lzma_vli = 0;
    assert_eq!(
        lzma_vli_decode(
            &mut number_of_records,
            ptr::null_mut(),
            buffer.as_ptr(),
            &mut buffer_pos,
            buffer_size
        ),
        LZMA_OK
    );

    while lzma_index_iter_next(&mut iter, LZMA_INDEX_ITER_ANY) == 0 {
        // Verify each Record (Unpadded Size, then Uncompressed Size).
        // Verify Unpadded Size.
        let mut unpadded_size: lzma_vli = 0;
        let mut uncompressed_size: lzma_vli = 0;
        assert_eq!(
            lzma_vli_decode(
                &mut unpadded_size,
                ptr::null_mut(),
                buffer.as_ptr(),
                &mut buffer_pos,
                buffer_size
            ),
            LZMA_OK
        );
        assert_eq!(unpadded_size, iter.block.unpadded_size);

        // Verify Uncompressed Size
        assert_eq!(
            lzma_vli_decode(
                &mut uncompressed_size,
                ptr::null_mut(),
                buffer.as_ptr(),
                &mut buffer_pos,
                buffer_size
            ),
            LZMA_OK
        );
        assert_eq!(uncompressed_size, iter.block.uncompressed_size);

        block_count += 1;
    }

    // Verify Number of Records
    assert_eq!(number_of_records, block_count);

    // Verify Index Padding
    while !buffer_pos.is_multiple_of(4) {
        assert_eq!(buffer[buffer_pos], 0);
        buffer_pos += 1;
    }

    // Verify CRC32
    let crc32 = lzma_crc32(buffer.as_ptr(), buffer_pos, 0);
    assert_eq!(
        u32::from_le_bytes(buffer[buffer_pos..buffer_pos + 4].try_into().unwrap()),
        crc32
    );
}

unsafe fn get_index_size(idx: *const lzma_index) -> usize {
    let size = lzma_index_size(idx);
    assert!(size < usize::MAX as lzma_vli);
    size as usize
}

#[test]
fn test_lzma_index_encoder() {
    unsafe {
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        let mut strm = new_stream();

        // First do basic NULL checks
        assert_eq!(
            lzma_index_encoder(ptr::null_mut(), ptr::null()),
            LZMA_PROG_ERROR
        );
        assert_eq!(lzma_index_encoder(&mut strm, ptr::null()), LZMA_PROG_ERROR);
        assert_eq!(lzma_index_encoder(ptr::null_mut(), idx), LZMA_PROG_ERROR);

        // Append three small Blocks
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, 10),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN * 2, 100),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN * 3, 1000),
            LZMA_OK
        );

        // Encode this lzma_index into a buffer
        let mut buffer_size = get_index_size(idx);
        let mut buffer = vec![0u8; buffer_size];

        assert_eq!(lzma_index_encoder(&mut strm, idx), LZMA_OK);

        strm.avail_out = buffer_size;
        strm.next_out = buffer.as_mut_ptr();

        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_STREAM_END);
        assert_eq!(strm.avail_out, 0);

        lzma_end(&mut strm);

        verify_index_buffer(idx, &buffer);

        // Test with multiple Streams concatenated into 1 Index
        let second = lzma_index_init(ptr::null());
        assert!(!second.is_null());

        // Include 1 Block
        assert_eq!(
            lzma_index_append(second, ptr::null(), UNPADDED_SIZE_MIN * 4, 20),
            LZMA_OK
        );

        // Include Stream Padding
        assert_eq!(lzma_index_stream_padding(second, 16), LZMA_OK);

        assert_eq!(lzma_index_cat(idx, second, ptr::null()), LZMA_OK);
        buffer_size = get_index_size(idx);
        buffer = vec![0u8; buffer_size];
        assert_eq!(lzma_index_encoder(&mut strm, idx), LZMA_OK);

        strm.avail_out = buffer_size;
        strm.next_out = buffer.as_mut_ptr();

        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_STREAM_END);
        assert_eq!(strm.avail_out, 0);

        verify_index_buffer(idx, &buffer);

        lzma_index_end(idx, ptr::null());
        lzma_end(&mut strm);
    }
}

unsafe fn generate_index_decode_buffer() -> (*mut lzma_index, Vec<u8>) {
    let decode_test_index = lzma_index_init(ptr::null());
    assert!(!decode_test_index.is_null());

    // Add 4 Blocks
    for i in 1..5u64 {
        assert_eq!(
            lzma_index_append(decode_test_index, ptr::null(), 0x1000 * i, 0x100 * i),
            LZMA_OK
        );
    }

    let size = lzma_index_size(decode_test_index) as usize;
    let mut decode_buffer = vec![0u8; size];
    let mut decode_buffer_size: usize = 0;

    assert_eq!(
        lzma_index_buffer_encode(
            decode_test_index,
            decode_buffer.as_mut_ptr(),
            &mut decode_buffer_size,
            size
        ),
        LZMA_OK
    );
    assert!(decode_buffer_size != 0);
    decode_buffer.truncate(decode_buffer_size);

    (decode_test_index, decode_buffer)
}

unsafe fn decode_index(
    buffer: *const u8,
    size: usize,
    strm: *mut lzma_stream,
    expected_error: lzma_ret,
) {
    (*strm).avail_in = size;
    (*strm).next_in = buffer;
    assert_eq!(lzma_code(strm, LZMA_FINISH), expected_error);
}

#[test]
fn test_lzma_index_decoder() {
    unsafe {
        let (decode_test_index, decode_buffer) = generate_index_decode_buffer();
        let decode_buffer_size = decode_buffer.len();
        assert!(decode_buffer_size != 0);

        let mut strm = new_stream();

        assert_eq!(
            lzma_index_decoder(ptr::null_mut(), ptr::null_mut(), MEMLIMIT),
            LZMA_PROG_ERROR
        );
        assert_eq!(
            lzma_index_decoder(&mut strm, ptr::null_mut(), MEMLIMIT),
            LZMA_PROG_ERROR
        );

        // If the first argument (lzma_stream *strm) is NULL then
        // *idx must still become NULL since the API docs say that
        // it's done if an error occurs. This was fixed in
        // 71eed2520e2eecae89bade9dceea16e56cfa2ea0.
        let idx_allocated = lzma_index_init(ptr::null());
        let mut idx = idx_allocated;
        assert_eq!(
            lzma_index_decoder(ptr::null_mut(), &mut idx, MEMLIMIT),
            LZMA_PROG_ERROR
        );
        assert!(idx.is_null());

        lzma_index_end(idx_allocated, ptr::null());

        // Do actual decode
        assert_eq!(lzma_index_decoder(&mut strm, &mut idx, MEMLIMIT), LZMA_OK);

        decode_index(
            decode_buffer.as_ptr(),
            decode_buffer_size,
            &mut strm,
            LZMA_STREAM_END,
        );

        // Compare results with expected
        assert!(index_is_equal(decode_test_index, idx));

        lzma_index_end(idx, ptr::null());

        // Test again with too low memory limit
        assert_eq!(lzma_index_decoder(&mut strm, &mut idx, 0), LZMA_OK);

        decode_index(
            decode_buffer.as_ptr(),
            decode_buffer_size,
            &mut strm,
            LZMA_MEMLIMIT_ERROR,
        );

        let mut corrupt_buffer = decode_buffer.clone();

        assert_eq!(lzma_index_decoder(&mut strm, &mut idx, MEMLIMIT), LZMA_OK);

        // First corrupt the Index Indicator
        corrupt_buffer[0] ^= 1;
        decode_index(
            corrupt_buffer.as_ptr(),
            decode_buffer_size,
            &mut strm,
            LZMA_DATA_ERROR,
        );
        corrupt_buffer[0] ^= 1;

        // Corrupt something in the middle of Index
        corrupt_buffer[decode_buffer_size / 2] ^= 1;
        assert_eq!(lzma_index_decoder(&mut strm, &mut idx, MEMLIMIT), LZMA_OK);
        decode_index(
            corrupt_buffer.as_ptr(),
            decode_buffer_size,
            &mut strm,
            LZMA_DATA_ERROR,
        );
        corrupt_buffer[decode_buffer_size / 2] ^= 1;

        // Corrupt CRC32
        corrupt_buffer[decode_buffer_size - 1] ^= 1;
        assert_eq!(lzma_index_decoder(&mut strm, &mut idx, MEMLIMIT), LZMA_OK);
        decode_index(
            corrupt_buffer.as_ptr(),
            decode_buffer_size,
            &mut strm,
            LZMA_DATA_ERROR,
        );
        corrupt_buffer[decode_buffer_size - 1] ^= 1;

        // Corrupt Index Padding by setting it to non-zero
        corrupt_buffer[decode_buffer_size - 5] ^= 1;
        assert_eq!(lzma_index_decoder(&mut strm, &mut idx, MEMLIMIT), LZMA_OK);
        decode_index(
            corrupt_buffer.as_ptr(),
            decode_buffer_size,
            &mut strm,
            LZMA_DATA_ERROR,
        );
        corrupt_buffer[decode_buffer_size - 1] ^= 1;

        lzma_end(&mut strm);
        lzma_index_end(decode_test_index, ptr::null());
    }
}

#[test]
fn test_lzma_index_buffer_encode() {
    unsafe {
        // These are simpler test than in test_lzma_index_encoder()
        // because lzma_index_buffer_encode() is mostly a wrapper
        // around lzma_index_encoder() anyway.
        let idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN, 10),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN * 2, 100),
            LZMA_OK
        );
        assert_eq!(
            lzma_index_append(idx, ptr::null(), UNPADDED_SIZE_MIN * 3, 1000),
            LZMA_OK
        );

        let buffer_size = get_index_size(idx);
        let mut buffer = vec![0u8; buffer_size];
        let mut out_pos: usize = 1;

        // First test bad arguments
        assert_eq!(
            lzma_index_buffer_encode(ptr::null(), ptr::null_mut(), ptr::null_mut(), 0),
            LZMA_PROG_ERROR
        );
        assert_eq!(
            lzma_index_buffer_encode(idx, ptr::null_mut(), ptr::null_mut(), 0),
            LZMA_PROG_ERROR
        );
        assert_eq!(
            lzma_index_buffer_encode(idx, buffer.as_mut_ptr(), ptr::null_mut(), 0),
            LZMA_PROG_ERROR
        );
        assert_eq!(
            lzma_index_buffer_encode(idx, buffer.as_mut_ptr(), &mut out_pos, 0),
            LZMA_PROG_ERROR
        );
        out_pos = 0;
        assert_eq!(
            lzma_index_buffer_encode(idx, buffer.as_mut_ptr(), &mut out_pos, 0),
            LZMA_BUF_ERROR
        );
        assert_eq!(out_pos, 0);
        assert_eq!(
            lzma_index_buffer_encode(idx, buffer.as_mut_ptr(), &mut out_pos, 1),
            LZMA_BUF_ERROR
        );

        // Do encoding
        assert_eq!(
            lzma_index_buffer_encode(idx, buffer.as_mut_ptr(), &mut out_pos, buffer_size),
            LZMA_OK
        );
        assert_eq!(out_pos, buffer_size);

        // Validate results
        verify_index_buffer(idx, &buffer);

        lzma_index_end(idx, ptr::null());
    }
}

#[test]
fn test_lzma_index_buffer_decode() {
    unsafe {
        let (decode_test_index, decode_buffer) = generate_index_decode_buffer();
        let decode_buffer_size = decode_buffer.len();
        assert!(decode_buffer_size != 0);

        // Simple test since test_lzma_index_decoder() covers most of the
        // lzma_index_buffer_decode() code anyway.

        // First test NULL checks
        assert_eq!(
            lzma_index_buffer_decode(
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                0
            ),
            LZMA_PROG_ERROR
        );

        let mut memlimit: u64 = MEMLIMIT;
        let mut in_pos: usize = 0;
        let idx_allocated = lzma_index_init(ptr::null());
        let mut idx = idx_allocated;

        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                0
            ),
            LZMA_PROG_ERROR
        );
        assert!(idx.is_null());

        idx = idx_allocated;
        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                &mut memlimit,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                0
            ),
            LZMA_PROG_ERROR
        );
        assert!(idx.is_null());

        idx = idx_allocated;
        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                &mut memlimit,
                ptr::null(),
                decode_buffer.as_ptr(),
                ptr::null_mut(),
                0
            ),
            LZMA_PROG_ERROR
        );
        assert!(idx.is_null());

        idx = idx_allocated;
        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                &mut memlimit,
                ptr::null(),
                decode_buffer.as_ptr(),
                ptr::null_mut(),
                0
            ),
            LZMA_PROG_ERROR
        );
        assert!(idx.is_null());

        idx = idx_allocated;
        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                &mut memlimit,
                ptr::null(),
                decode_buffer.as_ptr(),
                &mut in_pos,
                0
            ),
            LZMA_DATA_ERROR
        );
        assert!(idx.is_null());

        in_pos = 1;
        idx = idx_allocated;
        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                &mut memlimit,
                ptr::null(),
                decode_buffer.as_ptr(),
                &mut in_pos,
                0
            ),
            LZMA_PROG_ERROR
        );
        assert!(idx.is_null());

        // Test too short input
        in_pos = 0;
        idx = idx_allocated;
        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                &mut memlimit,
                ptr::null(),
                decode_buffer.as_ptr(),
                &mut in_pos,
                decode_buffer_size - 1
            ),
            LZMA_DATA_ERROR
        );
        assert!(idx.is_null());

        lzma_index_end(idx_allocated, ptr::null());

        // Test expected successful decode
        in_pos = 0;
        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                &mut memlimit,
                ptr::null(),
                decode_buffer.as_ptr(),
                &mut in_pos,
                decode_buffer_size
            ),
            LZMA_OK
        );

        assert_eq!(in_pos, decode_buffer_size);
        assert!(index_is_equal(decode_test_index, idx));

        lzma_index_end(idx, ptr::null());

        // Test too much input. This won't read past
        // the end of the allocated array (decode_buffer_size bytes).
        in_pos = 0;
        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                &mut memlimit,
                ptr::null(),
                decode_buffer.as_ptr(),
                &mut in_pos,
                decode_buffer_size + 16
            ),
            LZMA_OK
        );

        assert_eq!(in_pos, decode_buffer_size);
        assert!(index_is_equal(decode_test_index, idx));

        lzma_index_end(idx, ptr::null());

        // Test too small memlimit
        in_pos = 0;
        memlimit = 1;
        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                &mut memlimit,
                ptr::null(),
                decode_buffer.as_ptr(),
                &mut in_pos,
                decode_buffer_size
            ),
            LZMA_MEMLIMIT_ERROR
        );
        assert!(memlimit > 1);
        assert!(memlimit < MEMLIMIT);

        lzma_index_end(decode_test_index, ptr::null());
    }
}

// With liblzma <= 5.8.2 (before the commit c8c22869e780),
// this triggers a buffer overflow in lzma_index_append().
#[test]
fn test_decode_empty_and_append() {
    unsafe {
        let mut buf = [0u8; 256];
        let mut idx = lzma_index_init(ptr::null());
        assert!(!idx.is_null());

        // Encode an empty Index.
        let mut buf_size: usize = 0;
        assert_eq!(
            lzma_index_buffer_encode(idx, buf.as_mut_ptr(), &mut buf_size, buf.len()),
            LZMA_OK
        );
        assert!(buf_size > 0);
        lzma_index_end(idx, ptr::null());
        idx = ptr::null_mut();

        // Decode the empty Index.
        let mut memlimit: u64 = MEMLIMIT;
        let mut buf_pos: usize = 0;
        assert_eq!(
            lzma_index_buffer_decode(
                &mut idx,
                &mut memlimit,
                ptr::null(),
                buf.as_ptr(),
                &mut buf_pos,
                buf_size
            ),
            LZMA_OK
        );
        assert_eq!(buf_pos, buf_size);

        // Append one Record to the decoded empty idx.
        assert_eq!(lzma_index_append(idx, ptr::null(), 55, 1), LZMA_OK);
        lzma_index_end(idx, ptr::null());
    }
}
