use crate::types::*;
use std::alloc::{Layout, alloc, alloc_zeroed, dealloc};

pub(crate) const RUST_ALLOC_ALIGN: usize = 16;
const ZERO_SIZE_PTR: *mut c_void = core::ptr::without_provenance_mut(RUST_ALLOC_ALIGN);

#[derive(Copy, Clone)]
#[repr(C)]
struct RustAllocHeader {
    total_size: usize,
    align: usize,
    offset: usize,
}

const fn round_up(value: usize, align: usize) -> usize {
    (value + (align - 1)) & !(align - 1)
}

pub(crate) fn alloc_impl(size: usize, align: usize, zeroed: bool) -> *mut c_void {
    if size == 0 {
        return ZERO_SIZE_PTR;
    }
    let align = align
        .max(RUST_ALLOC_ALIGN)
        .max(core::mem::align_of::<RustAllocHeader>());
    let header_size = core::mem::size_of::<RustAllocHeader>();
    let Some(value) = header_size.checked_add(align - 1) else {
        return core::ptr::null_mut();
    };
    let offset = round_up(value, align);
    let Some(total_size) = offset.checked_add(size) else {
        return core::ptr::null_mut();
    };
    let Ok(layout) = Layout::from_size_align(total_size, align) else {
        return core::ptr::null_mut();
    };
    let base = unsafe {
        if zeroed {
            alloc_zeroed(layout)
        } else {
            alloc(layout)
        }
    };
    if base.is_null() {
        return core::ptr::null_mut();
    }
    let user_ptr = unsafe { base.add(offset) };
    let header_ptr = unsafe { user_ptr.sub(header_size) as *mut RustAllocHeader };
    unsafe {
        header_ptr.write(RustAllocHeader {
            total_size,
            align,
            offset,
        });
    }
    user_ptr as *mut c_void
}

fn raw_layout(size: usize, align: usize) -> Option<Layout> {
    Layout::from_size_align(size, align.max(RUST_ALLOC_ALIGN)).ok()
}

pub(crate) unsafe fn free_impl(ptr: *mut c_void) {
    if ptr.is_null() || ptr == ZERO_SIZE_PTR {
        return;
    }
    let header_size = core::mem::size_of::<RustAllocHeader>();
    let user_ptr = ptr as *mut u8;
    let header_ptr = unsafe { user_ptr.sub(header_size) as *const RustAllocHeader };
    let header = unsafe { header_ptr.read() };
    let base = unsafe { user_ptr.sub(header.offset) };
    let layout = unsafe { Layout::from_size_align_unchecked(header.total_size, header.align) };
    unsafe { dealloc(base, layout) };
}

#[cfg(feature = "custom_allocator")]
pub(crate) unsafe fn alloc_bytes(size: size_t) -> *mut c_void {
    alloc_impl(size as usize, RUST_ALLOC_ALIGN, false)
}

#[cfg(feature = "custom_allocator")]
pub(crate) unsafe fn alloc_zeroed_bytes(size: size_t) -> *mut c_void {
    alloc_impl(size as usize, RUST_ALLOC_ALIGN, true)
}

#[cfg(feature = "custom_allocator")]
pub(crate) unsafe fn free_ptr(ptr: *mut c_void) {
    unsafe { free_impl(ptr) };
}

#[cfg(feature = "custom_allocator")]
pub unsafe fn lzma_alloc(size: size_t, _allocator: *const lzma_allocator) -> *mut c_void {
    unsafe { alloc_bytes(size) }
}

#[cfg(feature = "custom_allocator")]
pub unsafe fn lzma_alloc_zero(size: size_t, _allocator: *const lzma_allocator) -> *mut c_void {
    unsafe { alloc_zeroed_bytes(size) }
}

#[cfg(feature = "custom_allocator")]
pub unsafe fn lzma_free(ptr: *mut c_void, _allocator: *const lzma_allocator) {
    unsafe { free_ptr(ptr) };
}

pub(crate) unsafe fn internal_alloc_bytes(
    size: size_t,
    _allocator: *const lzma_allocator,
) -> *mut c_void {
    let Some(layout) = raw_layout(size as usize, RUST_ALLOC_ALIGN) else {
        return core::ptr::null_mut();
    };
    if layout.size() == 0 {
        return core::ptr::NonNull::<u8>::dangling().as_ptr().cast();
    }
    unsafe { alloc(layout).cast() }
}

pub(crate) unsafe fn internal_alloc_untyped_bytes(
    size: size_t,
    _allocator: *const lzma_allocator,
) -> *mut c_void {
    alloc_impl(size as usize, RUST_ALLOC_ALIGN, false)
}

pub(crate) unsafe fn internal_alloc_zeroed_bytes(
    size: size_t,
    _allocator: *const lzma_allocator,
) -> *mut c_void {
    let Some(layout) = raw_layout(size as usize, RUST_ALLOC_ALIGN) else {
        return core::ptr::null_mut();
    };
    if layout.size() == 0 {
        return core::ptr::NonNull::<u8>::dangling().as_ptr().cast();
    }
    unsafe { alloc_zeroed(layout).cast() }
}

pub(crate) unsafe fn internal_alloc_object<T>(_allocator: *const lzma_allocator) -> *mut T {
    let Some(layout) = raw_layout(core::mem::size_of::<T>(), core::mem::align_of::<T>()) else {
        return core::ptr::null_mut();
    };
    if layout.size() == 0 {
        return core::ptr::NonNull::<T>::dangling().as_ptr();
    }
    unsafe { alloc(layout).cast::<T>() }
}

pub(crate) unsafe fn internal_alloc_array<T>(
    count: size_t,
    _allocator: *const lzma_allocator,
) -> *mut T {
    let Some(size) = (count as usize).checked_mul(core::mem::size_of::<T>()) else {
        return core::ptr::null_mut();
    };
    let Some(layout) = raw_layout(size, core::mem::align_of::<T>()) else {
        return core::ptr::null_mut();
    };
    if layout.size() == 0 {
        return core::ptr::NonNull::<T>::dangling().as_ptr();
    }
    unsafe { alloc(layout).cast::<T>() }
}

pub(crate) unsafe fn internal_alloc_zeroed_array<T>(
    count: size_t,
    _allocator: *const lzma_allocator,
) -> *mut T {
    let Some(size) = (count as usize).checked_mul(core::mem::size_of::<T>()) else {
        return core::ptr::null_mut();
    };
    let Some(layout) = raw_layout(size, core::mem::align_of::<T>()) else {
        return core::ptr::null_mut();
    };
    if layout.size() == 0 {
        return core::ptr::NonNull::<T>::dangling().as_ptr();
    }
    unsafe { alloc_zeroed(layout).cast::<T>() }
}

pub(crate) unsafe fn internal_free_bytes(
    ptr: *mut c_void,
    size: size_t,
    _allocator: *const lzma_allocator,
) {
    let Some(layout) = raw_layout(size as usize, RUST_ALLOC_ALIGN) else {
        return;
    };
    if ptr.is_null() || layout.size() == 0 {
        return;
    }
    unsafe { dealloc(ptr.cast::<u8>(), layout) };
}

pub(crate) unsafe fn internal_free_untyped(ptr: *mut c_void, _allocator: *const lzma_allocator) {
    assert!(
        ptr.is_null(),
        "allocated xz-core coders must install a typed end function"
    );
}

pub(crate) unsafe fn internal_free_untyped_bytes(
    ptr: *mut c_void,
    _allocator: *const lzma_allocator,
) {
    unsafe { free_impl(ptr) };
}

pub(crate) unsafe fn internal_free<T>(ptr: *mut T, _allocator: *const lzma_allocator) {
    if ptr.is_null() || core::mem::size_of::<T>() == 0 {
        return;
    }
    let Some(layout) = raw_layout(core::mem::size_of::<T>(), core::mem::align_of::<T>()) else {
        return;
    };
    unsafe { dealloc(ptr.cast::<u8>(), layout) };
}

/// Frees an object whose size and alignment come from the pointer's metadata,
/// which is how a trait object's state is freed after its `end`.
pub(crate) unsafe fn internal_free_dyn<T: ?Sized>(ptr: *mut T, _allocator: *const lzma_allocator) {
    let value = unsafe { Layout::for_value(&*ptr) };
    if value.size() == 0 {
        return;
    }
    let Some(layout) = raw_layout(value.size(), value.align()) else {
        return;
    };
    unsafe { dealloc(ptr.cast::<u8>(), layout) };
}

pub(crate) unsafe fn internal_free_array<T>(
    ptr: *mut T,
    count: size_t,
    _allocator: *const lzma_allocator,
) {
    let Some(size) = (count as usize).checked_mul(core::mem::size_of::<T>()) else {
        return;
    };
    let Some(layout) = raw_layout(size, core::mem::align_of::<T>()) else {
        return;
    };
    if ptr.is_null() || layout.size() == 0 {
        return;
    }
    unsafe { dealloc(ptr.cast::<u8>(), layout) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "custom_allocator")]
    fn raw_allocator_round_trip() {
        unsafe {
            let ptr = alloc_bytes(32);
            assert!(!ptr.is_null());
            free_ptr(ptr);
        }
    }

    #[test]
    #[cfg(feature = "custom_allocator")]
    fn raw_allocation_respects_alignment() {
        #[repr(align(32))]
        struct Align32([u8; 32]);

        unsafe {
            let ptr = alloc_impl(
                core::mem::size_of::<Align32>(),
                core::mem::align_of::<Align32>(),
                false,
            );
            assert!(!ptr.is_null());
            assert_eq!((ptr as usize) % core::mem::align_of::<Align32>(), 0);
            free_impl(ptr);
        }
    }

    #[test]
    fn internal_allocator_round_trip() {
        unsafe {
            let ptr = internal_alloc_bytes(32, core::ptr::null());
            assert!(!ptr.is_null());
            internal_free_bytes(ptr, 32, core::ptr::null());
        }
    }
}
