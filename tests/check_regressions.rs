#![cfg(not(target_family = "wasm"))]

use std::mem::MaybeUninit;

use xz_core::check::crc32_fast::crc32;
use xz_core::check::crc64_fast::crc64;
use xz_core::types::{
    lzma_check_finish, lzma_check_init, lzma_check_size, lzma_check_state, lzma_check_update,
    LZMA_CHECK_CRC32, LZMA_CHECK_CRC64, LZMA_CHECK_SHA256,
};

unsafe fn finish_check(check_type: u32, input: &[u8]) -> lzma_check_state {
    let mut check = MaybeUninit::<lzma_check_state>::zeroed().assume_init();
    lzma_check_init(&mut check, check_type);
    lzma_check_update(&mut check, check_type, input.as_ptr(), input.len());
    lzma_check_finish(&mut check, check_type);
    check
}

#[test]
fn crc_functions_match_standard_check_vectors() {
    let input = b"123456789";

    assert_eq!(crc32(input, 0), 0xcbf4_3926);
    assert_eq!(crc64(input, 0), 0x995d_c9bb_df19_39fa);
}

#[test]
fn check_finish_writes_crc_bytes_in_little_endian_order() {
    unsafe {
        let input = b"abc";

        let mut crc32 = MaybeUninit::<lzma_check_state>::zeroed().assume_init();
        lzma_check_init(&mut crc32, LZMA_CHECK_CRC32);
        lzma_check_update(&mut crc32, LZMA_CHECK_CRC32, input.as_ptr(), input.len());
        let crc32_value = crc32.state.crc32;
        lzma_check_finish(&mut crc32, LZMA_CHECK_CRC32);
        assert_eq!(&crc32.buffer.u8_0[..4], &crc32_value.to_le_bytes());

        let mut crc64 = MaybeUninit::<lzma_check_state>::zeroed().assume_init();
        lzma_check_init(&mut crc64, LZMA_CHECK_CRC64);
        lzma_check_update(&mut crc64, LZMA_CHECK_CRC64, input.as_ptr(), input.len());
        let crc64_value = crc64.state.crc64;
        lzma_check_finish(&mut crc64, LZMA_CHECK_CRC64);
        assert_eq!(&crc64.buffer.u8_0[..8], &crc64_value.to_le_bytes());
    }
}

#[test]
fn sha256_check_matches_standard_vector() {
    let expected = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];

    unsafe {
        let check = finish_check(LZMA_CHECK_SHA256, b"abc");
        let size = lzma_check_size(LZMA_CHECK_SHA256) as usize;
        assert_eq!(size, expected.len());
        assert_eq!(&check.buffer.u8_0[..size], expected);
    }
}
