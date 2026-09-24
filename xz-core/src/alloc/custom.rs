use crate::types::*;

use super::c::{c_alloc_bytes, c_alloc_zeroed_bytes, c_allocator_ptr, c_free_ptr};
use crate::raw_alloc::{RUST_ALLOC_ALIGN, alloc_impl, free_ptr};

fn c_size(size: size_t) -> size_t {
    if size == 0 { 1 } else { size }
}

pub fn allocator_or_c(allocator: *const lzma_allocator) -> *const lzma_allocator {
    if allocator.is_null() {
        c_allocator_ptr()
    } else {
        allocator
    }
}

pub unsafe fn lzma_alloc(size: size_t, allocator: *const lzma_allocator) -> *mut c_void {
    let size = c_size(size);
    if !allocator.is_null()
        && let Some(alloc) = unsafe { (*allocator).alloc }
    {
        return unsafe { alloc((*allocator).opaque, 1, size) };
    }
    unsafe { c_alloc_bytes(size) }
}

pub unsafe fn lzma_alloc_zero(size: size_t, allocator: *const lzma_allocator) -> *mut c_void {
    let size = c_size(size);
    if !allocator.is_null()
        && let Some(alloc) = unsafe { (*allocator).alloc }
    {
        let ptr = unsafe { alloc((*allocator).opaque, 1, size) };
        if !ptr.is_null() {
            unsafe { core::ptr::write_bytes(ptr as *mut u8, 0, size) };
        }
        return ptr;
    }
    unsafe { c_alloc_zeroed_bytes(size) }
}

pub unsafe fn lzma_free(ptr: *mut c_void, allocator: *const lzma_allocator) {
    if !allocator.is_null()
        && let Some(free) = unsafe { (*allocator).free }
    {
        unsafe { free((*allocator).opaque, ptr) };
        return;
    }
    unsafe { c_free_ptr(ptr) };
}

pub(crate) unsafe fn internal_alloc_bytes(
    size: size_t,
    allocator: *const lzma_allocator,
) -> *mut c_void {
    let size = c_size(size);
    if !allocator.is_null() {
        if let Some(alloc) = unsafe { (*allocator).alloc } {
            return unsafe { alloc((*allocator).opaque, 1, size) };
        }
        return unsafe { c_alloc_bytes(size) };
    }
    alloc_impl(size as usize, RUST_ALLOC_ALIGN, false)
}

pub(crate) unsafe fn internal_alloc_untyped_bytes(
    size: size_t,
    allocator: *const lzma_allocator,
) -> *mut c_void {
    unsafe { internal_alloc_bytes(size, allocator) }
}

pub(crate) unsafe fn internal_alloc_zeroed_bytes(
    size: size_t,
    allocator: *const lzma_allocator,
) -> *mut c_void {
    let size = c_size(size);
    if !allocator.is_null() {
        if let Some(alloc) = unsafe { (*allocator).alloc } {
            let ptr = unsafe { alloc((*allocator).opaque, 1, size) };
            if !ptr.is_null() {
                unsafe { core::ptr::write_bytes(ptr.cast::<u8>(), 0, size) };
            }
            return ptr;
        }
        return unsafe { c_alloc_zeroed_bytes(size) };
    }
    alloc_impl(size as usize, RUST_ALLOC_ALIGN, true)
}

pub(crate) unsafe fn internal_alloc_object<T>(allocator: *const lzma_allocator) -> *mut T {
    if !allocator.is_null() {
        let size = c_size(core::mem::size_of::<T>() as size_t);
        if let Some(alloc) = unsafe { (*allocator).alloc } {
            return unsafe { alloc((*allocator).opaque, 1, size) as *mut T };
        }
        return unsafe { c_alloc_bytes(size) as *mut T };
    }
    alloc_impl(core::mem::size_of::<T>(), core::mem::align_of::<T>(), false) as *mut T
}

pub(crate) unsafe fn internal_alloc_array<T>(
    count: size_t,
    allocator: *const lzma_allocator,
) -> *mut T {
    let Some(size) = (count as usize).checked_mul(core::mem::size_of::<T>()) else {
        return core::ptr::null_mut();
    };
    if !allocator.is_null() {
        let size = c_size(size as size_t);
        if let Some(alloc) = unsafe { (*allocator).alloc } {
            return unsafe { alloc((*allocator).opaque, 1, size) as *mut T };
        }
        return unsafe { c_alloc_bytes(size) as *mut T };
    }
    alloc_impl(size, core::mem::align_of::<T>(), false) as *mut T
}

pub(crate) unsafe fn internal_alloc_zeroed_array<T>(
    count: size_t,
    allocator: *const lzma_allocator,
) -> *mut T {
    let Some(size) = (count as usize).checked_mul(core::mem::size_of::<T>()) else {
        return core::ptr::null_mut();
    };
    if !allocator.is_null() {
        let alloc_size = c_size(size as size_t);
        if let Some(alloc) = unsafe { (*allocator).alloc } {
            let ptr = unsafe { alloc((*allocator).opaque, 1, alloc_size) as *mut T };
            if !ptr.is_null() {
                unsafe { core::ptr::write_bytes(ptr as *mut u8, 0, alloc_size as usize) };
            }
            return ptr;
        }
        return unsafe { c_alloc_zeroed_bytes(alloc_size) as *mut T };
    }
    alloc_impl(size, core::mem::align_of::<T>(), true) as *mut T
}

pub(crate) unsafe fn internal_free_bytes(
    ptr: *mut c_void,
    _size: size_t,
    allocator: *const lzma_allocator,
) {
    if !allocator.is_null()
        && let Some(free) = unsafe { (*allocator).free }
    {
        unsafe { free((*allocator).opaque, ptr) };
        return;
    }
    if !allocator.is_null() {
        unsafe { c_free_ptr(ptr) };
        return;
    }
    unsafe { free_ptr(ptr) };
}

pub(crate) unsafe fn internal_free_untyped(ptr: *mut c_void, allocator: *const lzma_allocator) {
    unsafe { internal_free_bytes(ptr, 0, allocator) };
}

pub(crate) unsafe fn internal_free_untyped_bytes(
    ptr: *mut c_void,
    allocator: *const lzma_allocator,
) {
    unsafe { internal_free_bytes(ptr, 0, allocator) };
}

pub(crate) unsafe fn internal_free<T>(ptr: *mut T, allocator: *const lzma_allocator) {
    unsafe { internal_free_bytes(ptr.cast(), 0, allocator) };
}

pub(crate) unsafe fn internal_free_dyn<T: ?Sized>(ptr: *mut T, allocator: *const lzma_allocator) {
    unsafe { internal_free_bytes(ptr.cast(), 0, allocator) };
}

pub(crate) unsafe fn internal_free_array<T>(
    ptr: *mut T,
    _count: size_t,
    allocator: *const lzma_allocator,
) {
    unsafe { internal_free_bytes(ptr.cast(), 0, allocator) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};

    static FREE_COUNT: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn c_alloc(_opaque: *mut c_void, nmemb: size_t, size: size_t) -> *mut c_void {
        let Some(bytes) = (nmemb as usize).checked_mul(size as usize) else {
            return core::ptr::null_mut();
        };
        unsafe { c_alloc_bytes(bytes as size_t) }
    }

    unsafe extern "C" fn counted_c_free(_opaque: *mut c_void, ptr: *mut c_void) {
        FREE_COUNT.fetch_add(1, Ordering::Relaxed);
        unsafe { c_free_ptr(ptr) };
    }

    #[test]
    fn internal_free_falls_back_to_c_free_for_partial_c_allocator() {
        unsafe {
            let allocator = lzma_allocator {
                alloc: Some(c_alloc),
                free: None,
                opaque: core::ptr::null_mut(),
            };

            let ptr = internal_alloc_object::<u64>(&allocator);
            assert!(!ptr.is_null());
            internal_free(ptr, &allocator);
        }
    }

    #[test]
    fn internal_alloc_falls_back_to_c_alloc_for_partial_c_allocator() {
        unsafe {
            FREE_COUNT.store(0, Ordering::Relaxed);
            let allocator = lzma_allocator {
                alloc: None,
                free: Some(counted_c_free),
                opaque: core::ptr::null_mut(),
            };

            let ptr = internal_alloc_bytes(16, &allocator);
            assert!(!ptr.is_null());
            internal_free_bytes(ptr, 16, &allocator);
            assert_eq!(FREE_COUNT.load(Ordering::Relaxed), 1);
        }
    }
}
