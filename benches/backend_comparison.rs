#![cfg(not(target_family = "wasm"))]

use std::ptr;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

#[cfg(any(
    all(feature = "xz-core", feature = "xz-sys"),
    all(feature = "xz-core", feature = "liblzma-sys"),
    all(feature = "xz-sys", feature = "liblzma-sys"),
))]
compile_error!("backend_comparison bench must be built with exactly one backend feature");
#[cfg(not(any(feature = "xz-core", feature = "xz-sys", feature = "liblzma-sys")))]
compile_error!("backend_comparison bench requires `xz-core`, `xz-sys`, or `liblzma-sys`");

#[cfg(feature = "liblzma-sys")]
use liblzma_sys::{
    lzma_crc32, lzma_crc64, lzma_easy_buffer_encode, lzma_stream_buffer_bound,
    lzma_stream_buffer_decode, LZMA_CHECK_CRC64, LZMA_OK,
};
#[cfg(feature = "xz-core")]
use xz_core::{
    common::{
        easy_buffer_encoder::lzma_easy_buffer_encode,
        stream_buffer_decoder::lzma_stream_buffer_decode,
        stream_buffer_encoder::lzma_stream_buffer_bound,
    },
    types::{LZMA_CHECK_CRC64, LZMA_OK},
};
#[cfg(feature = "xz-sys")]
use xz_sys::{
    lzma_crc32, lzma_crc64, lzma_easy_buffer_encode, lzma_stream_buffer_bound,
    lzma_stream_buffer_decode, LZMA_CHECK_CRC64, LZMA_OK,
};

#[cfg(feature = "xz-core")]
const BACKEND_NAME: &str = "xz-core";
#[cfg(feature = "liblzma-sys")]
const BACKEND_NAME: &str = "liblzma-sys";
#[cfg(feature = "xz-sys")]
const BACKEND_NAME: &str = "xz-sys";

/// xz-core takes the buffer as a slice; the other two backends take a pointer
/// and a size. These keep one shape for the shared benches below.
#[cfg(feature = "xz-core")]
unsafe fn lzma_crc32(buf: *const u8, size: usize, crc: u32) -> u32 {
    xz_core::check::crc32_fast::crc32(unsafe { core::slice::from_raw_parts(buf, size) }, crc)
}

#[cfg(feature = "xz-core")]
unsafe fn lzma_crc64(buf: *const u8, size: usize, crc: u64) -> u64 {
    xz_core::check::crc64_fast::crc64(unsafe { core::slice::from_raw_parts(buf, size) }, crc)
}

fn make_payload(size: usize) -> Vec<u8> {
    let mut x: u64 = 0x9E3779B97F4A7C15;
    let mut out = Vec::with_capacity(size);
    for _ in 0..size {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        out.push((x.wrapping_mul(0x2545F4914F6CDD1D) >> 56) as u8);
    }
    out
}

unsafe fn backend_encode(input: &[u8]) -> Vec<u8> {
    let bound = lzma_stream_buffer_bound(input.len());
    let mut out = vec![0u8; bound];
    let mut out_pos: usize = 0;
    let ret = lzma_easy_buffer_encode(
        6,
        LZMA_CHECK_CRC64,
        ptr::null(),
        input.as_ptr(),
        input.len(),
        out.as_mut_ptr(),
        &mut out_pos,
        out.len(),
    );
    assert_eq!(ret, LZMA_OK, "{BACKEND_NAME} encode failed with {ret}");
    out.truncate(out_pos);
    out
}

unsafe fn backend_decode(compressed: &[u8], out_size: usize) -> Vec<u8> {
    let mut out = vec![0u8; out_size];
    let mut memlimit = u64::MAX;
    let mut in_pos = 0usize;
    let mut out_pos = 0usize;
    let ret = lzma_stream_buffer_decode(
        &mut memlimit,
        0,
        ptr::null(),
        compressed.as_ptr(),
        &mut in_pos,
        compressed.len(),
        out.as_mut_ptr(),
        &mut out_pos,
        out.len(),
    );
    assert_eq!(ret, LZMA_OK, "{BACKEND_NAME} decode failed with {ret}");
    assert_eq!(
        in_pos,
        compressed.len(),
        "{BACKEND_NAME} decode left trailing input: consumed {in_pos} of {} bytes",
        compressed.len()
    );
    assert_eq!(
        out_pos, out_size,
        "{BACKEND_NAME} decode produced {out_pos} bytes, expected {out_size}"
    );
    out.truncate(out_pos);
    out
}

fn bench_encode(c: &mut Criterion) {
    let sizes: &[(usize, &str)] = &[(1024, "1KB"), (64 * 1024, "64KB"), (1024 * 1024, "1MB")];

    let mut group = c.benchmark_group("encode");
    for &(size, label) in sizes {
        let input = make_payload(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new(BACKEND_NAME, label), &input, |b, input| {
            b.iter(|| unsafe { backend_encode(black_box(input)) })
        });
    }
    group.finish();
}

fn bench_decode(c: &mut Criterion) {
    let sizes: &[(usize, &str)] = &[(1024, "1KB"), (64 * 1024, "64KB"), (1024 * 1024, "1MB")];

    let mut group = c.benchmark_group("decode");
    for &(size, label) in sizes {
        let input = make_payload(size);
        let compressed = unsafe { backend_encode(&input) };
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new(BACKEND_NAME, label),
            &compressed,
            |b, data| b.iter(|| unsafe { backend_decode(black_box(data), size) }),
        );
    }
    group.finish();
}

fn bench_crc32(c: &mut Criterion) {
    let sizes: &[(usize, &str)] = &[(1024, "1KB"), (64 * 1024, "64KB"), (1024 * 1024, "1MB")];

    let mut group = c.benchmark_group("crc32");
    for &(size, label) in sizes {
        let data = make_payload(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new(BACKEND_NAME, label), &data, |b, data| {
            b.iter(|| unsafe { lzma_crc32(black_box(data.as_ptr()), data.len(), 0) })
        });
    }
    group.finish();
}

fn bench_crc64(c: &mut Criterion) {
    let sizes: &[(usize, &str)] = &[(1024, "1KB"), (64 * 1024, "64KB"), (1024 * 1024, "1MB")];

    let mut group = c.benchmark_group("crc64");
    for &(size, label) in sizes {
        let data = make_payload(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new(BACKEND_NAME, label), &data, |b, data| {
            b.iter(|| unsafe { lzma_crc64(black_box(data.as_ptr()), data.len(), 0) })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_encode,
    bench_decode,
    bench_crc32,
    bench_crc64
);
criterion_main!(benches);
