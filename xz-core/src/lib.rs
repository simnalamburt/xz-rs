#![allow(
    clashing_extern_declarations,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unsafe_op_in_unsafe_fn,
    unused_assignments,
    clippy::all
)]
// Soundness lints that the blanket `clippy::all` allow above would otherwise
// hide. The C original adds a `size_t` to a pointer; `.offset()` takes an
// `isize`, so on a 32-bit target every index above `isize::MAX` turns into a
// negative offset and reads or writes outside the allocation. `.add()` keeps
// the unsigned semantics the C code has.
#![deny(clippy::ptr_offset_with_cast)]
// A `pub fn` that dereferences one of its raw-pointer parameters can be called
// from safe code with any address, so the `unsafe` belongs in the signature.
#![deny(clippy::not_unsafe_ptr_arg_deref)]
/// Type-erases a coder's `code` function for the `lzma_next_coder` vtable.
///
/// The vtable slot has to be `*mut c_void`, so the cast back to the coder's own
/// type happens here, in one place per slot, next to the init function that
/// allocated the state. The implementation itself takes `&mut $coder`.
macro_rules! coder_code_fn {
    ($f:path, $coder:ty) => {{
        unsafe fn thunk(
            coder_ptr: *mut $crate::types::c_void,
            allocator: *const $crate::types::lzma_allocator,
            input: *const u8,
            in_pos: *mut $crate::types::size_t,
            in_size: $crate::types::size_t,
            out: *mut u8,
            out_pos: *mut $crate::types::size_t,
            out_size: $crate::types::size_t,
            action: $crate::types::lzma_action,
        ) -> $crate::types::lzma_ret {
            $f(
                &mut *(coder_ptr as *mut $coder),
                allocator,
                input,
                in_pos,
                in_size,
                out,
                out_pos,
                out_size,
                action,
            )
        }
        Some(thunk as _)
    }};
}

/// Type-erases a coder's `end` function. See [`coder_code_fn`].
macro_rules! coder_end_fn {
    ($f:path, $coder:ty) => {{
        unsafe fn thunk(
            coder_ptr: *mut $crate::types::c_void,
            allocator: *const $crate::types::lzma_allocator,
        ) {
            $f(&mut *(coder_ptr as *mut $coder), allocator)
        }
        Some(thunk as _)
    }};
}

/// Type-erases a coder's `update` function. See [`coder_code_fn`].
macro_rules! coder_update_fn {
    ($f:path, $coder:ty) => {{
        unsafe fn thunk(
            coder_ptr: *mut $crate::types::c_void,
            allocator: *const $crate::types::lzma_allocator,
            filters: *const $crate::types::lzma_filter,
            reversed_filters: *const $crate::types::lzma_filter,
        ) -> $crate::types::lzma_ret {
            $f(
                &mut *(coder_ptr as *mut $coder),
                allocator,
                filters,
                reversed_filters,
            )
        }
        Some(thunk as _)
    }};
}

#[macro_export]
macro_rules! c_str {
    ($s:literal) => {
        concat!($s, "\0").as_ptr() as *const ::std::os::raw::c_char
    };
}
pub mod alloc;
pub mod check;
pub mod common;
pub mod delta;
pub mod lz;
pub mod lzma;
pub mod rangecoder;
mod raw_alloc;
pub mod simple;
pub mod tuklib;
pub mod types;
