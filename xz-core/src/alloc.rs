#[cfg(feature = "custom_allocator")]
mod c;
#[cfg(feature = "custom_allocator")]
mod custom;
#[cfg(feature = "custom_allocator")]
mod rust;

#[cfg(not(feature = "custom_allocator"))]
use crate::raw_alloc as policy;
#[cfg(feature = "custom_allocator")]
use custom as policy;

#[cfg(feature = "custom_allocator")]
pub use c::{c_allocator, c_allocator_ptr, lzma_c_alloc, lzma_c_free};
#[cfg(feature = "custom_allocator")]
pub use custom::allocator_or_c;
#[cfg(feature = "custom_allocator")]
pub use rust::rust_allocator;

#[cfg(feature = "custom_allocator")]
pub use policy::{lzma_alloc, lzma_alloc_zero, lzma_free};

pub(crate) use policy::{
    internal_alloc_array, internal_alloc_bytes, internal_alloc_object,
    internal_alloc_untyped_bytes, internal_alloc_zeroed_array, internal_alloc_zeroed_bytes,
    internal_free, internal_free_array, internal_free_bytes, internal_free_dyn,
    internal_free_untyped_bytes,
};
