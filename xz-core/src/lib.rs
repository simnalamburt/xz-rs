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
