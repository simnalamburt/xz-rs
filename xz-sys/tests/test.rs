#![feature(std_misc)]

extern crate "xz-sys" as xz_sys;

use std::str;
use std::ffi::CStr;
use std::mem;
use xz_sys::*;

#[test]
fn version() {
    unsafe {
        let version = lzma_version_number();
        println!("{:?}", version);

        let version = lzma_version_string();
        let version = str::from_utf8_unchecked(CStr::from_ptr(version).to_bytes());
        println!("{:?}", version);
    }
}

#[test]
fn easy_encoder() {
    unsafe {
        let mut stream: lzma_stream = mem::zeroed();
        match lzma_easy_encoder(&mut stream, 6, LZMA_CHECK_CRC64) {
            LZMA_OK => (),
            e => panic!("Error on lzma_stream_encoder (result: {})", e as i32)
        }

        let buffer_in = [b'H', b'e', b'l', b'l', b'o', b',', b' ', b'w', b'o', b'r', b'l', b'd'];
        stream.next_in = buffer_in.as_ptr();
        stream.avail_in = buffer_in.len() as u64;

        let mut buffer_out = [0u8; 4096];
        stream.next_out = buffer_out.as_mut_ptr();
        stream.avail_out = buffer_out.len() as u64;

        match lzma_code(&mut stream, LZMA_RUN) {
            LZMA_OK => (),
            e => panic!("Error on lzma_code (result: {})", e as i32)
        }

        lzma_end(&mut stream);

        print!("[");
        for byte in &buffer_out[..stream.total_out as usize] {
            print!("{}, ", byte);
        }
        println!("\x08\x08]");
    }
}
