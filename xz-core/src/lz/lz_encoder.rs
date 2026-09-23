use crate::lz::lz_encoder_mf::{
    lzma_mf_bt2_find, lzma_mf_bt2_skip, lzma_mf_bt3_find, lzma_mf_bt3_skip, lzma_mf_bt4_find,
    lzma_mf_bt4_skip, lzma_mf_hc3_find, lzma_mf_hc3_skip, lzma_mf_hc4_find, lzma_mf_hc4_skip,
};
use crate::lzma::lzma2_encoder::lzma_lzma2_coder;
use crate::types::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_lz_options {
    pub before_size: size_t,
    pub dict_size: size_t,
    pub after_size: size_t,
    pub match_len_max: size_t,
    pub nice_len: size_t,
    pub match_finder: lzma_match_finder,
    pub depth: u32,
    pub preset_dict: *const u8,
    pub preset_dict_size: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_coder {
    pub lz: lzma_lz_encoder,
    pub mf: lzma_mf,
    pub next: lzma_next_coder,
}
#[inline]
unsafe fn move_window(mf: *mut lzma_mf) {
    debug_assert!((*mf).read_pos > (*mf).keep_size_before);
    let move_offset: u32 = ((*mf).read_pos - (*mf).keep_size_before) & !15;
    debug_assert!((*mf).write_pos > move_offset);
    let move_size: size_t = ((*mf).write_pos - move_offset) as size_t;
    core::ptr::copy(
        (*mf).buffer.offset(move_offset as isize) as *const u8,
        (*mf).buffer as *mut u8,
        move_size,
    );
    (*mf).offset += move_offset;
    (*mf).read_pos -= move_offset;
    (*mf).read_limit -= move_offset;
    (*mf).write_pos -= move_offset;
}
unsafe fn fill_window(
    coder: *mut lzma_coder,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    action: lzma_action,
) -> lzma_ret {
    debug_assert!((*coder).mf.read_pos <= (*coder).mf.write_pos);
    if (*coder).mf.read_pos >= (*coder).mf.size - (*coder).mf.keep_size_after {
        move_window(::core::ptr::addr_of_mut!((*coder).mf));
    }
    let mut write_pos: size_t = (*coder).mf.write_pos as size_t;
    let mut ret = if (*coder).next.coder.is_some() {
        (*coder).next.code(
            allocator,
            input,
            in_pos,
            in_size,
            (*coder).mf.buffer,
            ::core::ptr::addr_of_mut!(write_pos),
            (*coder).mf.size as size_t,
            action,
        )
    } else {
        lzma_bufcpy(
            input,
            in_pos,
            in_size,
            (*coder).mf.buffer,
            ::core::ptr::addr_of_mut!(write_pos),
            (*coder).mf.size as size_t,
        );
        if action != LZMA_RUN && *in_pos == in_size {
            LZMA_STREAM_END
        } else {
            LZMA_OK
        }
    };
    (*coder).mf.write_pos = write_pos as u32;
    core::ptr::write_bytes(
        (*coder).mf.buffer.add(write_pos) as *mut u8,
        0 as u8,
        LZMA_MEMCMPLEN_EXTRA as usize,
    );
    if ret == LZMA_STREAM_END {
        ret = LZMA_OK;
        (*coder).mf.action = action;
        (*coder).mf.read_limit = (*coder).mf.write_pos;
    } else if (*coder).mf.write_pos > (*coder).mf.keep_size_after {
        (*coder).mf.read_limit = (*coder).mf.write_pos - (*coder).mf.keep_size_after;
    }
    if (*coder).mf.pending > 0 && (*coder).mf.read_pos < (*coder).mf.read_limit {
        let pending: u32 = (*coder).mf.pending;
        (*coder).mf.pending = 0;
        debug_assert!((*coder).mf.read_pos >= pending);
        (*coder).mf.read_pos -= pending;
        ((*coder).mf.skip)(::core::ptr::addr_of_mut!((*coder).mf), pending);
    }
    ret
}
/// The encoder the LZ layer drives, the counterpart of
/// [`lzma_lz_decoder`](crate::lz::lz_decoder::lzma_lz_decoder). The
/// implementations are LZMA1 and LZMA2, so the table of function pointers C
/// keeps here is an enum and every call below is a direct one.
#[derive(Copy, Clone)]
pub enum lzma_lz_encoder {
    /// Before the filter's init function fills this in, and after `end`.
    Uninitialized,
    Lzma1(*mut lzma_lzma1_encoder),
    Lzma2(*mut lzma_lzma2_coder),
}

#[cold]
fn uninitialized() -> ! {
    panic!("uninitialized LZ encoder callback")
}

impl lzma_lz_encoder {
    pub unsafe fn code(
        &mut self,
        mf: *mut lzma_mf,
        out: *mut u8,
        out_pos: *mut size_t,
        out_size: size_t,
    ) -> lzma_ret {
        match *self {
            Self::Lzma1(coder) => {
                crate::lzma::lzma_encoder::lzma_encode(&mut *coder, mf, out, out_pos, out_size)
            }
            Self::Lzma2(coder) => {
                crate::lzma::lzma2_encoder::lzma2_encode(&mut *coder, mf, out, out_pos, out_size)
            }
            Self::Uninitialized => uninitialized(),
        }
    }

    pub unsafe fn end(&mut self, allocator: *const lzma_allocator) {
        match *self {
            Self::Lzma1(coder) => {
                crate::lzma::lzma_encoder::lzma_encoder_end(&mut *coder, allocator)
            }
            Self::Lzma2(coder) => {
                crate::lzma::lzma2_encoder::lzma2_encoder_end(&mut *coder, allocator)
            }
            Self::Uninitialized => {
                #[cfg(feature = "custom_allocator")]
                crate::alloc::internal_free_bytes(core::ptr::null_mut(), 0, allocator);
            }
        }
        *self = Self::Uninitialized;
    }

    /// Only LZMA2 can update its options; C stores `NULL` for the others and
    /// the caller answers `LZMA_PROG_ERROR`, which is what the other arms do.
    pub unsafe fn options_update(&mut self, filter: *const lzma_filter) -> lzma_ret {
        match *self {
            Self::Lzma2(coder) => {
                crate::lzma::lzma2_encoder::lzma2_encoder_options_update(&mut *coder, filter)
            }
            _ => LZMA_PROG_ERROR,
        }
    }

    /// Only LZMA1 has this. `None` is C's NULL slot, which leaves the answer
    /// to the caller.
    pub unsafe fn set_out_limit(
        &mut self,
        uncomp_size: *mut u64,
        out_limit: u64,
    ) -> Option<lzma_ret> {
        match *self {
            Self::Lzma1(coder) => Some(crate::lzma::lzma_encoder::lzma_lzma_set_out_limit(
                &mut *coder,
                uncomp_size,
                out_limit,
            )),
            _ => None,
        }
    }
}

unsafe fn lz_encode(
    coder: &mut lzma_coder,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    action: lzma_action,
) -> lzma_ret {
    while *out_pos < out_size && (*in_pos < in_size || action != LZMA_RUN) {
        if coder.mf.action == LZMA_RUN && coder.mf.read_pos >= coder.mf.read_limit {
            let ret_: lzma_ret = fill_window(coder, allocator, input, in_pos, in_size, action);
            if ret_ != LZMA_OK {
                return ret_;
            }
        }
        let ret: lzma_ret =
            coder
                .lz
                .code(::core::ptr::addr_of_mut!(coder.mf), out, out_pos, out_size);
        if ret != LZMA_OK {
            coder.mf.action = LZMA_RUN;
            return ret;
        }
    }
    LZMA_OK
}
unsafe fn lz_encoder_prepare(
    mf: *mut lzma_mf,
    allocator: *const lzma_allocator,
    lz_options: *const lzma_lz_options,
) -> bool {
    if (*lz_options).dict_size < LZMA_DICT_SIZE_MIN as size_t
        || (*lz_options).dict_size > ((1u32 << 30) + (1u32 << 29)) as size_t
        || (*lz_options).nice_len > (*lz_options).match_len_max
    {
        return true;
    }
    (*mf).keep_size_before = ((*lz_options).before_size + (*lz_options).dict_size) as u32;
    (*mf).keep_size_after = ((*lz_options).after_size + (*lz_options).match_len_max) as u32;
    let mut reserve: u32 = ((*lz_options).dict_size / 2) as u32;
    if reserve > 1 << 30 {
        reserve /= 2;
    }
    reserve +=
        (((*lz_options).before_size + (*lz_options).match_len_max + (*lz_options).after_size) / 2
            + (1u32 << 19) as size_t) as u32;
    let old_size: u32 = (*mf).size;
    (*mf).size = (*mf).keep_size_before + reserve + (*mf).keep_size_after;
    if !(*mf).buffer.is_null() && old_size != (*mf).size {
        crate::alloc::internal_free_array(
            (*mf).buffer,
            (old_size + LZMA_MEMCMPLEN_EXTRA) as size_t,
            allocator,
        );
        (*mf).buffer = core::ptr::null_mut();
    }
    (*mf).match_len_max = (*lz_options).match_len_max as u32;
    (*mf).nice_len = (*lz_options).nice_len as u32;
    (*mf).cyclic_size = (*lz_options).dict_size as u32 + 1;
    match (*lz_options).match_finder {
        3 => {
            (*mf).find = lzma_mf_hc3_find as lzma_mf_find_function;
            (*mf).skip = lzma_mf_hc3_skip as lzma_mf_skip_function;
        }
        4 => {
            (*mf).find = lzma_mf_hc4_find as lzma_mf_find_function;
            (*mf).skip = lzma_mf_hc4_skip as lzma_mf_skip_function;
        }
        18 => {
            (*mf).find = lzma_mf_bt2_find as lzma_mf_find_function;
            (*mf).skip = lzma_mf_bt2_skip as lzma_mf_skip_function;
        }
        19 => {
            (*mf).find = lzma_mf_bt3_find as lzma_mf_find_function;
            (*mf).skip = lzma_mf_bt3_skip as lzma_mf_skip_function;
        }
        20 => {
            (*mf).find = lzma_mf_bt4_find as lzma_mf_find_function;
            (*mf).skip = lzma_mf_bt4_skip as lzma_mf_skip_function;
        }
        _ => return true,
    }
    let hash_bytes: u32 = mf_get_hash_bytes((*lz_options).match_finder) as u32;
    let is_bt: bool = (*lz_options).match_finder & 0x10 != 0;
    let mut hs: u32 = 0;
    if hash_bytes == 2 {
        hs = 0xffff;
    } else {
        hs = (*lz_options).dict_size as u32 - 1;
        hs |= hs >> 1;
        hs |= hs >> 2;
        hs |= hs >> 4;
        hs |= hs >> 8;
        hs >>= 1;
        hs |= 0xffff;
        if hs > 1 << 24 {
            if hash_bytes == 3 {
                hs = (1u32 << 24) - 1;
            } else {
                hs >>= 1;
            }
        }
    }
    (*mf).hash_mask = hs;
    hs += 1;
    if hash_bytes > 2 {
        hs += HASH_2_SIZE;
    }
    if hash_bytes > 3 {
        hs += HASH_3_SIZE;
    }
    let old_hash_count: u32 = (*mf).hash_count;
    let old_sons_count: u32 = (*mf).sons_count;
    (*mf).hash_count = hs;
    (*mf).sons_count = (*mf).cyclic_size;
    if is_bt {
        (*mf).sons_count *= 2;
    }
    if old_hash_count != (*mf).hash_count || old_sons_count != (*mf).sons_count {
        crate::alloc::internal_free_array((*mf).hash, old_hash_count as size_t, allocator);
        (*mf).hash = core::ptr::null_mut();
        crate::alloc::internal_free_array((*mf).son, old_sons_count as size_t, allocator);
        (*mf).son = core::ptr::null_mut();
    }
    (*mf).depth = (*lz_options).depth;
    if (*mf).depth == 0 {
        if is_bt {
            (*mf).depth = 16u32 + (*mf).nice_len / 2;
        } else {
            (*mf).depth = 4u32 + (*mf).nice_len / 4;
        }
    }
    false
}
unsafe fn lz_encoder_init(
    mf: *mut lzma_mf,
    allocator: *const lzma_allocator,
    lz_options: *const lzma_lz_options,
) -> bool {
    if (*mf).buffer.is_null() {
        (*mf).buffer = crate::alloc::internal_alloc_array::<u8>(
            ((*mf).size + LZMA_MEMCMPLEN_EXTRA) as size_t,
            allocator,
        );
        if (*mf).buffer.is_null() {
            return true;
        }
        core::ptr::write_bytes(
            (*mf).buffer.offset((*mf).size as isize) as *mut u8,
            0 as u8,
            LZMA_MEMCMPLEN_EXTRA as usize,
        );
    }
    (*mf).offset = (*mf).cyclic_size;
    (*mf).read_pos = 0;
    (*mf).read_ahead = 0;
    (*mf).read_limit = 0;
    (*mf).write_pos = 0;
    (*mf).pending = 0;
    if (*mf).hash.is_null() {
        (*mf).hash =
            crate::alloc::internal_alloc_zeroed_array::<u32>((*mf).hash_count as size_t, allocator);
        (*mf).son =
            crate::alloc::internal_alloc_array::<u32>((*mf).sons_count as size_t, allocator);
        if (*mf).hash.is_null() || (*mf).son.is_null() {
            crate::alloc::internal_free_array((*mf).hash, (*mf).hash_count as size_t, allocator);
            (*mf).hash = core::ptr::null_mut();
            crate::alloc::internal_free_array((*mf).son, (*mf).sons_count as size_t, allocator);
            (*mf).son = core::ptr::null_mut();
            return true;
        }
    } else {
        core::ptr::write_bytes(
            (*mf).hash as *mut u8,
            0 as u8,
            ((*mf).hash_count as size_t) * core::mem::size_of::<u32>(),
        );
    }
    (*mf).cyclic_pos = 0;
    if !(*lz_options).preset_dict.is_null() && (*lz_options).preset_dict_size > 0 {
        (*mf).write_pos = if (*lz_options).preset_dict_size < (*mf).size {
            (*lz_options).preset_dict_size
        } else {
            (*mf).size
        };
        core::ptr::copy_nonoverlapping(
            (*lz_options)
                .preset_dict
                .offset((*lz_options).preset_dict_size as isize)
                .offset(-((*mf).write_pos as isize)) as *const u8,
            (*mf).buffer as *mut u8,
            (*mf).write_pos as size_t,
        );
        (*mf).action = LZMA_SYNC_FLUSH;
        ((*mf).skip)(mf, (*mf).write_pos);
    }
    (*mf).action = LZMA_RUN;
    false
}
pub fn lzma_lz_encoder_memusage(lz_options: &lzma_lz_options) -> u64 {
    let mut mf: lzma_mf = lzma_mf_s {
        buffer: core::ptr::null_mut(),
        size: 0,
        keep_size_before: 0,
        keep_size_after: 0,
        offset: 0,
        read_pos: 0,
        read_ahead: 0,
        read_limit: 0,
        write_pos: 0,
        pending: 0,
        find: lzma_mf_find_uninitialized,
        skip: lzma_mf_skip_uninitialized,
        hash: core::ptr::null_mut(),
        son: core::ptr::null_mut(),
        cyclic_pos: 0,
        cyclic_size: 0,
        hash_mask: 0,
        depth: 0,
        nice_len: 0,
        match_len_max: 0,
        action: LZMA_RUN,
        hash_count: 0,
        sons_count: 0,
    };
    if unsafe { lz_encoder_prepare(::core::ptr::addr_of_mut!(mf), core::ptr::null(), lz_options) } {
        return UINT64_MAX;
    }
    // mf.size, not mf.size + LZMA_MEMCMPLEN_EXTRA: lz_encoder.c reports the
    // same figure and leaves the tail padding out of the estimate. The buffer
    // is still allocated with the padding, so this under-reports by EXTRA
    // bytes exactly as the C original does.
    ((mf.hash_count as u64) + (mf.sons_count as u64)) * core::mem::size_of::<u32>() as u64
        + mf.size as u64
        + core::mem::size_of::<lzma_coder>() as u64
}
unsafe fn lz_encoder_end(coder: &mut lzma_coder, allocator: *const lzma_allocator) {
    lzma_next_end(::core::ptr::addr_of_mut!(coder.next), allocator);
    crate::alloc::internal_free_array(coder.mf.son, coder.mf.sons_count as size_t, allocator);
    crate::alloc::internal_free_array(coder.mf.hash, coder.mf.hash_count as size_t, allocator);
    crate::alloc::internal_free_array(
        coder.mf.buffer,
        (coder.mf.size + LZMA_MEMCMPLEN_EXTRA) as size_t,
        allocator,
    );
    coder.lz.end(allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn lz_encoder_update(
    coder: &mut lzma_coder,
    allocator: *const lzma_allocator,
    _filters_null: *const lzma_filter,
    reversed_filters: *const lzma_filter,
) -> lzma_ret {
    let ret_: lzma_ret = coder.lz.options_update(reversed_filters);
    if ret_ != LZMA_OK {
        return ret_;
    }
    lzma_next_filter_update(
        ::core::ptr::addr_of_mut!(coder.next),
        allocator,
        reversed_filters.offset(1),
    )
}
unsafe fn lz_encoder_set_out_limit(
    coder: &mut lzma_coder,
    uncomp_size: *mut u64,
    out_limit: u64,
) -> lzma_ret {
    if coder.next.coder.is_none() {
        if let Some(ret) = coder.lz.set_out_limit(uncomp_size, out_limit) {
            return ret;
        }
    }
    LZMA_OPTIONS_ERROR
}
pub unsafe fn lzma_lz_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
    lz_init: unsafe fn(
        *mut lzma_lz_encoder,
        *const lzma_allocator,
        lzma_vli,
        *const c_void,
        *mut lzma_lz_options,
    ) -> lzma_ret,
) -> lzma_ret {
    let mut coder: *mut lzma_coder = (*next).coder_as::<lzma_coder>();
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).set_coder(coder);
        (*coder).lz = lzma_lz_encoder::Uninitialized;
        (*coder).mf.buffer = core::ptr::null_mut();
        (*coder).mf.size = 0;
        (*coder).mf.hash = core::ptr::null_mut();
        (*coder).mf.son = core::ptr::null_mut();
        (*coder).mf.hash_count = 0;
        (*coder).mf.sons_count = 0;
        (*coder).next = LZMA_NEXT_CODER_INIT;
    }
    let mut lz_options: lzma_lz_options = lzma_lz_options {
        before_size: 0,
        dict_size: 0,
        after_size: 0,
        match_len_max: 0,
        nice_len: 0,
        match_finder: 0,
        depth: 0,
        preset_dict: core::ptr::null(),
        preset_dict_size: 0,
    };
    let ret_: lzma_ret = lz_init(
        ::core::ptr::addr_of_mut!((*coder).lz),
        allocator,
        (*filters).id,
        (*filters).options,
        ::core::ptr::addr_of_mut!(lz_options),
    );
    if ret_ != LZMA_OK {
        return ret_;
    }
    if matches!((*coder).lz, lzma_lz_encoder::Uninitialized) {
        return LZMA_PROG_ERROR;
    }
    if lz_encoder_prepare(
        ::core::ptr::addr_of_mut!((*coder).mf),
        allocator,
        ::core::ptr::addr_of_mut!(lz_options),
    ) {
        return LZMA_OPTIONS_ERROR;
    }
    if lz_encoder_init(
        ::core::ptr::addr_of_mut!((*coder).mf),
        allocator,
        ::core::ptr::addr_of_mut!(lz_options),
    ) {
        return LZMA_MEM_ERROR;
    }
    lzma_next_filter_init(
        ::core::ptr::addr_of_mut!((*coder).next),
        allocator,
        filters.offset(1),
    )
}
impl NextCoder for lzma_coder {
    unsafe fn code(
        &mut self,
        allocator: *const lzma_allocator,
        input: *const u8,
        in_pos: *mut size_t,
        in_size: size_t,
        out: *mut u8,
        out_pos: *mut size_t,
        out_size: size_t,
        action: lzma_action,
    ) -> lzma_ret {
        lz_encode(
            self, allocator, input, in_pos, in_size, out, out_pos, out_size, action,
        )
    }
    unsafe fn end(&mut self, allocator: *const lzma_allocator) {
        lz_encoder_end(self, allocator)
    }
    fn can_update(&self) -> bool {
        true
    }
    unsafe fn update(
        &mut self,
        allocator: *const lzma_allocator,
        filters: *const lzma_filter,
        reversed_filters: *const lzma_filter,
    ) -> lzma_ret {
        lz_encoder_update(self, allocator, filters, reversed_filters)
    }
    unsafe fn set_out_limit(&mut self, uncomp_size: *mut u64, out_limit: u64) -> Option<lzma_ret> {
        Some(lz_encoder_set_out_limit(self, uncomp_size, out_limit))
    }
}
pub fn lzma_mf_is_supported(mf: lzma_match_finder) -> lzma_bool {
    match mf {
        3 => return true as lzma_bool,
        4 => return true as lzma_bool,
        18 => return true as lzma_bool,
        19 => return true as lzma_bool,
        20 => return true as lzma_bool,
        _ => return false as lzma_bool,
    };
}
