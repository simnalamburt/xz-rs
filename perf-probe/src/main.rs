#![cfg(not(target_family = "wasm"))]

#[cfg(any(
    all(feature = "xz-core", feature = "xz-sys"),
    all(feature = "xz-core", feature = "liblzma-sys"),
    all(feature = "xz-sys", feature = "liblzma-sys"),
))]
compile_error!("Enable exactly one backend feature: xz-core, xz-sys, or liblzma-sys");
#[cfg(not(any(feature = "xz-core", feature = "xz-sys", feature = "liblzma-sys")))]
compile_error!("Enable one backend feature: xz-core, xz-sys, or liblzma-sys");

use std::env;
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::ptr;
use std::time::{Duration, Instant};

#[cfg(feature = "liblzma-sys")]
use liblzma_sys::{
    LZMA_CHECK_CRC64, LZMA_OK, LZMA_STREAM_HEADER_SIZE, lzma_crc32, lzma_crc64,
    lzma_easy_buffer_encode, lzma_index as BackendIndex, lzma_index_buffer_decode, lzma_index_end,
    lzma_index_uncompressed_size, lzma_stream_buffer_bound, lzma_stream_buffer_decode,
    lzma_stream_flags as BackendStreamFlags, lzma_stream_footer_decode,
};
#[cfg(feature = "xz-core-custom-allocator")]
use xz_core::alloc::{c_allocator, rust_allocator};
#[cfg(feature = "xz-core")]
use xz_core::common::{
    easy_buffer_encoder::lzma_easy_buffer_encode, index::lzma_index_end,
    index_decoder::lzma_index_buffer_decode, stream_buffer_decoder::lzma_stream_buffer_decode,
    stream_buffer_encoder::lzma_stream_buffer_bound,
};
#[cfg(feature = "xz-core")]
use xz_core::types::{
    LZMA_CHECK_CRC64, LZMA_OK, LZMA_STREAM_HEADER_SIZE, lzma_index as BackendIndex,
    lzma_stream_flags as BackendStreamFlags,
};
#[cfg(feature = "xz-sys")]
use xz_sys::{
    LZMA_CHECK_CRC64, LZMA_OK, LZMA_STREAM_HEADER_SIZE, lzma_crc32, lzma_crc64,
    lzma_easy_buffer_encode, lzma_index as BackendIndex, lzma_index_buffer_decode, lzma_index_end,
    lzma_index_uncompressed_size, lzma_stream_buffer_bound, lzma_stream_buffer_decode,
    lzma_stream_flags as BackendStreamFlags, lzma_stream_footer_decode,
};

#[cfg(feature = "xz-core")]
const BACKEND_NAME: &str = "xz-core";
#[cfg(feature = "liblzma-sys")]
const BACKEND_NAME: &str = "liblzma-sys";
#[cfg(feature = "xz-sys")]
const BACKEND_NAME: &str = "xz-sys";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Workload {
    Encode,
    Decode,
    Size,
    Crc32,
    Crc64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputKind {
    Random,
    Text,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AllocatorPolicy {
    Default,
    Rust,
    C,
}

#[derive(Debug)]
struct Config {
    workload: Workload,
    input_kind: InputKind,
    input_path: Option<PathBuf>,
    compressed_input_path: Option<PathBuf>,
    save_output_path: Option<PathBuf>,
    size: usize,
    chunk_size: Option<usize>,
    expected_size: Option<usize>,
    preset: u32,
    iters: usize,
    warmup: usize,
    allocator: AllocatorPolicy,
}

#[derive(Debug)]
struct Measurement {
    elapsed: Duration,
    cpu: Duration,
    throughput_mib_s: f64,
    digest: u64,
}

// Wall clock counts the time the process spends descheduled, so on a loaded
// machine it measures the rest of the system as much as the backend. Thread
// CPU time only advances while this thread is on a core, which keeps the
// backend comparison usable when the machine is busy. It still cannot tell a
// performance core from an efficiency one, so callers should compare minima
// over several rounds rather than single measurements.
#[cfg(unix)]
fn thread_cpu_time() -> Duration {
    let mut ts = std::mem::MaybeUninit::<libc::timespec>::zeroed();
    let ret = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, ts.as_mut_ptr()) };
    if ret != 0 {
        return Duration::ZERO;
    }
    let ts = unsafe { ts.assume_init() };
    Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
}

// Reported as zero rather than silently substituting the wall clock, so a
// platform without a thread CPU clock is visible in the output.
#[cfg(not(unix))]
fn thread_cpu_time() -> Duration {
    Duration::ZERO
}

fn main() {
    let config = Config::parse(env::args().skip(1)).unwrap_or_else(|message| {
        eprintln!("{message}");
        std::process::exit(2);
    });

    println!(
        "config: backend={} workload={:?} input={:?} size={} preset={} iters={} warmup={} allocator={:?}",
        BACKEND_NAME,
        config.workload,
        config.input_kind,
        config.size,
        config.preset,
        config.iters,
        config.warmup,
        config.allocator
    );
    if let Some(path) = &config.input_path {
        println!("input_path: {}", path.display());
    }
    if let Some(path) = &config.compressed_input_path {
        println!("compressed_input_path: {}", path.display());
    }
    if let Some(path) = &config.save_output_path {
        println!("save_output_path: {}", path.display());
    }

    match config.workload {
        Workload::Encode => run_encode(&config),
        Workload::Decode => run_decode(&config),
        Workload::Size => run_size(&config),
        Workload::Crc32 => run_crc32(&config),
        Workload::Crc64 => run_crc64(&config),
    }
}

impl Config {
    fn parse<I>(mut args: I) -> Result<Self, String>
    where
        I: Iterator<Item = String>,
    {
        let mut config = Self {
            workload: Workload::Encode,
            input_kind: InputKind::Text,
            input_path: None,
            compressed_input_path: None,
            save_output_path: None,
            size: 1024 * 1024,
            chunk_size: None,
            expected_size: None,
            preset: 6,
            iters: 200,
            warmup: 20,
            allocator: AllocatorPolicy::Default,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--workload" => {
                    config.workload = parse_workload(next_arg(&mut args, "--workload")?)?;
                }
                "--input-kind" => {
                    config.input_kind = parse_input_kind(next_arg(&mut args, "--input-kind")?)?;
                }
                "--input" => {
                    config.input_path = Some(PathBuf::from(next_arg(&mut args, "--input")?));
                }
                "--compressed-input" => {
                    config.compressed_input_path =
                        Some(PathBuf::from(next_arg(&mut args, "--compressed-input")?));
                }
                "--save-output" => {
                    config.save_output_path =
                        Some(PathBuf::from(next_arg(&mut args, "--save-output")?));
                }
                "--size" => {
                    config.size = parse_usize(next_arg(&mut args, "--size")?, "--size")?;
                }
                "--chunk-size" => {
                    config.chunk_size = Some(parse_usize(
                        next_arg(&mut args, "--chunk-size")?,
                        "--chunk-size",
                    )?);
                }
                "--expected-size" => {
                    config.expected_size = Some(parse_usize(
                        next_arg(&mut args, "--expected-size")?,
                        "--expected-size",
                    )?);
                }
                "--preset" => {
                    config.preset = parse_u32(next_arg(&mut args, "--preset")?, "--preset")?;
                }
                "--iters" => {
                    config.iters = parse_usize(next_arg(&mut args, "--iters")?, "--iters")?;
                }
                "--warmup" => {
                    config.warmup = parse_usize(next_arg(&mut args, "--warmup")?, "--warmup")?;
                }
                "--allocator" => {
                    config.allocator = parse_allocator(next_arg(&mut args, "--allocator")?)?;
                }
                "--help" | "-h" => return Err(usage()),
                unknown => return Err(format!("unknown argument `{unknown}`\n\n{}", usage())),
            }
        }

        if config.iters == 0 {
            return Err("`--iters` must be greater than zero".to_owned());
        }

        if matches!(config.chunk_size, Some(0)) {
            return Err("`--chunk-size` must be greater than zero".to_owned());
        }

        if matches!(config.workload, Workload::Decode | Workload::Size)
            && config.compressed_input_path.is_some()
            && config.expected_size.is_none()
        {
            return Err(
                "`decode`/`size` with `--compressed-input` also requires `--expected-size`"
                    .to_owned(),
            );
        }

        if config.chunk_size.is_some()
            && !matches!(config.workload, Workload::Crc32 | Workload::Crc64)
        {
            return Err("`--chunk-size` is only supported with crc32/crc64 workloads".to_owned());
        }

        #[cfg(not(feature = "xz-core-custom-allocator"))]
        if config.allocator != AllocatorPolicy::Default {
            return Err("`--allocator` requires `--features xz-core-custom-allocator`".to_owned());
        }

        Ok(config)
    }
}

fn usage() -> String {
    let mut message = String::new();
    message.push_str("Usage:\n");
    message.push_str(
        "  cargo run -p perf-probe --release --no-default-features --features <xz-core|xz-sys|liblzma-sys> -- \\\n",
    );
    message.push_str("    --workload <encode|decode|size|crc32|crc64> [options]\n\n");
    message.push_str("Options:\n");
    message.push_str("  --input <path>              Read raw workload bytes from a file\n");
    message.push_str("  --compressed-input <path>   Read compressed bytes for decode\n");
    message.push_str("  --save-output <path>        Save encode/decode output bytes\n");
    message.push_str("  --input-kind <text|random>\n");
    message
        .push_str("  --size <bytes>              Synthetic input size when --input is omitted\n");
    message.push_str(
        "  --chunk-size <bytes>        Split crc32/crc64 input into incremental chunks\n",
    );
    message.push_str(
        "  --expected-size <bytes>     Required for decode when --compressed-input is used\n",
    );
    message.push_str("  --preset <0-9>              LZMA preset used by encode/decode prep\n");
    message.push_str("  --iters <n>                 Timed iterations\n");
    message.push_str("  --warmup <n>                Untimed warmup iterations\n");
    message.push_str("  --allocator <default|rust|c> xz-core custom_allocator policy probe only\n");
    message
}

fn next_arg<I>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("missing value for `{flag}`\n\n{}", usage()))
}

fn parse_workload(value: String) -> Result<Workload, String> {
    match value.as_str() {
        "encode" => Ok(Workload::Encode),
        "decode" => Ok(Workload::Decode),
        "size" => Ok(Workload::Size),
        "crc32" => Ok(Workload::Crc32),
        "crc64" => Ok(Workload::Crc64),
        _ => Err(format!("unsupported workload `{value}`")),
    }
}

fn parse_input_kind(value: String) -> Result<InputKind, String> {
    match value.as_str() {
        "random" => Ok(InputKind::Random),
        "text" => Ok(InputKind::Text),
        _ => Err(format!("unsupported input kind `{value}`")),
    }
}

fn parse_allocator(value: String) -> Result<AllocatorPolicy, String> {
    match value.as_str() {
        "default" => Ok(AllocatorPolicy::Default),
        "rust" => Ok(AllocatorPolicy::Rust),
        "c" => Ok(AllocatorPolicy::C),
        _ => Err(format!("unsupported allocator `{value}`")),
    }
}

fn parse_usize(value: String, flag: &str) -> Result<usize, String> {
    value
        .parse()
        .map_err(|_| format!("failed to parse `{flag}` as usize: `{value}`"))
}

fn parse_u32(value: String, flag: &str) -> Result<u32, String> {
    value
        .parse()
        .map_err(|_| format!("failed to parse `{flag}` as u32: `{value}`"))
}

fn load_raw_input(config: &Config) -> Result<Vec<u8>, std::io::Error> {
    match &config.input_path {
        Some(path) => fs::read(path),
        None => Ok(match config.input_kind {
            InputKind::Random => make_random_payload(config.size),
            InputKind::Text => make_text_payload(config.size),
        }),
    }
}

fn load_compressed_input(config: &Config) -> Result<(Vec<u8>, usize), std::io::Error> {
    match &config.compressed_input_path {
        Some(path) => Ok((
            fs::read(path)?,
            config
                .expected_size
                .expect("expected_size checked during argument parsing"),
        )),
        None => {
            let raw = load_raw_input(config)?;
            let compressed = unsafe { backend_encode(&raw, config.preset, config.allocator) };
            Ok((compressed, raw.len()))
        }
    }
}

fn make_random_payload(size: usize) -> Vec<u8> {
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

fn make_text_payload(size: usize) -> Vec<u8> {
    let chunk = b"Portable Network Archive keeps long-tail formats maintainable by replacing risky C dependency paths with inspectable Rust implementations. ";
    let mut out = Vec::with_capacity(size);
    while out.len() < size {
        let remaining = size - out.len();
        out.extend_from_slice(&chunk[..remaining.min(chunk.len())]);
    }
    out
}

fn run_encode(config: &Config) {
    let input = load_raw_input(config).unwrap_or_else(|err| {
        eprintln!("failed to load input: {err}");
        std::process::exit(1);
    });

    let first = unsafe { backend_encode(&input, config.preset, config.allocator) };
    if let Some(path) = &config.save_output_path {
        fs::write(path, &first).unwrap_or_else(|err| {
            eprintln!("failed to write encoded output: {err}");
            std::process::exit(1);
        });
    }

    let measurement = measure(input.len(), config.iters, config.warmup, || unsafe {
        let output = backend_encode(&input, config.preset, config.allocator);
        fold_bytes(output.len(), &output)
    });
    print_measurement(&measurement, config.iters);
}

fn run_decode(config: &Config) {
    let (compressed, expected_size) = load_compressed_input(config).unwrap_or_else(|err| {
        eprintln!("failed to load compressed input: {err}");
        std::process::exit(1);
    });

    let first = unsafe { backend_decode(&compressed, expected_size) };
    if let Some(path) = &config.save_output_path {
        fs::write(path, &first).unwrap_or_else(|err| {
            eprintln!("failed to write decoded output: {err}");
            std::process::exit(1);
        });
    }
    println!(
        "decode_input: compressed_size={} original_size={}",
        compressed.len(),
        expected_size
    );

    let measurement = measure(expected_size, config.iters, config.warmup, || unsafe {
        let output = backend_decode(&compressed, expected_size);
        fold_bytes(output.len(), &output)
    });
    print_measurement(&measurement, config.iters);
}

fn run_size(config: &Config) {
    let (compressed, expected_size) = load_compressed_input(config).unwrap_or_else(|err| {
        eprintln!("failed to load compressed input: {err}");
        std::process::exit(1);
    });
    println!(
        "size_input: compressed_size={} original_size={}",
        compressed.len(),
        expected_size
    );

    let first = unsafe { backend_uncompressed_size(&compressed) };
    assert_eq!(
        first, expected_size as u64,
        "{BACKEND_NAME} size failed: expected {} got {}",
        expected_size, first
    );

    let measurement = measure(compressed.len(), config.iters, config.warmup, || unsafe {
        let input = black_box(compressed.as_slice());
        backend_uncompressed_size(input)
    });
    print_measurement(&measurement, config.iters);
}

fn run_crc32(config: &Config) {
    let input = load_raw_input(config).unwrap_or_else(|err| {
        eprintln!("failed to load input: {err}");
        std::process::exit(1);
    });
    let measurement = measure(input.len(), config.iters, config.warmup, || {
        let slice = black_box(input.as_slice());
        let mut crc = 0u32;
        if let Some(chunk_size) = config.chunk_size {
            for chunk in slice.chunks(chunk_size) {
                crc = unsafe { lzma_crc32(chunk.as_ptr(), chunk.len(), crc) };
            }
        } else {
            crc = unsafe { lzma_crc32(slice.as_ptr(), slice.len(), crc) };
        }
        crc as u64
    });
    print_measurement(&measurement, config.iters);
}

fn run_crc64(config: &Config) {
    let input = load_raw_input(config).unwrap_or_else(|err| {
        eprintln!("failed to load input: {err}");
        std::process::exit(1);
    });
    let measurement = measure(input.len(), config.iters, config.warmup, || {
        let slice = black_box(input.as_slice());
        let mut crc = 0u64;
        if let Some(chunk_size) = config.chunk_size {
            for chunk in slice.chunks(chunk_size) {
                crc = unsafe { lzma_crc64(chunk.as_ptr(), chunk.len(), crc) };
            }
        } else {
            crc = unsafe { lzma_crc64(slice.as_ptr(), slice.len(), crc) };
        }
        crc
    });
    print_measurement(&measurement, config.iters);
}

fn measure<F>(bytes_per_iter: usize, iters: usize, warmup: usize, mut work: F) -> Measurement
where
    F: FnMut() -> u64,
{
    let mut digest = 0u64;
    for _ in 0..warmup {
        digest ^= black_box(work());
    }

    let cpu_start = thread_cpu_time();
    let start = Instant::now();
    for _ in 0..iters {
        digest ^= black_box(work());
    }
    let elapsed = start.elapsed();
    let cpu = thread_cpu_time().saturating_sub(cpu_start);
    let mib = (bytes_per_iter as f64 * iters as f64) / (1024.0 * 1024.0);

    Measurement {
        elapsed,
        cpu,
        throughput_mib_s: mib / elapsed.as_secs_f64(),
        digest,
    }
}

fn print_measurement(measurement: &Measurement, iters: usize) {
    println!(
        "{}: total={:.3?} ns_per_iter={:.0} cpu_ns_per_iter={:.0} throughput_mib_s={:.2} digest={:#x}",
        BACKEND_NAME,
        measurement.elapsed,
        measurement.elapsed.as_nanos() as f64 / iters as f64,
        measurement.cpu.as_nanos() as f64 / iters as f64,
        measurement.throughput_mib_s,
        measurement.digest
    );
}

fn fold_bytes(len: usize, data: &[u8]) -> u64 {
    let mut acc = len as u64;
    for &byte in data.iter().take(32) {
        acc = acc.rotate_left(5) ^ u64::from(byte);
    }
    acc
}

unsafe fn backend_encode(input: &[u8], preset: u32, allocator: AllocatorPolicy) -> Vec<u8> {
    #[cfg(feature = "xz-core")]
    let bound = lzma_stream_buffer_bound(input.len());
    #[cfg(any(feature = "xz-sys", feature = "liblzma-sys"))]
    let bound = unsafe { lzma_stream_buffer_bound(input.len()) };
    let mut out = vec![0u8; bound];
    let mut out_pos: usize = 0;

    #[cfg(feature = "xz-core-custom-allocator")]
    let allocator_storage;
    #[cfg(feature = "xz-core-custom-allocator")]
    let allocator_ptr = match allocator {
        AllocatorPolicy::Default => ptr::null(),
        AllocatorPolicy::Rust => {
            allocator_storage = rust_allocator();
            &allocator_storage
        }
        AllocatorPolicy::C => {
            allocator_storage = c_allocator();
            &allocator_storage
        }
    };

    #[cfg(not(feature = "xz-core-custom-allocator"))]
    let allocator_ptr = {
        debug_assert_eq!(allocator, AllocatorPolicy::Default);
        ptr::null()
    };

    let ret = unsafe {
        lzma_easy_buffer_encode(
            preset,
            LZMA_CHECK_CRC64,
            allocator_ptr,
            input.as_ptr(),
            input.len(),
            out.as_mut_ptr(),
            &mut out_pos,
            out.len(),
        )
    };
    assert_eq!(ret, LZMA_OK, "{BACKEND_NAME} encode failed with {ret}");
    out.truncate(out_pos);
    out
}

unsafe fn backend_decode(compressed: &[u8], out_size: usize) -> Vec<u8> {
    let mut out = vec![0u8; out_size];
    let mut memlimit = u64::MAX;
    let mut in_pos = 0usize;
    let mut out_pos = 0usize;
    let ret = unsafe {
        lzma_stream_buffer_decode(
            &mut memlimit,
            0,
            ptr::null(),
            compressed.as_ptr(),
            &mut in_pos,
            compressed.len(),
            out.as_mut_ptr(),
            &mut out_pos,
            out.len(),
        )
    };
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

/// xz-core takes the buffer as a slice; the other two backends take a pointer
/// and a size. These keep one shape for the shared code below.
#[cfg(feature = "xz-core")]
unsafe fn lzma_crc32(buf: *const u8, size: usize, crc: u32) -> u32 {
    xz_core::check::crc32_fast::crc32(unsafe { core::slice::from_raw_parts(buf, size) }, crc)
}

#[cfg(feature = "xz-core")]
unsafe fn lzma_crc64(buf: *const u8, size: usize, crc: u64) -> u64 {
    xz_core::check::crc64_fast::crc64(unsafe { core::slice::from_raw_parts(buf, size) }, crc)
}

/// xz-core takes the Index by reference; the other two backends take a bare
/// pointer. This keeps one shape for the shared code below.
#[cfg(feature = "xz-core")]
unsafe fn lzma_index_uncompressed_size(i: *const BackendIndex) -> u64 {
    unsafe { xz_core::common::index::lzma_index_uncompressed_size(&*i) }
}

/// xz-core states the twelve-byte Stream Footer in the type; the other two
/// backends take a bare pointer. This keeps one shape for the shared code below.
#[cfg(feature = "xz-core")]
unsafe fn lzma_stream_footer_decode(
    options: *mut BackendStreamFlags,
    input: *const u8,
) -> xz_core::types::lzma_ret {
    unsafe {
        xz_core::common::stream_flags_decoder::lzma_stream_footer_decode(
            &mut *options,
            &*input.cast(),
        )
    }
}

unsafe fn backend_uncompressed_size(compressed: &[u8]) -> u64 {
    let footer_len = LZMA_STREAM_HEADER_SIZE as usize;
    let footer_offset = compressed
        .len()
        .checked_sub(footer_len)
        .expect("compressed payload must contain at least a footer");
    let footer = compressed[footer_offset..].as_ptr();

    let mut footer_flags = std::mem::MaybeUninit::<BackendStreamFlags>::uninit();
    let footer_ret = unsafe { lzma_stream_footer_decode(footer_flags.as_mut_ptr(), footer) };
    assert_eq!(
        footer_ret, LZMA_OK,
        "{BACKEND_NAME} stream_footer_decode failed with {footer_ret}"
    );
    let footer_flags = unsafe { footer_flags.assume_init() };

    let index_plus_footer = footer_len + footer_flags.backward_size as usize;
    let mut index_pos = 0usize;
    let mut memlimit = u64::MAX;
    let mut index = std::mem::MaybeUninit::<*mut BackendIndex>::uninit();
    let index_bytes = &compressed[compressed.len() - index_plus_footer..];
    let index_ret = unsafe {
        lzma_index_buffer_decode(
            index.as_mut_ptr(),
            &mut memlimit,
            ptr::null(),
            index_bytes.as_ptr(),
            &mut index_pos,
            index_bytes.len(),
        )
    };
    assert_eq!(
        index_ret, LZMA_OK,
        "{BACKEND_NAME} index_buffer_decode failed with {index_ret}"
    );
    let index = unsafe { index.assume_init() };
    let size = unsafe { lzma_index_uncompressed_size(index) };
    unsafe { lzma_index_end(index, ptr::null()) };
    size
}
