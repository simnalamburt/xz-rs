#![cfg(not(target_family = "wasm"))]

use std::mem::MaybeUninit;
use std::ptr;

use xz_sys::{
    LZMA_CHECK_CRC64, LZMA_OK, LZMA_RUN, LZMA_SEEK_NEEDED, LZMA_STREAM_END, lzma_code,
    lzma_easy_buffer_encode, lzma_end, lzma_file_info_decoder, lzma_index, lzma_index_end,
    lzma_index_stream_count, lzma_index_uncompressed_size, lzma_stream,
};

fn encode_stream(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; data.len() + 4096];
    let mut out_pos = 0usize;
    unsafe {
        let ret = lzma_easy_buffer_encode(
            6,
            LZMA_CHECK_CRC64,
            ptr::null(),
            data.as_ptr(),
            data.len(),
            out.as_mut_ptr(),
            &mut out_pos,
            out.len(),
        );
        assert_eq!(ret, LZMA_OK);
    }
    out.truncate(out_pos);
    out
}

// Incompressible payload so each stream stays larger than the decoder's
// 8 KiB temp buffer, forcing real (backward) seeks between the streams.
fn pseudo_random_payload(seed: u64, len: usize) -> Vec<u8> {
    let mut state = seed;
    (0..len)
        .map(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            (state >> 56) as u8
        })
        .collect()
}

// The whole file is handed over in a single input buffer, so every seek the
// decoder requests (including the backward seeks between concatenated
// streams) is satisfied by adjusting *in_pos instead of LZMA_SEEK_NEEDED.
#[test]
fn file_info_decodes_concatenated_streams_from_one_buffer() {
    let first = pseudo_random_payload(1, 16 << 10);
    let second = pseudo_random_payload(2, 16 << 10);
    let mut file = encode_stream(&first);
    file.extend_from_slice(&encode_stream(&second));

    unsafe {
        let mut strm: lzma_stream = MaybeUninit::zeroed().assume_init();
        let mut index: *mut lzma_index = ptr::null_mut();
        let ret = lzma_file_info_decoder(&mut strm, &mut index, u64::MAX, file.len() as u64);
        assert_eq!(ret, LZMA_OK);

        strm.next_in = file.as_ptr();
        strm.avail_in = file.len();
        loop {
            match lzma_code(&mut strm, LZMA_RUN) {
                LZMA_STREAM_END => break,
                LZMA_SEEK_NEEDED => {
                    let pos = strm.seek_pos as usize;
                    assert!(pos <= file.len());
                    strm.next_in = file.as_ptr().add(pos);
                    strm.avail_in = file.len() - pos;
                }
                ret => panic!("unexpected lzma_ret {ret}"),
            }
        }

        assert!(!index.is_null());
        assert_eq!(lzma_index_stream_count(index), 2);
        assert_eq!(
            lzma_index_uncompressed_size(index),
            (first.len() + second.len()) as u64
        );
        lzma_index_end(index, ptr::null());
        lzma_end(&mut strm);
    }
}
