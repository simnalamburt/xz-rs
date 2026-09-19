#![cfg(not(target_family = "wasm"))]

use std::ffi::CStr;
use std::ptr;

use xz_sys::{LZMA_VLI_UNKNOWN, lzma_filter, lzma_str_to_filters};

type CInt = std::os::raw::c_int;

#[test]
fn str_to_filters_null_pointer_message_matches_c_api() {
    let mut filters = [lzma_filter {
        id: LZMA_VLI_UNKNOWN,
        options: ptr::null_mut(),
    }; 5];
    let mut error_pos: CInt = -1;

    unsafe {
        let msg = lzma_str_to_filters(
            ptr::null(),
            &mut error_pos,
            filters.as_mut_ptr(),
            0,
            ptr::null(),
        );

        assert_eq!(error_pos, 0);
        assert_eq!(
            CStr::from_ptr(msg).to_bytes(),
            b"Unexpected NULL pointer argument(s) to lzma_str_to_filters()"
        );
    }
}
