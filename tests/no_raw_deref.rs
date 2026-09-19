// The test reads src/ from disk, which wasm targets cannot do.
#![cfg(not(target_family = "wasm"))]

//! `&*ptr` and `&mut *ptr` turn a C pointer into a reference with no NULL
//! test, which is exactly what the `extra-safety` feature exists to catch. No
//! clippy lint rejects the pattern, so this test scans the source instead.
//! `from_raw_parts` rebuilds a slice from a raw pointer. The crate shares the
//! reference shape across backends, so `src/` must not contain these patterns.

use std::fs;
use std::path::Path;

#[test]
fn src_has_no_raw_deref() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    visit_rs(&root, &mut offenders);
    assert!(
        offenders.is_empty(),
        "raw pointer turned into a reference or slice in src/:\n{}",
        offenders.join("\n")
    );
}

fn visit_rs(dir: &Path, offenders: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, offenders);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            scan_file(&path, offenders);
        }
    }
}

fn scan_file(path: &Path, offenders: &mut Vec<String>) {
    let source = fs::read_to_string(path).unwrap();
    let rel = path.strip_prefix(env!("CARGO_MANIFEST_DIR")).unwrap();
    for (index, line) in source.lines().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        if code.contains("&*") || code.contains("&mut *") || code.contains("from_raw_parts") {
            offenders.push(format!("{}:{}: {}", rel.display(), index + 1, line.trim()));
        }
    }
}
