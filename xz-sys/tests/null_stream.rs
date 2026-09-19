#![cfg(not(target_family = "wasm"))]

//! xz-core takes `lzma_stream` by reference, so the NULL `strm` answer each
//! C entry point documents now lives in xz-sys alone. This pins every one of
//! those answers, including the store `lzma_index_decoder` must make before
//! it rejects the stream.

use std::ptr;

use xz_core::types::{
    LZMA_CHECK_CRC64, LZMA_CHECK_NONE, LZMA_PROG_ERROR, lzma_check, lzma_index, lzma_ret,
};
use xz_sys::*;

const MEMLIMIT: u64 = u64::MAX;

#[test]
fn accessors_answer_null_stream_without_touching_it() {
    unsafe {
        lzma_end(ptr::null_mut());
        assert_eq!(lzma_get_check(ptr::null()), LZMA_CHECK_NONE as lzma_check);
        assert_eq!(lzma_memusage(ptr::null()), 0);
        assert_eq!(lzma_memlimit_get(ptr::null()), 0);
        assert_eq!(lzma_memlimit_set(ptr::null_mut(), 1), LZMA_PROG_ERROR);

        let (mut progress_in, mut progress_out) = (u64::MAX, u64::MAX);
        lzma_get_progress(ptr::null_mut(), &mut progress_in, &mut progress_out);
        assert_eq!((progress_in, progress_out), (0, 0));
    }
}

#[test]
fn coders_reject_null_stream() {
    let null = ptr::null_mut();
    let results: [(&str, lzma_ret); 18] = unsafe {
        [
            ("lzma_code", lzma_code(null, LZMA_RUN)),
            (
                "lzma_filters_update",
                lzma_filters_update(null, ptr::null()),
            ),
            (
                "lzma_easy_encoder",
                lzma_easy_encoder(null, 6, LZMA_CHECK_CRC64),
            ),
            (
                "lzma_stream_encoder",
                lzma_stream_encoder(null, ptr::null(), LZMA_CHECK_CRC64),
            ),
            (
                "lzma_stream_decoder",
                lzma_stream_decoder(null, MEMLIMIT, 0),
            ),
            ("lzma_auto_decoder", lzma_auto_decoder(null, MEMLIMIT, 0)),
            ("lzma_alone_encoder", lzma_alone_encoder(null, ptr::null())),
            ("lzma_alone_decoder", lzma_alone_decoder(null, MEMLIMIT)),
            ("lzma_lzip_decoder", lzma_lzip_decoder(null, MEMLIMIT, 0)),
            ("lzma_raw_encoder", lzma_raw_encoder(null, ptr::null())),
            ("lzma_raw_decoder", lzma_raw_decoder(null, ptr::null())),
            (
                "lzma_block_encoder",
                lzma_block_encoder(null, ptr::null_mut()),
            ),
            (
                "lzma_block_decoder",
                lzma_block_decoder(null, ptr::null_mut()),
            ),
            ("lzma_index_encoder", lzma_index_encoder(null, ptr::null())),
            (
                "lzma_index_decoder",
                lzma_index_decoder(null, ptr::null_mut(), MEMLIMIT),
            ),
            (
                "lzma_microlzma_encoder",
                lzma_microlzma_encoder(null, ptr::null()),
            ),
            (
                "lzma_microlzma_decoder",
                lzma_microlzma_decoder(null, 0, 0, 0, 0),
            ),
            (
                "lzma_file_info_decoder",
                lzma_file_info_decoder(null, ptr::null_mut(), MEMLIMIT, 0),
            ),
        ]
    };
    for (name, ret) in results {
        assert_eq!(ret, LZMA_PROG_ERROR, "{name}(NULL strm)");
    }
}

#[cfg(feature = "parallel")]
#[test]
fn threaded_coders_reject_null_stream() {
    unsafe {
        assert_eq!(
            lzma_stream_encoder_mt(ptr::null_mut(), ptr::null()),
            LZMA_PROG_ERROR
        );
        assert_eq!(
            lzma_stream_decoder_mt(ptr::null_mut(), ptr::null()),
            LZMA_PROG_ERROR
        );
    }
}

#[test]
fn index_decoder_clears_the_index_before_rejecting_null_stream() {
    unsafe {
        let mut idx: *mut lzma_index = ptr::dangling_mut();
        assert_eq!(
            lzma_index_decoder(ptr::null_mut(), &mut idx, MEMLIMIT),
            LZMA_PROG_ERROR
        );
        assert!(idx.is_null());
    }
}
