//! LZMA/XZ encoding and decoding streams
//!
//! This library provides LZMA and xz encoding/decoding streams.
//! I/O streams are provided in the `read`, `write`,
//! and `bufread` modules (same types, different bounds). Raw in-memory
//! compression/decompression is provided via the `stream` module and contains
//! many of the raw APIs exposed by the selected backend.
//!
//! # Examples
//!
//! ```no_run
//! use xz::read::{XzDecoder, XzEncoder};
//! use std::io::prelude::*;
//!
//! // Round trip some bytes from a byte source, into a compressor, into a
//! // decompressor, and finally into a vector.
//! let data = "Hello, World!".as_bytes();
//! let compressor = XzEncoder::new(data, 9);
//! let mut decompressor = XzDecoder::new(compressor);
//!
//! let mut contents = String::new();
//! decompressor.read_to_string(&mut contents).unwrap();
//! assert_eq!(contents, "Hello, World!");
//! ```
//! # Static linking
//!
//! You can enable static-linking using the `static` feature, so that the XZ
//! library is not required at runtime when using the `liblzma-sys` backend:
//!
//! ```toml
//! xz = { version = "0.4", default-features = false, features = ["liblzma-sys", "static"] }
//! ```
//!
//! # Multithreading
//!
//! This crate optionally can support multithreading using the `parallel`
//! feature of this crate:
//!
//! ```toml
//! xz = { version = "0.4", features = ["parallel"] }
//! ```
//!
//! # Async I/O
//!
//! Dropped `tokio` support since 0.4.0.
//! If you need to use async I/O, use `async-compression` crate with `lzma` feature flag.
//!
//! ```toml
//! async-compression = { version = "0.4", features = ["lzma"] }
//! ```

#![doc(html_root_url = "https://docs.rs/xz/0.4.6")]
#![deny(missing_docs)]

#[cfg(any(
    all(feature = "xz-core", feature = "xz-sys"),
    all(feature = "xz-core", feature = "liblzma-sys"),
    all(feature = "xz-sys", feature = "liblzma-sys"),
))]
compile_error!("Enable exactly one of `xz-core`, `xz-sys`, or `liblzma-sys`");

#[cfg(not(any(feature = "xz-core", feature = "xz-sys", feature = "liblzma-sys")))]
compile_error!("Enable `xz-core`, `xz-sys`, or `liblzma-sys`");

#[cfg(feature = "xz-core")]
pub(crate) mod sys {
    pub(crate) use xz_core::check::check::lzma_check_is_supported;
    pub(crate) use xz_core::common::{
        alone_decoder::lzma_alone_decoder,
        alone_encoder::lzma_alone_encoder,
        auto_decoder::lzma_auto_decoder,
        common::{lzma_code, lzma_end, lzma_memlimit_get, lzma_memlimit_set},
        easy_encoder::lzma_easy_encoder,
        filter_decoder::lzma_raw_decoder,
        filter_encoder::lzma_raw_encoder,
        index::lzma_index_end,
        index_decoder::lzma_index_buffer_decode,
        lzip_decoder::lzma_lzip_decoder,
        stream_decoder::lzma_stream_decoder,
        stream_encoder::lzma_stream_encoder,
        string_conversion::LZMA_PRESET_DEFAULT,
    };
    #[cfg(feature = "parallel")]
    pub(crate) use xz_core::common::{
        filter_encoder::lzma_mt_block_size,
        stream_mt::{
            lzma_stream_decoder_mt, lzma_stream_encoder_mt, lzma_stream_encoder_mt_memusage,
        },
    };
    pub(crate) use xz_core::lz::lz_encoder::lzma_mf_is_supported;
    pub(crate) use xz_core::lzma::lzma_encoder_presets::{
        lzma_lzma_preset, LZMA_PRESET_LEVEL_MASK,
    };
    pub(crate) use xz_core::types::*;

    /// xz-core takes the properties as a slice. The other two backends take a
    /// pointer and a length, so this restores the shape the rest of the crate
    /// shares across backends.
    pub(crate) unsafe fn lzma_properties_decode(
        filter: *mut lzma_filter,
        allocator: *const lzma_allocator,
        props: *const u8,
        props_size: usize,
    ) -> lzma_ret {
        xz_core::common::filter_decoder::lzma_properties_decode(
            &mut *filter,
            allocator,
            core::slice::from_raw_parts(props, props_size),
        )
    }

    /// xz-core takes the Index by reference. The other two backends take a bare
    /// pointer, so this restores the shape the rest of the crate shares across
    /// backends.
    pub(crate) unsafe fn lzma_index_uncompressed_size(i: *const lzma_index) -> lzma_vli {
        xz_core::common::index::lzma_index_uncompressed_size(&*i)
    }

    /// xz-core states the twelve-byte Stream Footer in the type. The other two
    /// backends take a bare pointer, so this restores the shape the rest of the
    /// crate shares across backends.
    pub(crate) unsafe fn lzma_stream_footer_decode(
        options: *mut lzma_stream_flags,
        input: *const u8,
    ) -> lzma_ret {
        xz_core::common::stream_flags_decoder::lzma_stream_footer_decode(
            &mut *options,
            &*input.cast(),
        )
    }
}

#[cfg(feature = "xz-sys")]
pub(crate) mod sys {
    pub(crate) use xz_sys::*;
}

#[cfg(feature = "liblzma-sys")]
pub(crate) mod sys {
    pub(crate) use liblzma_sys::*;
}

use std::io::{self, prelude::*};

pub mod stream;

pub mod bufread;
pub mod read;
pub mod write;

/// Decompress from the given source as if using a [read::XzDecoder].
///
/// Result will be in the xz format.
pub fn decode_all<R: Read>(source: R) -> io::Result<Vec<u8>> {
    let mut vec = Vec::new();
    let mut r = read::XzDecoder::new(source);
    r.read_to_end(&mut vec)?;
    Ok(vec)
}

/// Compress from the given source as if using a [read::XzEncoder].
///
/// The output data will be in the xz format.
/// The `level` argument is typically 0-9 with 6 being a good default.
/// To use the slower `xz --extreme`-style preset, bitwise-OR a level with
/// [`stream::PRESET_EXTREME`] (for example, `6 | stream::PRESET_EXTREME`).
pub fn encode_all<R: Read>(source: R, level: u32) -> io::Result<Vec<u8>> {
    let mut vec = Vec::new();
    let mut r = read::XzEncoder::new(source, level);
    r.read_to_end(&mut vec)?;
    Ok(vec)
}

/// Compress all data from the given source as if using a [read::XzEncoder].
///
/// Compressed data will be appended to `destination`.
/// The `level` argument is typically 0-9 with 6 being a good default.
/// To use the slower `xz --extreme`-style preset, bitwise-OR a level with
/// [`stream::PRESET_EXTREME`] (for example, `6 | stream::PRESET_EXTREME`).
pub fn copy_encode<R: Read, W: Write>(source: R, mut destination: W, level: u32) -> io::Result<()> {
    io::copy(&mut read::XzEncoder::new(source, level), &mut destination)?;
    Ok(())
}

/// Decompress all data from the given source as if using a [read::XzDecoder].
///
/// Decompressed data will be appended to `destination`.
pub fn copy_decode<R: Read, W: Write>(source: R, mut destination: W) -> io::Result<()> {
    io::copy(&mut read::XzDecoder::new(source), &mut destination)?;
    Ok(())
}

/// Find the size in bytes of uncompressed data from xz file.
pub fn uncompressed_size<R: Read + Seek>(mut source: R) -> io::Result<u64> {
    use std::mem::MaybeUninit;
    let mut footer = [0u8; sys::LZMA_STREAM_HEADER_SIZE as usize];

    source.seek(io::SeekFrom::End(0 - (sys::LZMA_STREAM_HEADER_SIZE as i64)))?;
    source.read_exact(&mut footer)?;

    let lzma_stream_flags = unsafe {
        let mut lzma_stream_flags = MaybeUninit::uninit();
        let ret = sys::lzma_stream_footer_decode(lzma_stream_flags.as_mut_ptr(), footer.as_ptr());

        if ret != sys::LZMA_OK {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to parse lzma footer",
            ));
        }

        lzma_stream_flags.assume_init()
    };

    // backward_size can exceed usize on 32-bit targets; fail cleanly instead
    // of truncating.
    let index_plus_footer =
        usize::try_from(sys::LZMA_STREAM_HEADER_SIZE as u64 + lzma_stream_flags.backward_size)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "lzma index too large"))?;

    source.seek(io::SeekFrom::End(0 - index_plus_footer as i64))?;

    let buf = source
        .bytes()
        .take(index_plus_footer)
        .collect::<io::Result<Vec<u8>>>()?;

    let uncompressed_size = unsafe {
        let mut i: MaybeUninit<*mut sys::lzma_index> = MaybeUninit::uninit();
        let mut memlimit = u64::MAX;
        let mut in_pos = 0usize;

        let ret = sys::lzma_index_buffer_decode(
            i.as_mut_ptr(),
            &mut memlimit,
            std::ptr::null(),
            buf.as_ptr(),
            &mut in_pos,
            buf.len(),
        );

        if ret != sys::LZMA_OK {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to parse lzma footer",
            ));
        }

        let i = i.assume_init();

        let uncompressed_size = sys::lzma_index_uncompressed_size(i);

        sys::lzma_index_end(i, std::ptr::null());

        uncompressed_size
    };

    Ok(uncompressed_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;
    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn all() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let e = encode_all(&v[..], 6).unwrap();
            let d = decode_all(&e[..]).unwrap();
            v == d
        }
    }

    #[test]
    fn copy() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let mut e = Vec::new();
            copy_encode(&v[..], &mut e, 6).unwrap();
            let mut d = Vec::new();
            copy_decode(&e[..], &mut d).unwrap();
            v == d
        }
    }

    #[test]
    fn size() {
        quickcheck(test as fn(_) -> _);

        fn test(v: Vec<u8>) -> bool {
            let mut e = Vec::new();
            copy_encode(&v[..], &mut e, 6).unwrap();

            let s = super::uncompressed_size(std::io::Cursor::new(e)).unwrap();

            (s as usize) == v.len()
        }
    }
}
