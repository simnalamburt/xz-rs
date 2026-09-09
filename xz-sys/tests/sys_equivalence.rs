#![cfg(not(target_family = "wasm"))]

use std::collections::BTreeMap;
use std::fs;
use std::mem;
use std::path::Path;
use std::ptr;

fn parse_feature_table(cargo_toml: &str) -> BTreeMap<String, Vec<String>> {
    let mut in_features = false;
    let mut features = BTreeMap::new();

    for raw_line in cargo_toml.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_features = line == "[features]";
            continue;
        }
        if !in_features || line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let (name, rhs) = line
            .split_once('=')
            .expect("feature entry must contain '='");
        let name = name.trim().to_owned();
        let rhs = rhs.trim();
        let mut deps = Vec::new();

        if rhs != "[]" {
            assert!(
                rhs.starts_with('[') && rhs.ends_with(']'),
                "feature value must be an array: {name} = {rhs}",
            );
            let inner = &rhs[1..rhs.len() - 1];
            for item in inner.split(',') {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                let item = item
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .unwrap_or_else(|| panic!("feature dependency must be quoted: {item}"));
                deps.push(item.to_owned());
            }
        }

        features.insert(name, deps);
    }

    features
}

#[test]
fn rs_sys_avoids_literal_lzma_const_defs() {
    let src = include_str!("../src/lib.rs");
    let mut stmt = String::new();
    let mut start_line = 0usize;
    let mut in_const = false;

    for (idx, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        if !in_const && trimmed.starts_with("pub const LZMA_") {
            in_const = true;
            start_line = idx + 1;
            stmt.clear();
        }

        if in_const {
            stmt.push_str(trimmed);
            stmt.push(' ');
            if trimmed.ends_with(';') {
                assert!(
                    stmt.contains("= xz_core::"),
                    "LZMA const must alias xz-core symbol, not use local literal (line {}): {}",
                    start_line,
                    stmt
                );
                in_const = false;
            }
        }
    }
}

#[test]
fn rs_sys_uses_libc_size_t() {
    let src = include_str!("../src/lib.rs");
    assert!(
        src.contains("use libc::size_t;"),
        "xz-sys must import libc::size_t directly"
    );
    assert!(
        !src.contains("type size_t ="),
        "local size_t alias is forbidden; use libc::size_t"
    );
}

#[test]
fn rs_sys_exports_required_public_wrappers() {
    let src = include_str!("../src/lib.rs");

    for name in [
        "lzma_alloc",
        "lzma_alloc_zero",
        "lzma_free",
        "lzma_bcj_arm64_encode",
        "lzma_bcj_arm64_decode",
        "lzma_bcj_riscv_encode",
        "lzma_bcj_riscv_decode",
        "lzma_bcj_x86_encode",
        "lzma_bcj_x86_decode",
    ] {
        assert!(
            src.contains(&format!("fn {name}(")),
            "xz-sys must export wrapper for {name}",
        );
    }
}

#[test]
fn xz_sys_keeps_liblzma_sys_compatible_features() {
    let rs_sys_features = parse_feature_table(include_str!("../Cargo.toml"));
    // Keep this list aligned with liblzma-sys 0.4.6 so xz-sys remains a
    // drop-in feature-compatible replacement for consumers.
    let expected = BTreeMap::from([
        ("bindgen".to_string(), Vec::new()),
        ("default".to_string(), vec!["bindgen".to_string()]),
        ("fat-lto".to_string(), Vec::new()),
        ("parallel".to_string(), Vec::new()),
        ("static".to_string(), Vec::new()),
        ("thin-lto".to_string(), Vec::new()),
        ("uncheck_liblzma_version".to_string(), Vec::new()),
        (
            "wasm".to_string(),
            vec!["static".to_string(), "bindgen".to_string()],
        ),
    ]);

    assert_eq!(
        rs_sys_features, expected,
        "xz-sys feature table must stay compatible with liblzma-sys feature names",
    );
}

#[test]
fn api_constants_match_c_backend() {
    macro_rules! assert_const_eq {
        ($($name:ident),+ $(,)?) => {
            $(
                assert_eq!(
                    xz_sys::$name as u128,
                    liblzma_sys::$name as u128,
                    "constant mismatch: {}",
                    stringify!($name)
                );
            )+
        };
    }

    assert_const_eq!(
        LZMA_VERSION_MAJOR,
        LZMA_VERSION_MINOR,
        LZMA_VERSION_PATCH,
        LZMA_OK,
        LZMA_STREAM_END,
        LZMA_NO_CHECK,
        LZMA_UNSUPPORTED_CHECK,
        LZMA_GET_CHECK,
        LZMA_MEM_ERROR,
        LZMA_MEMLIMIT_ERROR,
        LZMA_FORMAT_ERROR,
        LZMA_OPTIONS_ERROR,
        LZMA_DATA_ERROR,
        LZMA_BUF_ERROR,
        LZMA_PROG_ERROR,
        LZMA_SEEK_NEEDED,
        LZMA_RUN,
        LZMA_SYNC_FLUSH,
        LZMA_FULL_FLUSH,
        LZMA_FULL_BARRIER,
        LZMA_FINISH,
        LZMA_CHECK_NONE,
        LZMA_CHECK_CRC32,
        LZMA_CHECK_CRC64,
        LZMA_CHECK_SHA256,
        LZMA_MODE_FAST,
        LZMA_MODE_NORMAL,
        LZMA_MF_HC3,
        LZMA_MF_HC4,
        LZMA_MF_BT2,
        LZMA_MF_BT3,
        LZMA_MF_BT4,
        LZMA_TELL_NO_CHECK,
        LZMA_TELL_UNSUPPORTED_CHECK,
        LZMA_TELL_ANY_CHECK,
        LZMA_IGNORE_CHECK,
        LZMA_CONCATENATED,
        LZMA_PRESET_DEFAULT,
        LZMA_PRESET_LEVEL_MASK,
        LZMA_PRESET_EXTREME,
        LZMA_DICT_SIZE_MIN,
        LZMA_DICT_SIZE_DEFAULT,
        LZMA_LCLP_MIN,
        LZMA_LCLP_MAX,
        LZMA_LC_DEFAULT,
        LZMA_LP_DEFAULT,
        LZMA_PB_MIN,
        LZMA_PB_MAX,
        LZMA_PB_DEFAULT,
        LZMA_BACKWARD_SIZE_MIN,
        LZMA_BACKWARD_SIZE_MAX,
        LZMA_VLI_MAX,
        LZMA_VLI_UNKNOWN,
        LZMA_VLI_BYTES_MAX,
        LZMA_CHECK_ID_MAX,
        LZMA_CHECK_SIZE_MAX,
        LZMA_FILTERS_MAX,
        LZMA_DELTA_DIST_MIN,
        LZMA_DELTA_DIST_MAX,
        LZMA_FILTER_X86,
        LZMA_FILTER_POWERPC,
        LZMA_FILTER_IA64,
        LZMA_FILTER_ARM,
        LZMA_FILTER_ARMTHUMB,
        LZMA_FILTER_SPARC,
        LZMA_FILTER_ARM64,
        LZMA_FILTER_DELTA,
        LZMA_FILTER_RISCV,
        LZMA_FILTER_LZMA1,
        LZMA_FILTER_LZMA2,
        LZMA_STREAM_HEADER_SIZE,
        LZMA_BLOCK_HEADER_SIZE_MIN,
        LZMA_BLOCK_HEADER_SIZE_MAX,
    );

    assert_eq!(
        xz_sys::LZMA_DELTA_TYPE_BYTE as u128,
        liblzma_sys::lzma_delta_type_LZMA_DELTA_TYPE_BYTE as u128,
        "constant mismatch: LZMA_DELTA_TYPE_BYTE",
    );
}

#[test]
fn api_constant_types_match_c_backend() {
    fn assert_same_type<T>(_: T, _: T) {}

    macro_rules! assert_const_type_eq {
        ($($name:ident),+ $(,)?) => {
            $(
                assert_same_type(liblzma_sys::$name, xz_sys::$name);
            )+
        };
    }

    assert_const_type_eq!(
        LZMA_VERSION_MAJOR,
        LZMA_VERSION_MINOR,
        LZMA_VERSION_PATCH,
        LZMA_OK,
        LZMA_STREAM_END,
        LZMA_NO_CHECK,
        LZMA_UNSUPPORTED_CHECK,
        LZMA_GET_CHECK,
        LZMA_MEM_ERROR,
        LZMA_MEMLIMIT_ERROR,
        LZMA_FORMAT_ERROR,
        LZMA_OPTIONS_ERROR,
        LZMA_DATA_ERROR,
        LZMA_BUF_ERROR,
        LZMA_PROG_ERROR,
        LZMA_SEEK_NEEDED,
        LZMA_RUN,
        LZMA_SYNC_FLUSH,
        LZMA_FULL_FLUSH,
        LZMA_FULL_BARRIER,
        LZMA_FINISH,
        LZMA_CHECK_NONE,
        LZMA_CHECK_CRC32,
        LZMA_CHECK_CRC64,
        LZMA_CHECK_SHA256,
        LZMA_MODE_FAST,
        LZMA_MODE_NORMAL,
        LZMA_MF_HC3,
        LZMA_MF_HC4,
        LZMA_MF_BT2,
        LZMA_MF_BT3,
        LZMA_MF_BT4,
        LZMA_TELL_NO_CHECK,
        LZMA_TELL_UNSUPPORTED_CHECK,
        LZMA_TELL_ANY_CHECK,
        LZMA_IGNORE_CHECK,
        LZMA_CONCATENATED,
        LZMA_PRESET_DEFAULT,
        LZMA_PRESET_LEVEL_MASK,
        LZMA_PRESET_EXTREME,
        LZMA_DICT_SIZE_MIN,
        LZMA_DICT_SIZE_DEFAULT,
        LZMA_LCLP_MIN,
        LZMA_LCLP_MAX,
        LZMA_LC_DEFAULT,
        LZMA_LP_DEFAULT,
        LZMA_PB_MIN,
        LZMA_PB_MAX,
        LZMA_PB_DEFAULT,
        LZMA_BACKWARD_SIZE_MIN,
        LZMA_BACKWARD_SIZE_MAX,
        LZMA_VLI_MAX,
        LZMA_VLI_UNKNOWN,
        LZMA_VLI_BYTES_MAX,
        LZMA_CHECK_ID_MAX,
        LZMA_CHECK_SIZE_MAX,
        LZMA_FILTERS_MAX,
        LZMA_DELTA_DIST_MIN,
        LZMA_DELTA_DIST_MAX,
        LZMA_FILTER_X86,
        LZMA_FILTER_POWERPC,
        LZMA_FILTER_IA64,
        LZMA_FILTER_ARM,
        LZMA_FILTER_ARMTHUMB,
        LZMA_FILTER_SPARC,
        LZMA_FILTER_ARM64,
        LZMA_FILTER_DELTA,
        LZMA_FILTER_RISCV,
        LZMA_FILTER_LZMA1,
        LZMA_FILTER_LZMA2,
        LZMA_STREAM_HEADER_SIZE,
        LZMA_BLOCK_HEADER_SIZE_MIN,
        LZMA_BLOCK_HEADER_SIZE_MAX,
    );

    assert_same_type(
        liblzma_sys::lzma_delta_type_LZMA_DELTA_TYPE_BYTE,
        xz_sys::LZMA_DELTA_TYPE_BYTE,
    );
}

#[test]
fn api_type_layout_matches_c_backend() {
    macro_rules! assert_layout_eq {
        ($name:ident) => {
            assert_eq!(
                std::mem::size_of::<xz_sys::$name>(),
                std::mem::size_of::<liblzma_sys::$name>(),
                "size mismatch: {}",
                stringify!($name)
            );
            assert_eq!(
                std::mem::align_of::<xz_sys::$name>(),
                std::mem::align_of::<liblzma_sys::$name>(),
                "align mismatch: {}",
                stringify!($name)
            );
        };
    }

    assert_layout_eq!(lzma_bool);
    assert_layout_eq!(lzma_ret);
    assert_layout_eq!(lzma_action);
    assert_layout_eq!(lzma_check);
    assert_layout_eq!(lzma_vli);
    assert_layout_eq!(lzma_delta_type);
    assert_layout_eq!(lzma_mode);
    assert_layout_eq!(lzma_match_finder);
    assert_layout_eq!(lzma_allocator);
    assert_layout_eq!(lzma_stream);
    assert_layout_eq!(lzma_filter);
    assert_layout_eq!(lzma_options_lzma);
    assert_layout_eq!(lzma_options_delta);
    assert_layout_eq!(lzma_stream_flags);
    assert_layout_eq!(lzma_options_bcj);

    #[cfg(feature = "parallel")]
    assert_layout_eq!(lzma_mt);

    // Opaque types can have intentionally different concrete representations.
    let _: *mut xz_sys::lzma_internal = std::ptr::null_mut();
    let _: *mut liblzma_sys::lzma_internal = std::ptr::null_mut();
    let _: *mut xz_sys::lzma_index = std::ptr::null_mut();
    let _: *mut liblzma_sys::lzma_index = std::ptr::null_mut();
}

#[test]
fn api_functions_are_exported() {
    macro_rules! assert_fn_exported {
        ($($name:ident),+ $(,)?) => {
            $(
                let _ = liblzma_sys::$name as *const () as usize;
                let _ = xz_sys::$name as *const () as usize;
            )+
        };
    }

    assert_fn_exported!(
        lzma_alone_decoder,
        lzma_alone_encoder,
        lzma_auto_decoder,
        lzma_bcj_arm64_decode,
        lzma_bcj_arm64_encode,
        lzma_bcj_riscv_decode,
        lzma_bcj_riscv_encode,
        lzma_bcj_x86_decode,
        lzma_bcj_x86_encode,
        lzma_block_buffer_bound,
        lzma_block_buffer_decode,
        lzma_block_buffer_encode,
        lzma_block_compressed_size,
        lzma_block_decoder,
        lzma_block_encoder,
        lzma_block_header_decode,
        lzma_block_header_encode,
        lzma_block_header_size,
        lzma_block_total_size,
        lzma_block_uncomp_encode,
        lzma_block_unpadded_size,
        lzma_check_is_supported,
        lzma_check_size,
        lzma_code,
        lzma_cputhreads,
        lzma_crc32,
        lzma_crc64,
        lzma_easy_buffer_encode,
        lzma_easy_decoder_memusage,
        lzma_easy_encoder,
        lzma_easy_encoder_memusage,
        lzma_end,
        lzma_file_info_decoder,
        lzma_filter_decoder_is_supported,
        lzma_filter_encoder_is_supported,
        lzma_filter_flags_decode,
        lzma_filter_flags_encode,
        lzma_filter_flags_size,
        lzma_filters_copy,
        lzma_filters_free,
        lzma_filters_update,
        lzma_get_check,
        lzma_get_progress,
        lzma_index_append,
        lzma_index_block_count,
        lzma_index_buffer_decode,
        lzma_index_buffer_encode,
        lzma_index_cat,
        lzma_index_checks,
        lzma_index_decoder,
        lzma_index_dup,
        lzma_index_encoder,
        lzma_index_end,
        lzma_index_file_size,
        lzma_index_hash_append,
        lzma_index_hash_decode,
        lzma_index_hash_end,
        lzma_index_hash_init,
        lzma_index_hash_size,
        lzma_index_init,
        lzma_index_iter_init,
        lzma_index_iter_locate,
        lzma_index_iter_next,
        lzma_index_iter_rewind,
        lzma_index_memusage,
        lzma_index_memused,
        lzma_index_size,
        lzma_index_stream_count,
        lzma_index_stream_flags,
        lzma_index_stream_padding,
        lzma_index_stream_size,
        lzma_index_total_size,
        lzma_index_uncompressed_size,
        lzma_lzip_decoder,
        lzma_lzma_preset,
        lzma_memlimit_get,
        lzma_memlimit_set,
        lzma_memusage,
        lzma_mf_is_supported,
        lzma_microlzma_decoder,
        lzma_microlzma_encoder,
        lzma_mode_is_supported,
        lzma_mt_block_size,
        lzma_physmem,
        lzma_properties_decode,
        lzma_properties_encode,
        lzma_properties_size,
        lzma_raw_buffer_decode,
        lzma_raw_buffer_encode,
        lzma_raw_decoder,
        lzma_raw_decoder_memusage,
        lzma_raw_encoder,
        lzma_raw_encoder_memusage,
        lzma_str_from_filters,
        lzma_str_list_filters,
        lzma_str_to_filters,
        lzma_stream_buffer_bound,
        lzma_stream_buffer_decode,
        lzma_stream_buffer_encode,
        lzma_stream_decoder,
        lzma_stream_encoder,
        lzma_stream_flags_compare,
        lzma_stream_footer_decode,
        lzma_stream_footer_encode,
        lzma_stream_header_decode,
        lzma_stream_header_encode,
        lzma_version_number,
        lzma_version_string,
        lzma_vli_decode,
        lzma_vli_encode,
        lzma_vli_size,
    );

    // The multithreaded entry points exist only when xz-sys is built with the
    // parallel feature, so they are compared in a separate invocation.
    #[cfg(feature = "parallel")]
    assert_fn_exported!(
        lzma_stream_decoder_mt,
        lzma_stream_encoder_mt,
        lzma_stream_encoder_mt_memusage,
    );
}

macro_rules! encode_easy_impl {
    ($sys:ident, $input:expr) => {{
        let bound = $sys::lzma_stream_buffer_bound($input.len());
        let mut out = vec![0u8; bound];
        let mut out_pos: usize = 0;
        let ret = $sys::lzma_easy_buffer_encode(
            6,
            $sys::LZMA_CHECK_CRC64,
            ptr::null(),
            $input.as_ptr(),
            $input.len(),
            out.as_mut_ptr(),
            &mut out_pos,
            out.len(),
        );
        out.truncate(out_pos);
        (ret as u32, out)
    }};
}

macro_rules! decode_stream_impl {
    ($sys:ident, $input:expr, $expected_size_hint:expr) => {{
        let mut out = Vec::with_capacity($expected_size_hint.max(256));
        let mut stream: $sys::lzma_stream = unsafe { mem::zeroed() };
        let mut ret = $sys::lzma_stream_decoder(&mut stream, u64::MAX, 0);
        if ret as u32 != $sys::LZMA_OK as u32 {
            return (ret as u32, out);
        }

        loop {
            if out.spare_capacity_mut().is_empty() {
                let additional = out.capacity().max(256);
                out.reserve(additional);
            }

            let spare = out.spare_capacity_mut();
            stream.next_in = $input.as_ptr();
            stream.avail_in = $input.len() - stream.total_in as usize;
            stream.next_out = spare.as_ptr() as *mut u8;
            stream.avail_out = spare.len();

            ret = $sys::lzma_code(&mut stream, $sys::LZMA_FINISH);

            let written = spare.len() - stream.avail_out;
            unsafe {
                out.set_len(out.len() + written);
            }

            if ret as u32 == $sys::LZMA_STREAM_END as u32 {
                ret = $sys::LZMA_OK;
                break;
            }

            if ret as u32 != $sys::LZMA_OK as u32 {
                break;
            }

            if stream.avail_out != 0 {
                ret = $sys::LZMA_BUF_ERROR;
                break;
            }
        }

        $sys::lzma_end(&mut stream);
        (ret as u32, out)
    }};
}

#[inline]
unsafe fn c_encode_easy(input: &[u8]) -> (u32, Vec<u8>) {
    encode_easy_impl!(liblzma_sys, input)
}

#[inline]
unsafe fn rs_encode_easy(input: &[u8]) -> (u32, Vec<u8>) {
    encode_easy_impl!(xz_sys, input)
}

#[inline]
unsafe fn c_decode_stream_buffer(input: &[u8], expected_size_hint: usize) -> (u32, Vec<u8>) {
    decode_stream_impl!(liblzma_sys, input, expected_size_hint)
}

#[inline]
unsafe fn rs_decode_stream_buffer(input: &[u8], expected_size_hint: usize) -> (u32, Vec<u8>) {
    decode_stream_impl!(xz_sys, input, expected_size_hint)
}

fn deterministic_payload(case: usize) -> Vec<u8> {
    let len = match case % 7 {
        0 => 0,
        1 => 1,
        2 => 3,
        3 => 31,
        4 => 257,
        5 => 4096,
        _ => 16384,
    };
    let mut x = (0x9E3779B97F4A7C15u64 ^ (case as u64).wrapping_mul(0xD6E8FEB86659FD93)).max(1);
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        out.push((x.wrapping_mul(0x2545F4914F6CDD1D) & 0xFF) as u8);
    }
    out
}

#[test]
fn differential_roundtrip_across_backends() {
    for case in 0..128usize {
        let input = deterministic_payload(case);
        unsafe {
            let (c_enc_ret, c_encoded) = c_encode_easy(&input);
            let (rs_enc_ret, rs_encoded) = rs_encode_easy(&input);
            assert_eq!(
                c_enc_ret, rs_enc_ret,
                "encoder ret mismatch at case {case}: c={c_enc_ret} rs={rs_enc_ret}"
            );
            assert_eq!(
                c_enc_ret,
                liblzma_sys::LZMA_OK as u32,
                "C encoder failed at case {case} with ret {c_enc_ret}"
            );

            let (c_from_c_ret, c_from_c) = c_decode_stream_buffer(&c_encoded, input.len());
            let (rs_from_c_ret, rs_from_c) = rs_decode_stream_buffer(&c_encoded, input.len());
            let (c_from_rs_ret, c_from_rs) = c_decode_stream_buffer(&rs_encoded, input.len());
            let (rs_from_rs_ret, rs_from_rs) = rs_decode_stream_buffer(&rs_encoded, input.len());

            assert_eq!(
                c_from_c_ret, rs_from_c_ret,
                "decode(c_encoded) ret mismatch at case {case}: c={c_from_c_ret} rs={rs_from_c_ret}"
            );
            assert_eq!(
                c_from_rs_ret, rs_from_rs_ret,
                "decode(rs_encoded) ret mismatch at case {case}: c={c_from_rs_ret} rs={rs_from_rs_ret}"
            );

            assert_eq!(
                c_from_c, input,
                "C decode(C encode) payload mismatch at case {case}"
            );
            assert_eq!(
                rs_from_c, input,
                "RS decode(C encode) payload mismatch at case {case}"
            );
            assert_eq!(
                c_from_rs, input,
                "C decode(RS encode) payload mismatch at case {case}"
            );
            assert_eq!(
                rs_from_rs, input,
                "RS decode(RS encode) payload mismatch at case {case}"
            );
        }
    }
}

#[test]
fn differential_error_codes_on_invalid_corpus() {
    let files_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../vendor/xz/tests/files");
    for entry in fs::read_dir(&files_dir).expect("read xz test corpus directory") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|s| s.to_str()) != Some("xz") {
            continue;
        }

        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !(filename.starts_with("bad") || filename.starts_with("unsupported")) {
            continue;
        }
        if filename.contains("unsupported-check") {
            // This is implementation-defined in existing tests.
            continue;
        }

        let data = fs::read(&path).expect("read corpus file");
        unsafe {
            let (c_ret, _) = c_decode_stream_buffer(&data, 4096);
            let (rs_ret, _) = rs_decode_stream_buffer(&data, 4096);
            assert_eq!(
                c_ret,
                rs_ret,
                "error ret mismatch for {}: c={} rs={}",
                path.display(),
                c_ret,
                rs_ret
            );
        }
    }
}
