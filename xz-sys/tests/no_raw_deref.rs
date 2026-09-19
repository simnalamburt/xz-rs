//! `&*ptr` and `&mut *ptr` turn a C pointer into a reference with no NULL
//! test, which is exactly what the `extra-safety` feature exists to catch. No
//! clippy lint rejects the pattern, so this test scans the source instead.
//! Only the `c_ref` and `c_mut` helpers may contain it; every other site goes
//! through them.

const SOURCE: &str = include_str!("../src/lib.rs");

#[test]
fn raw_deref_appears_only_in_the_helpers() {
    let mut in_helper = false;
    let mut offenders = Vec::new();
    for (index, line) in SOURCE.lines().enumerate() {
        if line.starts_with("unsafe fn c_ref") || line.starts_with("unsafe fn c_mut") {
            in_helper = true;
        }
        let code = line.split("//").next().unwrap_or("");
        if !in_helper && (code.contains("&*") || code.contains("&mut *")) {
            offenders.push(format!("src/lib.rs:{}: {}", index + 1, line.trim()));
        }
        if in_helper && line == "}" {
            in_helper = false;
        }
    }
    assert!(
        offenders.is_empty(),
        "raw pointer dereferenced into a reference outside c_ref/c_mut:\n{}",
        offenders.join("\n")
    );
}
