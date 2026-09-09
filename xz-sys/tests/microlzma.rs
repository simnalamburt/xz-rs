#![cfg(not(target_family = "wasm"))]

// Port of vendor/xz/tests/test_microlzma.c

use std::mem::MaybeUninit;
use std::ptr;

use xz_sys::{
    LZMA_BUF_ERROR, LZMA_DATA_ERROR, LZMA_DICT_SIZE_DEFAULT, LZMA_DICT_SIZE_MIN, LZMA_FINISH,
    LZMA_FULL_BARRIER, LZMA_FULL_FLUSH, LZMA_LCLP_MAX, LZMA_OK, LZMA_OPTIONS_ERROR,
    LZMA_PRESET_DEFAULT, LZMA_PROG_ERROR, LZMA_RUN, LZMA_STREAM_END, LZMA_SYNC_FLUSH, LZMA_VLI_MAX,
    lzma_code, lzma_crc32, lzma_end, lzma_lzma_preset, lzma_microlzma_decoder,
    lzma_microlzma_encoder, lzma_options_lzma, lzma_stream,
};

const BUFFER_SIZE: usize = 1024;

// MicroLZMA encoded "Hello\nWorld\n" output size in bytes.
const ENCODED_OUTPUT_SIZE: usize = 17;

// Byte array of "Hello\nWorld\n". This is used for various encoder tests.
const HELLO_WORLD: [u8; 12] = [
    0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x0A, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x0A,
];

// This is the CRC32 value of the MicroLZMA encoding of "Hello\nWorld\n".
// The settings used were based on LZMA_PRESET_DEFAULT as of liblzma 5.6.0.
// This is to test for regressions that cause MicroLZMA output to change.
const HELLO_WORLD_ENCODED_CRC: u32 = 0x3CDE40A8;

// Byte array of "Goodbye World!". This is used for various decoder tests.
const GOODBYE_WORLD: [u8; 14] = [
    0x47, 0x6F, 0x6F, 0x64, 0x62, 0x79, 0x65, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21,
];

// Equivalent of LZMA_STREAM_INIT.
fn stream_init() -> lzma_stream {
    unsafe { MaybeUninit::zeroed().assume_init() }
}

fn options_default() -> lzma_options_lzma {
    let mut opt_lzma = unsafe { MaybeUninit::<lzma_options_lzma>::zeroed().assume_init() };
    // lzma_lzma_preset returns false (0) on success.
    assert_eq!(
        unsafe { lzma_lzma_preset(&mut opt_lzma, LZMA_PRESET_DEFAULT) },
        0
    );
    opt_lzma
}

// Function implementation borrowed from lzma_decoder.c. It is needed to
// ensure the first byte of a MicroLZMA stream is set correctly with the
// negation of the LZMA properties.
fn lzma_lzma_lclppb_decode(options: &mut lzma_options_lzma, byte: u8) -> bool {
    if byte > (4 * 5 + 4) * 9 + 8 {
        return true;
    }

    // See the file format specification to understand this.
    let mut byte = u32::from(byte);
    options.pb = byte / (9 * 5);
    byte -= options.pb * 9 * 5;
    options.lp = byte / 9;
    options.lc = byte - options.lp * 9;

    options.lc + options.lp > LZMA_LCLP_MAX
}

///////////////////
// Encoder tests //
///////////////////

// This tests a few of the basic options. These options are not unique to
// MicroLZMA in any way, its mostly ensuring that the options are actually
// being checked before initializing the decoder internals.
#[test]
fn test_encode_options() {
    unsafe {
        let mut strm = stream_init();
        let mut opt_lzma = options_default();

        // NULL stream
        assert_eq!(
            lzma_microlzma_encoder(ptr::null_mut(), &opt_lzma),
            LZMA_PROG_ERROR
        );

        // lc/lp/pb = 5/0/2 (lc invalid)
        opt_lzma.lc = 5;
        opt_lzma.lp = 0;
        opt_lzma.pb = 2;
        assert_eq!(
            lzma_microlzma_encoder(&mut strm, &opt_lzma),
            LZMA_OPTIONS_ERROR
        );

        // lc/lp/pb = 0/5/2 (lp invalid)
        opt_lzma.lc = 0;
        opt_lzma.lp = 5;
        opt_lzma.pb = 2;
        assert_eq!(
            lzma_microlzma_encoder(&mut strm, &opt_lzma),
            LZMA_OPTIONS_ERROR
        );

        // lc/lp/pb = 3/2/2 (lc + lp invalid)
        opt_lzma.lc = 3;
        opt_lzma.lp = 2;
        opt_lzma.pb = 2;
        assert_eq!(
            lzma_microlzma_encoder(&mut strm, &opt_lzma),
            LZMA_OPTIONS_ERROR
        );

        // lc/lp/pb = 3/0/5 (pb invalid)
        opt_lzma.lc = 3;
        opt_lzma.lp = 0;
        opt_lzma.pb = 5;
        assert_eq!(
            lzma_microlzma_encoder(&mut strm, &opt_lzma),
            LZMA_OPTIONS_ERROR
        );

        // Zero out lp, pb, lc options to not interfere with later tests.
        opt_lzma.lp = 0;
        opt_lzma.pb = 0;
        opt_lzma.lc = 0;

        // Set invalid dictionary size.
        opt_lzma.dict_size = LZMA_DICT_SIZE_MIN - 1;
        assert_eq!(
            lzma_microlzma_encoder(&mut strm, &opt_lzma),
            LZMA_OPTIONS_ERROR
        );

        // Maximum dictionary size for the encoder, as described in lzma12.h
        // is 1.5 GiB.
        opt_lzma.dict_size = (1u32 << 30) + (1u32 << 29) + 1;
        assert_eq!(
            lzma_microlzma_encoder(&mut strm, &opt_lzma),
            LZMA_OPTIONS_ERROR
        );

        lzma_end(&mut strm);
    }
}

#[test]
fn test_encode_basic() {
    unsafe {
        let mut strm = stream_init();
        let opt_lzma = options_default();

        // Initialize the encoder using the default options.
        assert_eq!(lzma_microlzma_encoder(&mut strm, &opt_lzma), LZMA_OK);

        let mut output = [0u8; BUFFER_SIZE];

        strm.next_in = HELLO_WORLD.as_ptr();
        strm.avail_in = HELLO_WORLD.len();
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = output.len();

        // Everything must be encoded in one lzma_code() call.
        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_STREAM_END);

        // Check that the entire input was consumed.
        assert_eq!(strm.total_in, HELLO_WORLD.len() as u64);

        // Check that the first byte in the output stream is not 0x00.
        // In a regular raw LZMA stream the first byte is always 0x00.
        // In MicroLZMA the first byte replaced by the bitwise-negation
        // of the LZMA properties.
        assert_ne!(output[0], 0x00);

        let props = !output[0];

        let mut test_options = MaybeUninit::<lzma_options_lzma>::zeroed().assume_init();
        assert!(!lzma_lzma_lclppb_decode(&mut test_options, props));

        assert_eq!(opt_lzma.lc, test_options.lc);
        assert_eq!(opt_lzma.lp, test_options.lp);
        assert_eq!(opt_lzma.pb, test_options.pb);

        // Compute the check over the output data. This is compared to
        // the expected check value.
        let check_val = lzma_crc32(output.as_ptr(), strm.total_out as usize, 0);

        assert_eq!(check_val, HELLO_WORLD_ENCODED_CRC);

        lzma_end(&mut strm);
    }
}

// This tests the behavior when strm.avail_out is so small it cannot hold
// the header plus 1 encoded byte (< 6).
#[test]
fn test_encode_small_out() {
    unsafe {
        let mut strm = stream_init();
        let opt_lzma = options_default();

        assert_eq!(lzma_microlzma_encoder(&mut strm, &opt_lzma), LZMA_OK);

        let mut output = [0u8; BUFFER_SIZE];

        strm.next_in = HELLO_WORLD.as_ptr();
        strm.avail_in = HELLO_WORLD.len();
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = 5;

        // LZMA_PROG_ERROR is expected when strm.avail_out < 6
        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_PROG_ERROR);

        // The encoder must be reset because coders cannot be used again
        // after returning LZMA_PROG_ERROR.
        assert_eq!(lzma_microlzma_encoder(&mut strm, &opt_lzma), LZMA_OK);

        // Reset strm.avail_out to be > 6, but not enough to hold all of the
        // compressed data.
        strm.avail_out = ENCODED_OUTPUT_SIZE - 1;

        // Encoding should not return an error now.
        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_STREAM_END);
        assert!(strm.total_in < HELLO_WORLD.len() as u64);

        lzma_end(&mut strm);
    }
}

// LZMA_FINISH is the only supported action. All others must
// return LZMA_PROG_ERROR.
#[test]
fn test_encode_actions() {
    unsafe {
        let mut strm = stream_init();
        let opt_lzma = options_default();

        let actions = [
            LZMA_RUN,
            LZMA_SYNC_FLUSH,
            LZMA_FULL_FLUSH,
            LZMA_FULL_BARRIER,
        ];

        for action in actions {
            assert_eq!(lzma_microlzma_encoder(&mut strm, &opt_lzma), LZMA_OK);

            let mut output = [0u8; BUFFER_SIZE];

            strm.next_in = HELLO_WORLD.as_ptr();
            strm.avail_in = HELLO_WORLD.len();
            strm.next_out = output.as_mut_ptr();
            strm.avail_out = output.len();

            assert_eq!(lzma_code(&mut strm, action), LZMA_PROG_ERROR);
        }

        lzma_end(&mut strm);
    }
}

///////////////////
// Decoder tests //
///////////////////

// Helper function to encode data and return the buffer holding the
// compressed data along with the compressed size. The buffer is twice
// the input size, mirroring the extra space allocated by the C test.
fn basic_microlzma_encode(input: &[u8]) -> (Vec<u8>, usize) {
    unsafe {
        let mut strm = stream_init();
        let opt_lzma = options_default();

        // Lazy way to set the output size since the input should never
        // inflate by much in these simple test cases.
        let out_size = input.len() << 1;
        let mut compressed = vec![0u8; out_size];

        assert_eq!(lzma_microlzma_encoder(&mut strm, &opt_lzma), LZMA_OK);

        strm.next_in = input.as_ptr();
        strm.avail_in = input.len();
        strm.next_out = compressed.as_mut_ptr();
        strm.avail_out = out_size;

        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_STREAM_END);

        // Check that the entire input was consumed and that it fit into
        // the output buffer.
        assert_eq!(strm.total_in, input.len() as u64);

        lzma_end(&mut strm);

        // lzma_end() doesn't touch other members of lzma_stream than
        // lzma_stream.internal so using strm.total_out here is fine.
        (compressed, strm.total_out as usize)
    }
}

#[test]
fn test_decode_options() {
    unsafe {
        // NULL stream
        assert_eq!(
            lzma_microlzma_decoder(
                ptr::null_mut(),
                BUFFER_SIZE as u64,
                HELLO_WORLD.len() as u64,
                1,
                LZMA_DICT_SIZE_DEFAULT,
            ),
            LZMA_PROG_ERROR
        );

        // Uncompressed size larger than max
        let mut strm = stream_init();
        assert_eq!(
            lzma_microlzma_decoder(
                &mut strm,
                BUFFER_SIZE as u64,
                LZMA_VLI_MAX + 1,
                1,
                LZMA_DICT_SIZE_DEFAULT,
            ),
            LZMA_OPTIONS_ERROR
        );
    }
}

// Test that decoding succeeds when uncomp_size is correct regardless of
// the value of uncomp_size_is_exact.
#[test]
fn test_decode_uncomp_size_is_exact() {
    let (encoded, encoded_size) = basic_microlzma_encode(&GOODBYE_WORLD);

    unsafe {
        let mut strm = stream_init();

        assert_eq!(
            lzma_microlzma_decoder(
                &mut strm,
                encoded_size as u64,
                GOODBYE_WORLD.len() as u64,
                1,
                LZMA_DICT_SIZE_DEFAULT,
            ),
            LZMA_OK
        );

        let mut output = [0u8; BUFFER_SIZE];

        strm.next_in = encoded.as_ptr();
        strm.avail_in = encoded_size;
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = output.len();

        assert_eq!(lzma_code(&mut strm, LZMA_RUN), LZMA_STREAM_END);
        assert_eq!(strm.total_in, encoded_size as u64);

        assert_eq!(strm.total_out, GOODBYE_WORLD.len() as u64);
        assert_eq!(&output[..GOODBYE_WORLD.len()], GOODBYE_WORLD);

        // Reset decoder with uncomp_size_is_exact set to false and
        // uncomp_size set to correct value. Also test using the
        // uncompressed size as the dictionary size.
        assert_eq!(
            lzma_microlzma_decoder(
                &mut strm,
                encoded_size as u64,
                GOODBYE_WORLD.len() as u64,
                0,
                GOODBYE_WORLD.len() as u32,
            ),
            LZMA_OK
        );

        strm.next_in = encoded.as_ptr();
        strm.avail_in = encoded_size;
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = output.len();

        assert_eq!(lzma_code(&mut strm, LZMA_RUN), LZMA_STREAM_END);
        assert_eq!(strm.total_in, encoded_size as u64);

        assert_eq!(strm.total_out, GOODBYE_WORLD.len() as u64);
        assert_eq!(&output[..GOODBYE_WORLD.len()], GOODBYE_WORLD);

        lzma_end(&mut strm);
    }
}

// This tests decoding when MicroLZMA decoder is called with
// an incorrect uncompressed size.
#[test]
fn test_decode_uncomp_size_wrong() {
    let (encoded, encoded_size) = basic_microlzma_encode(&GOODBYE_WORLD);

    unsafe {
        let mut strm = stream_init();
        assert_eq!(
            lzma_microlzma_decoder(
                &mut strm,
                encoded_size as u64,
                (GOODBYE_WORLD.len() + 1) as u64,
                0,
                LZMA_DICT_SIZE_DEFAULT,
            ),
            LZMA_OK
        );

        let mut output = [0u8; BUFFER_SIZE];

        strm.next_in = encoded.as_ptr();
        strm.avail_in = encoded_size;
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = output.len();

        // LZMA_OK should be returned because the input size given was
        // larger than the actual encoded size. The decoder is expecting
        // more input to possibly fill the uncompressed size that was set.
        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_OK);

        assert_eq!(strm.total_out, GOODBYE_WORLD.len() as u64);

        assert_eq!(&output[..GOODBYE_WORLD.len()], GOODBYE_WORLD);

        // Next, test with uncomp_size_is_exact set.
        assert_eq!(
            lzma_microlzma_decoder(
                &mut strm,
                encoded_size as u64,
                (GOODBYE_WORLD.len() + 1) as u64,
                1,
                LZMA_DICT_SIZE_DEFAULT,
            ),
            LZMA_OK
        );

        strm.next_in = encoded.as_ptr();
        strm.avail_in = encoded_size;
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = output.len();

        // No error detected, even though all input was consumed and there
        // is more room in the output buffer. The generic lzma_code() logic
        // eventually reports LZMA_BUF_ERROR after repeated no-progress calls.
        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_OK);
        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_OK);
        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_BUF_ERROR);

        assert_eq!(strm.total_out, GOODBYE_WORLD.len() as u64);
        assert_eq!(&output[..GOODBYE_WORLD.len()], GOODBYE_WORLD);

        // Reset stream with uncomp_size smaller than the real
        // uncompressed size.
        assert_eq!(
            lzma_microlzma_decoder(
                &mut strm,
                encoded_size as u64,
                (HELLO_WORLD.len() - 1) as u64,
                1,
                LZMA_DICT_SIZE_DEFAULT,
            ),
            LZMA_OK
        );

        strm.next_in = encoded.as_ptr();
        strm.avail_in = encoded_size;
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = output.len();

        // This case actually results in an error since it decodes the full
        // uncompressed size but the range coder is not in the proper state
        // for the stream to end.
        assert_eq!(lzma_code(&mut strm, LZMA_RUN), LZMA_DATA_ERROR);

        lzma_end(&mut strm);
    }
}

#[test]
fn test_decode_comp_size_wrong() {
    let (encoded, encoded_size) = basic_microlzma_encode(&GOODBYE_WORLD);

    unsafe {
        let mut strm = stream_init();

        // encoded_size + 1 is safe because extra space was allocated for
        // the encoded buffer. The extra space isn't read by the decoder
        // because avail_in is still encoded_size.
        assert_eq!(
            lzma_microlzma_decoder(
                &mut strm,
                (encoded_size + 1) as u64,
                GOODBYE_WORLD.len() as u64,
                1,
                LZMA_DICT_SIZE_DEFAULT,
            ),
            LZMA_OK
        );

        let mut output = [0u8; BUFFER_SIZE];

        strm.next_in = encoded.as_ptr();
        strm.avail_in = encoded_size;
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = output.len();

        // When uncomp_size_is_exact is set, the compressed size must be
        // correct or else LZMA_DATA_ERROR is returned.
        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_DATA_ERROR);

        assert_eq!(
            lzma_microlzma_decoder(
                &mut strm,
                (encoded_size + 1) as u64,
                GOODBYE_WORLD.len() as u64,
                0,
                LZMA_DICT_SIZE_DEFAULT,
            ),
            LZMA_OK
        );

        strm.next_in = encoded.as_ptr();
        strm.avail_in = encoded_size;
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = output.len();

        // When uncomp_size_is_exact is not set, the decoder does not
        // detect when the compressed size is wrong as long as all of the
        // expected output has been decoded. This is because the decoder
        // assumes that the real uncompressed size might be bigger than
        // the specified value and in that case more input might be needed
        // as well.
        assert_eq!(lzma_code(&mut strm, LZMA_FINISH), LZMA_STREAM_END);

        lzma_end(&mut strm);
    }
}

#[test]
fn test_decode_bad_lzma_properties() {
    let (encoded, encoded_size) = basic_microlzma_encode(&GOODBYE_WORLD);

    unsafe {
        // Alter first byte to encode invalid LZMA properties.
        let mut compressed = encoded[..encoded_size].to_vec();

        // lc=3, lp=2, pb=2
        compressed[0] = !0x6Fu8;

        let mut strm = stream_init();
        assert_eq!(
            lzma_microlzma_decoder(
                &mut strm,
                encoded_size as u64,
                GOODBYE_WORLD.len() as u64,
                0,
                LZMA_DICT_SIZE_DEFAULT,
            ),
            LZMA_OK
        );

        let mut output = [0u8; BUFFER_SIZE];

        strm.next_in = compressed.as_ptr();
        strm.avail_in = encoded_size;
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = output.len();

        assert_eq!(lzma_code(&mut strm, LZMA_RUN), LZMA_OPTIONS_ERROR);

        // Use valid, but incorrect LZMA properties.
        // lc=3, lp=1, pb=2
        compressed[0] = !0x66u8;

        assert_eq!(
            lzma_microlzma_decoder(
                &mut strm,
                encoded_size as u64,
                GOODBYE_WORLD.len() as u64,
                1,
                LZMA_DICT_SIZE_DEFAULT,
            ),
            LZMA_OK
        );

        strm.next_in = compressed.as_ptr();
        strm.avail_in = encoded_size;
        strm.next_out = output.as_mut_ptr();
        strm.avail_out = output.len();

        assert_eq!(lzma_code(&mut strm, LZMA_RUN), LZMA_DATA_ERROR);

        lzma_end(&mut strm);
    }
}
