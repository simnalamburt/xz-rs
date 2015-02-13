#![feature(std_misc)]

extern crate "xz-sys" as xz_sys;

use std::str;
use std::ffi;
use xz_sys::*;

#[test]
fn version() {
    unsafe {
        let version = lzma_version_number();
        println!("{:?}", version);

        let version = lzma_version_string();
        let version = str::from_utf8(ffi::c_str_to_bytes(&version)).unwrap();
        println!("{:?}", version);
    }
}
