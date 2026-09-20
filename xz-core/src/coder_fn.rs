//! Thunks that type-erase a coder's callbacks for the [`lzma_next_coder`]
//! vtable.
//!
//! That struct keeps C's shape, so every slot passes the coder state as
//! `*mut c_void` and each callback used to cast it back itself. The cast now
//! happens in a thunk generated where the slot is filled, next to the init
//! function that allocated the state, and the callback takes `&mut $coder`.
//!
//! The LZ layer's two tables had a closed set of implementations and are
//! enums instead: [`lzma_lz_decoder`] and [`lzma_lz_encoder`].
//!
//! [`lzma_next_coder`]: crate::types::lzma_next_coder
//! [`lzma_lz_decoder`]: crate::lz::lz_decoder::lzma_lz_decoder
//! [`lzma_lz_encoder`]: crate::lz::lz_encoder::lzma_lz_encoder

/// `lzma_next_coder::code`.
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

/// `lzma_next_coder::end`, and the `end` slot of both LZ vtables.
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

/// `lzma_next_coder::update`.
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

/// `lzma_next_coder::get_check`. The only slot that reads the coder state.
macro_rules! coder_get_check_fn {
    ($f:path, $coder:ty) => {{
        unsafe fn thunk(coder_ptr: *const $crate::types::c_void) -> $crate::types::lzma_check {
            $f(&*(coder_ptr as *const $coder))
        }
        Some(thunk as _)
    }};
}

/// `lzma_next_coder::get_progress`.
macro_rules! coder_get_progress_fn {
    ($f:path, $coder:ty) => {{
        unsafe fn thunk(
            coder_ptr: *mut $crate::types::c_void,
            progress_in: &mut u64,
            progress_out: &mut u64,
        ) {
            $f(&mut *(coder_ptr as *mut $coder), progress_in, progress_out)
        }
        Some(thunk as _)
    }};
}

/// `lzma_next_coder::memconfig`.
macro_rules! coder_memconfig_fn {
    ($f:path, $coder:ty) => {{
        unsafe fn thunk(
            coder_ptr: *mut $crate::types::c_void,
            memusage: *mut u64,
            old_memlimit: *mut u64,
            new_memlimit: u64,
        ) -> $crate::types::lzma_ret {
            $f(
                &mut *(coder_ptr as *mut $coder),
                memusage,
                old_memlimit,
                new_memlimit,
            )
        }
        Some(thunk as _)
    }};
}

/// `lzma_next_coder::set_out_limit`, and `lzma_lz_encoder::set_out_limit`.
macro_rules! coder_set_out_limit_fn {
    ($f:path, $coder:ty) => {{
        unsafe fn thunk(
            coder_ptr: *mut $crate::types::c_void,
            uncomp_size: *mut u64,
            out_limit: u64,
        ) -> $crate::types::lzma_ret {
            $f(&mut *(coder_ptr as *mut $coder), uncomp_size, out_limit)
        }
        Some(thunk as _)
    }};
}
