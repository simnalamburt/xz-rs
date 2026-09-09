use crate::common::alone_decoder::lzma_alone_decoder_init;
use crate::common::lzip_decoder::lzma_lzip_decoder_init;
use crate::types::*;
pub type auto_decoder_seq = c_uint;
pub const SEQ_FINISH: auto_decoder_seq = 2;
pub const SEQ_CODE: auto_decoder_seq = 1;
pub const SEQ_INIT: auto_decoder_seq = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_auto_coder {
    pub next: lzma_next_coder,
    pub memlimit: u64,
    pub flags: u32,
    pub sequence: auto_decoder_seq,
}
unsafe fn auto_decode(
    coder_ptr: *mut c_void,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    action: lzma_action,
) -> lzma_ret {
    let coder: *mut lzma_auto_coder = coder_ptr as *mut lzma_auto_coder;
    match (*coder).sequence {
        0 => {
            if *in_pos >= in_size {
                return LZMA_OK;
            }
            (*coder).sequence = SEQ_CODE;
            if *input.add(*in_pos) == 0xfd {
                let ret: lzma_ret = lzma_stream_decoder_init(
                    ::core::ptr::addr_of_mut!((*coder).next),
                    allocator,
                    (*coder).memlimit,
                    (*coder).flags,
                );
                if ret != LZMA_OK {
                    return ret;
                }
            } else if *input.add(*in_pos) == 0x4c {
                let ret: lzma_ret = lzma_lzip_decoder_init(
                    ::core::ptr::addr_of_mut!((*coder).next),
                    allocator,
                    (*coder).memlimit,
                    (*coder).flags,
                );
                if ret != LZMA_OK {
                    return ret;
                }
            } else {
                let ret: lzma_ret = lzma_alone_decoder_init(
                    ::core::ptr::addr_of_mut!((*coder).next),
                    allocator,
                    (*coder).memlimit,
                    true,
                );
                if ret != LZMA_OK {
                    return ret;
                }
                if (*coder).flags & LZMA_TELL_NO_CHECK as u32 != 0 {
                    return LZMA_NO_CHECK;
                }
                if (*coder).flags & LZMA_TELL_ANY_CHECK as u32 != 0 {
                    return LZMA_GET_CHECK;
                }
            }
        }
        1 | 2 => {}
        _ => return LZMA_PROG_ERROR,
    }
    if (*coder).sequence != SEQ_FINISH {
        debug_assert!((*coder).next.code.is_some());
        let code = (*coder).next.code.unwrap_unchecked();
        let ret: lzma_ret = code(
            (*coder).next.coder,
            allocator,
            input,
            in_pos,
            in_size,
            out,
            out_pos,
            out_size,
            action,
        );
        if ret != LZMA_STREAM_END || (*coder).flags & LZMA_CONCATENATED as u32 == 0 {
            return ret;
        }
        (*coder).sequence = SEQ_FINISH;
    }
    if *in_pos < in_size {
        return LZMA_DATA_ERROR;
    }
    if action == LZMA_FINISH {
        LZMA_STREAM_END
    } else {
        LZMA_OK
    }
}
unsafe fn auto_decoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_auto_coder = coder_ptr as *mut lzma_auto_coder;
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).next), allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn auto_decoder_get_check(coder_ptr: *const c_void) -> lzma_check {
    let coder: *const lzma_auto_coder = coder_ptr as *const lzma_auto_coder;
    match (*coder).next.get_check {
        Some(get_check) => get_check((*coder).next.coder),
        None => LZMA_CHECK_NONE,
    }
}
pub unsafe fn lzma_auto_decoder(strm: *mut lzma_stream, memlimit: u64, flags: u32) -> lzma_ret {
    let ret: lzma_ret = lzma_strm_init(strm);
    if ret != LZMA_OK {
        return ret;
    }
    let ret: lzma_ret = auto_decoder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        memlimit,
        flags,
    );
    if ret != LZMA_OK {
        lzma_end(strm);
        return ret;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
unsafe fn auto_decoder_memconfig(
    coder_ptr: *mut c_void,
    memusage: *mut u64,
    old_memlimit: *mut u64,
    new_memlimit: u64,
) -> lzma_ret {
    let coder: *mut lzma_auto_coder = coder_ptr as *mut lzma_auto_coder;
    let mut ret: lzma_ret = LZMA_OK;
    if let Some(memconfig) = (*coder).next.memconfig {
        ret = memconfig((*coder).next.coder, memusage, old_memlimit, new_memlimit);
    } else {
        *memusage = LZMA_MEMUSAGE_BASE;
        *old_memlimit = (*coder).memlimit;
        ret = LZMA_OK;
        if new_memlimit != 0 && new_memlimit < *memusage {
            ret = LZMA_MEMLIMIT_ERROR;
        }
    }
    if ret == LZMA_OK && new_memlimit != 0 {
        (*coder).memlimit = new_memlimit;
    }
    ret
}
unsafe fn auto_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    memlimit: u64,
    flags: u32,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret>,
        uintptr_t,
    >(Some(
        auto_decoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret>,
        uintptr_t,
    >(Some(
        auto_decoder_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, u64, u32) -> lzma_ret,
    ));
    if flags & !(LZMA_SUPPORTED_FLAGS as u32) != 0 {
        return LZMA_OPTIONS_ERROR;
    }
    let mut coder: *mut lzma_auto_coder = (*next).coder as *mut lzma_auto_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_auto_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        (*next).code = Some(
            auto_decode
                as unsafe fn(
                    *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    *mut size_t,
                    size_t,
                    *mut u8,
                    *mut size_t,
                    size_t,
                    lzma_action,
                ) -> lzma_ret,
        );
        (*next).end = Some(auto_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*next).get_check = Some(auto_decoder_get_check as unsafe fn(*const c_void) -> lzma_check);
        (*next).memconfig = Some(
            auto_decoder_memconfig as unsafe fn(*mut c_void, *mut u64, *mut u64, u64) -> lzma_ret,
        );
        (*coder).next = lzma_next_coder_s {
            coder: core::ptr::null_mut(),
            id: LZMA_VLI_UNKNOWN,
            init: 0,
            code: None,
            end: None,
            get_progress: None,
            get_check: None,
            memconfig: None,
            update: None,
            set_out_limit: None,
        };
    }
    (*coder).memlimit = if 1 > memlimit { 1 } else { memlimit };
    (*coder).flags = flags;
    (*coder).sequence = SEQ_INIT;
    LZMA_OK
}
