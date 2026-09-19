#![cfg(not(target_family = "wasm"))]

//! The variable-length integer coders are the one pair whose loop shape had to
//! be rewritten to drop the raw pointers, so every result they can produce is
//! compared against the same call into the C library.

use std::ptr;

use xz_sys::{lzma_ret, lzma_vli, lzma_vli_decode, lzma_vli_encode};

const LZMA_VLI_MAX: lzma_vli = (1 << 63) - 1;

/// Values on both sides of every byte boundary, plus the ends of the range and
/// one value past it.
fn values() -> Vec<lzma_vli> {
    let mut v = vec![0, 1, 2, 0x7f, 0x80, 0x81, LZMA_VLI_MAX, LZMA_VLI_MAX + 1];
    for shift in 1..9 {
        let edge: lzma_vli = 1 << (7 * shift);
        v.extend([edge - 1, edge, edge + 1]);
    }
    v.sort_unstable();
    v.dedup();
    v
}

fn encode_both(
    vli: lzma_vli,
    out_size: usize,
    multi_call: bool,
) -> (lzma_ret, Vec<u8>, usize, usize) {
    unsafe {
        let mut rs_out = vec![0xaau8; out_size];
        let mut rs_pos: usize = 0;
        let mut rs_vli_pos: usize = 0;
        let rs = lzma_vli_encode(
            vli,
            if multi_call {
                &mut rs_vli_pos
            } else {
                ptr::null_mut()
            },
            rs_out.as_mut_ptr(),
            &mut rs_pos,
            out_size,
        );

        let mut c_out = vec![0xaau8; out_size];
        let mut c_pos: usize = 0;
        let mut c_vli_pos: usize = 0;
        let c = liblzma_sys::lzma_vli_encode(
            vli,
            if multi_call {
                &mut c_vli_pos
            } else {
                ptr::null_mut()
            },
            c_out.as_mut_ptr(),
            &mut c_pos,
            out_size,
        );

        assert_eq!(
            rs as u32, c as u32,
            "ret for vli={vli:#x} out_size={out_size} multi={multi_call}"
        );
        assert_eq!(
            rs_out, c_out,
            "bytes for vli={vli:#x} out_size={out_size} multi={multi_call}"
        );
        assert_eq!(
            rs_pos, c_pos,
            "out_pos for vli={vli:#x} out_size={out_size} multi={multi_call}"
        );
        assert_eq!(
            rs_vli_pos, c_vli_pos,
            "vli_pos for vli={vli:#x} out_size={out_size} multi={multi_call}"
        );
        (rs, rs_out, rs_pos, rs_vli_pos)
    }
}

#[test]
fn encode_matches_c_for_every_buffer_size() {
    for vli in values() {
        for out_size in 0..12 {
            for multi_call in [false, true] {
                encode_both(vli, out_size, multi_call);
            }
        }
    }
}

/// Feeding the encoder one byte of room at a time is the mode `vli_pos`
/// exists for; the whole call sequence has to agree, not just its end.
#[test]
fn encode_one_byte_at_a_time_matches_c() {
    unsafe {
        for vli in values() {
            let mut rs_out = [0xaau8; 16];
            let mut c_out = [0xaau8; 16];
            let (mut rs_pos, mut rs_vli_pos) = (0usize, 0usize);
            let (mut c_pos, mut c_vli_pos) = (0usize, 0usize);

            for limit in 1..=16usize {
                let rs = lzma_vli_encode(
                    vli,
                    &mut rs_vli_pos,
                    rs_out.as_mut_ptr(),
                    &mut rs_pos,
                    limit,
                );
                let c = liblzma_sys::lzma_vli_encode(
                    vli,
                    &mut c_vli_pos,
                    c_out.as_mut_ptr(),
                    &mut c_pos,
                    limit,
                );
                assert_eq!(rs as u32, c as u32, "ret at limit={limit} for vli={vli:#x}");
                assert_eq!(rs_out, c_out, "bytes at limit={limit} for vli={vli:#x}");
                assert_eq!(rs_pos, c_pos, "out_pos at limit={limit} for vli={vli:#x}");
                assert_eq!(
                    rs_vli_pos, c_vli_pos,
                    "vli_pos at limit={limit} for vli={vli:#x}"
                );
                if rs != xz_sys::LZMA_OK {
                    break;
                }
            }
        }
    }
}

fn decode_both(input: &[u8], in_size: usize, multi_call: bool) {
    unsafe {
        let mut rs_vli: lzma_vli = 0xdead_beef;
        let mut rs_pos: usize = 0;
        let mut rs_vli_pos: usize = 0;
        let rs = lzma_vli_decode(
            &mut rs_vli,
            if multi_call {
                &mut rs_vli_pos
            } else {
                ptr::null_mut()
            },
            input.as_ptr(),
            &mut rs_pos,
            in_size,
        );

        let mut c_vli: lzma_vli = 0xdead_beef;
        let mut c_pos: usize = 0;
        let mut c_vli_pos: usize = 0;
        let c = liblzma_sys::lzma_vli_decode(
            &mut c_vli,
            if multi_call {
                &mut c_vli_pos
            } else {
                ptr::null_mut()
            },
            input.as_ptr(),
            &mut c_pos,
            in_size,
        );

        assert_eq!(
            rs as u32, c as u32,
            "ret for {input:?} in_size={in_size} multi={multi_call}"
        );
        assert_eq!(
            rs_vli, c_vli,
            "vli for {input:?} in_size={in_size} multi={multi_call}"
        );
        assert_eq!(
            rs_pos, c_pos,
            "in_pos for {input:?} in_size={in_size} multi={multi_call}"
        );
        assert_eq!(
            rs_vli_pos, c_vli_pos,
            "vli_pos for {input:?} in_size={in_size} multi={multi_call}"
        );
    }
}

#[test]
fn decode_matches_c_on_round_trips_and_truncations() {
    for vli in values() {
        let (ret, buf, len, _) = encode_both(vli, 16, false);
        if ret != xz_sys::LZMA_OK {
            continue;
        }
        // Every prefix, so the short-input paths of both modes are covered.
        for in_size in 0..=len {
            for multi_call in [false, true] {
                decode_both(&buf, in_size, multi_call);
            }
        }
    }
}

/// Encodings the encoder never produces: a padded zero byte, a ten-byte run of
/// continuation bits, and a terminator with nothing before it.
#[test]
fn decode_matches_c_on_malformed_input() {
    let cases: [&[u8]; 7] = [
        &[],
        &[0x00],
        &[0x80, 0x00],
        &[0x80, 0x80, 0x00],
        &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00],
        &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        &[0xff; 12],
    ];
    for input in cases {
        for in_size in 0..=input.len() {
            for multi_call in [false, true] {
                decode_both(input, in_size, multi_call);
            }
        }
    }
}
