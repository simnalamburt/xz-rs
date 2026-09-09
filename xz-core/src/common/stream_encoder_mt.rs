use crate::common::block_buffer_encoder::{lzma_block_buffer_bound64, lzma_block_uncomp_encode};
use crate::common::filter_encoder::lzma_mt_block_size;
use crate::common::outqueue::lzma_outq_memusage;
use crate::types::*;
type worker_thread = worker_thread_s;
#[derive(Copy, Clone)]
#[repr(C)]
struct worker_thread_s {
    pub state: worker_state,
    pub in_0: *mut u8,
    pub in_alloc_size: size_t,
    pub in_size: size_t,
    pub outbuf: *mut lzma_outbuf,
    pub coder: *mut lzma_stream_coder,
    #[cfg(feature = "custom_allocator")]
    pub allocator: *const lzma_allocator,
    pub progress_in: u64,
    pub progress_out: u64,
    pub block_encoder: lzma_next_coder,
    pub block_options: lzma_block,
    pub filters: [lzma_filter; 5],
    pub next: *mut worker_thread,
    pub mutex: mythread_mutex,
    pub cond: mythread_cond,
    pub thread_id: mythread,
}
#[inline]
unsafe fn worker_allocator(thr: *const worker_thread) -> *const lzma_allocator {
    #[cfg(feature = "custom_allocator")]
    {
        unsafe { (*thr).allocator }
    }
    #[cfg(not(feature = "custom_allocator"))]
    {
        let _ = thr;
        core::ptr::null()
    }
}
#[inline]
unsafe fn set_worker_allocator(thr: *mut worker_thread, allocator: *const lzma_allocator) {
    #[cfg(feature = "custom_allocator")]
    {
        unsafe {
            (*thr).allocator = allocator;
        }
    }
    #[cfg(not(feature = "custom_allocator"))]
    {
        let _ = (thr, allocator);
    }
}
type lzma_stream_coder = lzma_stream_coder_s;
#[derive(Copy, Clone)]
#[repr(C)]
struct lzma_stream_coder_s {
    pub sequence: stream_encoder_mt_seq,
    pub block_size: size_t,
    pub filters: [lzma_filter; 5],
    pub filters_cache: [lzma_filter; 5],
    pub index: *mut lzma_index,
    pub index_encoder: lzma_next_coder,
    pub stream_flags: lzma_stream_flags,
    pub header: [u8; 12],
    pub header_pos: size_t,
    pub outq: lzma_outq,
    pub outbuf_alloc_size: size_t,
    pub timeout: u32,
    pub thread_error: lzma_ret,
    pub threads: *mut worker_thread,
    pub threads_max: u32,
    pub threads_initialized: u32,
    pub threads_free: *mut worker_thread,
    pub thr: *mut worker_thread,
    pub progress_in: u64,
    pub progress_out: u64,
    pub mutex: mythread_mutex,
    pub cond: mythread_cond,
}
pub type stream_encoder_mt_seq = c_uint;
pub const SEQ_STREAM_FOOTER: stream_encoder_mt_seq = 3;
pub const SEQ_INDEX: stream_encoder_mt_seq = 2;
pub const SEQ_BLOCK: stream_encoder_mt_seq = 1;
pub const SEQ_STREAM_HEADER: stream_encoder_mt_seq = 0;
pub const THR_EXIT: worker_state = 4;
pub const THR_STOP: worker_state = 3;
pub const THR_FINISH: worker_state = 2;
pub const BLOCK_SIZE_MAX: c_ulonglong = UINT64_MAX.wrapping_div(LZMA_THREADS_MAX as u64);
unsafe fn worker_error(thr: *mut worker_thread, ret: lzma_ret) {
    let mut mythread_i_207: c_uint = 0;
    while if mythread_i_207 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*(*thr).coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*(*thr).coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_207: c_uint = 0;
        while mythread_j_207 == 0 {
            if (*(*thr).coder).thread_error == LZMA_OK {
                (*(*thr).coder).thread_error = ret;
            }
            mythread_cond_signal(::core::ptr::addr_of_mut!((*(*thr).coder).cond));
            mythread_j_207 = 1;
        }
        mythread_i_207 = 1;
    }
}
unsafe fn worker_encode(
    thr: *mut worker_thread,
    out_pos: *mut size_t,
    mut state: worker_state,
) -> worker_state {
    (*thr).block_options = lzma_block {
        version: 0,
        header_size: 0,
        check: (*(*thr).coder).stream_flags.check,
        compressed_size: (*(*thr).outbuf).allocated as lzma_vli,
        uncompressed_size: (*(*thr).coder).block_size as lzma_vli,
        filters: ::core::ptr::addr_of_mut!((*thr).filters) as *mut lzma_filter,
        raw_check: [0; 64],
        reserved_ptr1: core::ptr::null_mut(),
        reserved_ptr2: core::ptr::null_mut(),
        reserved_ptr3: core::ptr::null_mut(),
        reserved_int1: 0,
        reserved_int2: 0,
        reserved_int3: 0,
        reserved_int4: 0,
        reserved_int5: 0,
        reserved_int6: 0,
        reserved_int7: 0,
        reserved_int8: 0,
        reserved_enum1: LZMA_RESERVED_ENUM,
        reserved_enum2: LZMA_RESERVED_ENUM,
        reserved_enum3: LZMA_RESERVED_ENUM,
        reserved_enum4: LZMA_RESERVED_ENUM,
        ignore_check: 0,
        reserved_bool2: 0,
        reserved_bool3: 0,
        reserved_bool4: 0,
        reserved_bool5: 0,
        reserved_bool6: 0,
        reserved_bool7: 0,
        reserved_bool8: 0,
    };
    let mut ret: lzma_ret = lzma_block_header_size(&mut (*thr).block_options);
    if ret != LZMA_OK {
        worker_error(thr, ret);
        return THR_STOP;
    }
    ret = lzma_block_encoder_init(
        ::core::ptr::addr_of_mut!((*thr).block_encoder),
        worker_allocator(thr),
        ::core::ptr::addr_of_mut!((*thr).block_options),
    );
    if ret != LZMA_OK {
        worker_error(thr, ret);
        return THR_STOP;
    }
    let mut in_pos: size_t = 0;
    let mut in_size: size_t = 0;
    *out_pos = (*thr).block_options.header_size as size_t;
    let out_size: size_t = (*(*thr).outbuf).allocated;
    loop {
        let mut mythread_i_258: c_uint = 0;
        while if mythread_i_258 != 0 {
            mythread_mutex_unlock(::core::ptr::addr_of_mut!((*thr).mutex));
            0
        } else {
            mythread_mutex_lock(::core::ptr::addr_of_mut!((*thr).mutex));
            1
        } != 0
        {
            let mut mythread_j_258: c_uint = 0;
            while mythread_j_258 == 0 {
                (*thr).progress_in = in_pos as u64;
                (*thr).progress_out = *out_pos as u64;
                while in_size == (*thr).in_size && (*thr).state == THR_RUN {
                    mythread_cond_wait(
                        ::core::ptr::addr_of_mut!((*thr).cond),
                        ::core::ptr::addr_of_mut!((*thr).mutex),
                    );
                }
                state = (*thr).state;
                in_size = (*thr).in_size;
                mythread_j_258 = 1;
            }
            mythread_i_258 = 1;
        }
        if state >= THR_STOP {
            return state;
        }
        let mut action: lzma_action = (if state == THR_FINISH {
            LZMA_FINISH
        } else {
            LZMA_RUN
        }) as lzma_action;
        const IN_CHUNK_MAX: size_t = 16384;
        let mut in_limit: size_t = in_size;
        if in_size - in_pos > IN_CHUNK_MAX {
            in_limit = in_pos + IN_CHUNK_MAX;
            action = LZMA_RUN;
        }
        let code = match (*thr).block_encoder.code {
            Some(code) => code,
            None => {
                worker_error(thr, LZMA_PROG_ERROR);
                return THR_STOP;
            }
        };
        ret = code(
            (*thr).block_encoder.coder,
            worker_allocator(thr),
            (*thr).in_0,
            ::core::ptr::addr_of_mut!(in_pos),
            in_limit,
            ::core::ptr::addr_of_mut!((*(*thr).outbuf).buf) as *mut u8,
            out_pos,
            out_size,
            action,
        );
        if ret != LZMA_OK || *out_pos >= out_size {
            break;
        }
    }
    match ret {
        1 => {
            let outbuf = (*thr).outbuf;
            ret = lzma_block_header_encode(
                &(*thr).block_options,
                c_slice_mut(
                    ::core::ptr::addr_of_mut!((*outbuf).buf) as *mut u8,
                    (*outbuf).allocated,
                ),
            );
            if ret != LZMA_OK {
                worker_error(thr, ret);
                return THR_STOP;
            }
        }
        0 => {
            let mut mythread_i_321: c_uint = 0;
            while if mythread_i_321 != 0 {
                mythread_mutex_unlock(::core::ptr::addr_of_mut!((*thr).mutex));
                0
            } else {
                mythread_mutex_lock(::core::ptr::addr_of_mut!((*thr).mutex));
                1
            } != 0
            {
                let mut mythread_j_321: c_uint = 0;
                while mythread_j_321 == 0 {
                    while (*thr).state == THR_RUN {
                        mythread_cond_wait(
                            ::core::ptr::addr_of_mut!((*thr).cond),
                            ::core::ptr::addr_of_mut!((*thr).mutex),
                        );
                    }
                    state = (*thr).state;
                    in_size = (*thr).in_size;
                    mythread_j_321 = 1;
                }
                mythread_i_321 = 1;
            }
            if state >= THR_STOP {
                return state;
            }
            *out_pos = 0;
            ret = lzma_block_uncomp_encode(
                ::core::ptr::addr_of_mut!((*thr).block_options),
                (*thr).in_0,
                in_size,
                ::core::ptr::addr_of_mut!((*(*thr).outbuf).buf) as *mut u8,
                out_pos,
                out_size,
            );
            if ret != LZMA_OK {
                worker_error(thr, LZMA_PROG_ERROR);
                return THR_STOP;
            }
        }
        _ => {
            worker_error(thr, ret);
            return THR_STOP;
        }
    }
    (*(*thr).outbuf).unpadded_size =
        lzma_block_unpadded_size(::core::ptr::addr_of_mut!((*thr).block_options));
    (*(*thr).outbuf).uncompressed_size = (*thr).block_options.uncompressed_size;
    THR_FINISH
}
unsafe extern "C" fn worker_start(thr_ptr: *mut c_void) -> *mut c_void {
    let thr: *mut worker_thread = thr_ptr as *mut worker_thread;
    let mut state: worker_state = THR_IDLE;
    loop {
        let mut mythread_i_370: c_uint = 0;
        while if mythread_i_370 != 0 {
            mythread_mutex_unlock(::core::ptr::addr_of_mut!((*thr).mutex));
            0
        } else {
            mythread_mutex_lock(::core::ptr::addr_of_mut!((*thr).mutex));
            1
        } != 0
        {
            let mut mythread_j_370: c_uint = 0;
            while mythread_j_370 == 0 {
                loop {
                    if (*thr).state == THR_STOP {
                        (*thr).state = THR_IDLE;
                        mythread_cond_signal(::core::ptr::addr_of_mut!((*thr).cond));
                    }
                    state = (*thr).state;
                    if state != THR_IDLE {
                        break;
                    }
                    mythread_cond_wait(
                        ::core::ptr::addr_of_mut!((*thr).cond),
                        ::core::ptr::addr_of_mut!((*thr).mutex),
                    );
                }
                mythread_j_370 = 1;
            }
            mythread_i_370 = 1;
        }
        let mut out_pos: size_t = 0;
        if state <= THR_FINISH {
            state = worker_encode(thr, ::core::ptr::addr_of_mut!(out_pos), state);
        }
        if state == THR_EXIT {
            break;
        }
        let mut mythread_i_401: c_uint = 0;
        while if mythread_i_401 != 0 {
            mythread_mutex_unlock(::core::ptr::addr_of_mut!((*thr).mutex));
            0
        } else {
            mythread_mutex_lock(::core::ptr::addr_of_mut!((*thr).mutex));
            1
        } != 0
        {
            let mut mythread_j_401: c_uint = 0;
            while mythread_j_401 == 0 {
                if (*thr).state != THR_EXIT {
                    (*thr).state = THR_IDLE;
                    mythread_cond_signal(::core::ptr::addr_of_mut!((*thr).cond));
                }
                mythread_j_401 = 1;
            }
            mythread_i_401 = 1;
        }
        let mut mythread_i_408: c_uint = 0;
        while if mythread_i_408 != 0 {
            mythread_mutex_unlock(::core::ptr::addr_of_mut!((*(*thr).coder).mutex));
            0
        } else {
            mythread_mutex_lock(::core::ptr::addr_of_mut!((*(*thr).coder).mutex));
            1
        } != 0
        {
            let mut mythread_j_408: c_uint = 0;
            while mythread_j_408 == 0 {
                if state == THR_FINISH {
                    (*(*thr).outbuf).pos = out_pos;
                    (*(*thr).outbuf).finished = true;
                }
                (*(*thr).coder).progress_in += (*(*thr).outbuf).uncompressed_size as u64;
                (*(*thr).coder).progress_out += out_pos as u64;
                (*thr).progress_in = 0;
                (*thr).progress_out = 0;
                (*thr).next = (*(*thr).coder).threads_free;
                (*(*thr).coder).threads_free = thr;
                mythread_cond_signal(::core::ptr::addr_of_mut!((*(*thr).coder).cond));
                mythread_j_408 = 1;
            }
            mythread_i_408 = 1;
        }
    }
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*thr).filters) as *mut lzma_filter,
        worker_allocator(thr),
    );
    mythread_mutex_destroy(::core::ptr::addr_of_mut!((*thr).mutex));
    mythread_cond_destroy(::core::ptr::addr_of_mut!((*thr).cond));
    lzma_next_end(
        ::core::ptr::addr_of_mut!((*thr).block_encoder),
        worker_allocator(thr),
    );
    crate::alloc::internal_free_array((*thr).in_0, (*thr).in_alloc_size, worker_allocator(thr));
    MYTHREAD_RET_VALUE
}
unsafe fn threads_stop(coder: *mut lzma_stream_coder, wait_for_threads: bool) {
    let mut i: u32 = 0;
    while i < (*coder).threads_initialized {
        let mut mythread_i_449: c_uint = 0;
        while if mythread_i_449 != 0 {
            mythread_mutex_unlock(::core::ptr::addr_of_mut!(
                (*(*coder).threads.offset(i as isize)).mutex
            ));
            0
        } else {
            mythread_mutex_lock(::core::ptr::addr_of_mut!(
                (*(*coder).threads.offset(i as isize)).mutex
            ));
            1
        } != 0
        {
            let mut mythread_j_449: c_uint = 0;
            while mythread_j_449 == 0 {
                (*(*coder).threads.offset(i as isize)).state = THR_STOP;
                mythread_cond_signal(::core::ptr::addr_of_mut!(
                    (*(*coder).threads.offset(i as isize)).cond
                ));
                mythread_j_449 = 1;
            }
            mythread_i_449 = 1;
        }
        i += 1;
    }
    if !wait_for_threads {
        return;
    }
    let mut i_0: u32 = 0;
    while i_0 < (*coder).threads_initialized {
        let mut mythread_i_460: c_uint = 0;
        while if mythread_i_460 != 0 {
            mythread_mutex_unlock(::core::ptr::addr_of_mut!(
                (*(*coder).threads.offset(i_0 as isize)).mutex
            ));
            0
        } else {
            mythread_mutex_lock(::core::ptr::addr_of_mut!(
                (*(*coder).threads.offset(i_0 as isize)).mutex
            ));
            1
        } != 0
        {
            let mut mythread_j_460: c_uint = 0;
            while mythread_j_460 == 0 {
                while (*(*coder).threads.offset(i_0 as isize)).state != THR_IDLE {
                    mythread_cond_wait(
                        ::core::ptr::addr_of_mut!((*(*coder).threads.offset(i_0 as isize)).cond),
                        ::core::ptr::addr_of_mut!((*(*coder).threads.offset(i_0 as isize)).mutex),
                    );
                }
                mythread_j_460 = 1;
            }
            mythread_i_460 = 1;
        }
        i_0 += 1;
    }
}
unsafe fn threads_end(coder: *mut lzma_stream_coder, allocator: *const lzma_allocator) {
    let mut i: u32 = 0;
    while i < (*coder).threads_initialized {
        let mut mythread_i_477: c_uint = 0;
        while if mythread_i_477 != 0 {
            mythread_mutex_unlock(::core::ptr::addr_of_mut!(
                (*(*coder).threads.offset(i as isize)).mutex
            ));
            0
        } else {
            mythread_mutex_lock(::core::ptr::addr_of_mut!(
                (*(*coder).threads.offset(i as isize)).mutex
            ));
            1
        } != 0
        {
            let mut mythread_j_477: c_uint = 0;
            while mythread_j_477 == 0 {
                (*(*coder).threads.offset(i as isize)).state = THR_EXIT;
                mythread_cond_signal(::core::ptr::addr_of_mut!(
                    (*(*coder).threads.offset(i as isize)).cond
                ));
                mythread_j_477 = 1;
            }
            mythread_i_477 = 1;
        }
        i += 1;
    }
    let mut i_0: u32 = 0;
    while i_0 < (*coder).threads_initialized {
        let _ret: c_int = mythread_join((*(*coder).threads.offset(i_0 as isize)).thread_id);
        i_0 += 1;
    }
    crate::alloc::internal_free_array((*coder).threads, (*coder).threads_max as size_t, allocator);
}
unsafe fn initialize_new_thread(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    let thr: *mut worker_thread = (*coder)
        .threads
        .offset((*coder).threads_initialized as isize)
        as *mut worker_thread;
    (*thr).in_0 = crate::alloc::internal_alloc_array::<u8>((*coder).block_size, allocator);
    if (*thr).in_0.is_null() {
        return LZMA_MEM_ERROR;
    }
    (*thr).in_alloc_size = (*coder).block_size;
    if mythread_mutex_init(::core::ptr::addr_of_mut!((*thr).mutex)) == 0 {
        if mythread_cond_init(::core::ptr::addr_of_mut!((*thr).cond)) == 0 {
            (*thr).state = THR_IDLE;
            set_worker_allocator(thr, allocator);
            (*thr).coder = coder;
            (*thr).progress_in = 0;
            (*thr).progress_out = 0;
            (*thr).block_encoder = lzma_next_coder_s {
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
            (*thr).filters[0].id = LZMA_VLI_UNKNOWN;
            if mythread_create(
                ::core::ptr::addr_of_mut!((*thr).thread_id),
                worker_start as unsafe extern "C" fn(*mut c_void) -> *mut c_void,
                thr as *mut c_void,
            ) != 0
            {
                mythread_cond_destroy(::core::ptr::addr_of_mut!((*thr).cond));
            } else {
                (*coder).threads_initialized += 1;
                (*coder).thr = thr;
                return LZMA_OK;
            }
        }
        mythread_mutex_destroy(::core::ptr::addr_of_mut!((*thr).mutex));
    }
    crate::alloc::internal_free_array((*thr).in_0, (*thr).in_alloc_size, allocator);
    LZMA_MEM_ERROR
}
unsafe fn get_thread(coder: *mut lzma_stream_coder, allocator: *const lzma_allocator) -> lzma_ret {
    if !lzma_outq_has_buf(&(*coder).outq) {
        return LZMA_OK;
    }
    let ret_: lzma_ret = lzma_outq_prealloc_buf(
        ::core::ptr::addr_of_mut!((*coder).outq),
        allocator,
        (*coder).outbuf_alloc_size,
    );
    if ret_ != LZMA_OK {
        return ret_;
    }
    if (*coder).filters_cache[0].id == LZMA_VLI_UNKNOWN {
        let ret__0: lzma_ret = lzma_filters_copy(
            ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
            ::core::ptr::addr_of_mut!((*coder).filters_cache) as *mut lzma_filter,
            allocator,
        );
        if ret__0 != LZMA_OK {
            return ret__0;
        }
    }
    let mut mythread_i_560: c_uint = 0;
    while if mythread_i_560 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_560: c_uint = 0;
        while mythread_j_560 == 0 {
            if !(*coder).threads_free.is_null() {
                (*coder).thr = (*coder).threads_free;
                (*coder).threads_free = (*(*coder).threads_free).next;
            }
            mythread_j_560 = 1;
        }
        mythread_i_560 = 1;
    }
    if (*coder).thr.is_null() {
        if (*coder).threads_initialized == (*coder).threads_max {
            return LZMA_OK;
        }
        let ret__1: lzma_ret = initialize_new_thread(coder, allocator);
        if ret__1 != LZMA_OK {
            return ret__1;
        }
    }
    let mut mythread_i_578: c_uint = 0;
    while if mythread_i_578 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*(*coder).thr).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*(*coder).thr).mutex));
        1
    } != 0
    {
        let mut mythread_j_578: c_uint = 0;
        while mythread_j_578 == 0 {
            (*(*coder).thr).state = THR_RUN;
            (*(*coder).thr).in_size = 0;
            (*(*coder).thr).outbuf = lzma_outq_get_buf(
                ::core::ptr::addr_of_mut!((*coder).outq),
                core::ptr::null_mut(),
            );
            lzma_filters_free(
                ::core::ptr::addr_of_mut!((*(*coder).thr).filters) as *mut lzma_filter,
                allocator,
            );
            core::ptr::copy_nonoverlapping(
                ::core::ptr::addr_of_mut!((*coder).filters_cache) as *const u8,
                ::core::ptr::addr_of_mut!((*(*coder).thr).filters) as *mut u8,
                core::mem::size_of::<[lzma_filter; 5]>(),
            );
            (*coder).filters_cache[0].id = LZMA_VLI_UNKNOWN;
            mythread_cond_signal(::core::ptr::addr_of_mut!((*(*coder).thr).cond));
            mythread_j_578 = 1;
        }
        mythread_i_578 = 1;
    }
    LZMA_OK
}
unsafe fn stream_encode_in(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
    in_0: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    action: lzma_action,
) -> lzma_ret {
    while *in_pos < in_size || !(*coder).thr.is_null() && action != LZMA_RUN {
        if (*coder).thr.is_null() {
            let ret: lzma_ret = get_thread(coder, allocator);
            if (*coder).thr.is_null() {
                return ret;
            }
        }
        let mut thr_in_size: size_t = (*(*coder).thr).in_size;
        lzma_bufcpy(
            in_0,
            in_pos,
            in_size,
            (*(*coder).thr).in_0,
            ::core::ptr::addr_of_mut!(thr_in_size),
            (*coder).block_size,
        );
        let finish: bool =
            thr_in_size == (*coder).block_size || *in_pos == in_size && action != LZMA_RUN;
        let mut block_error: bool = false;
        let mut mythread_i_628: c_uint = 0;
        while if mythread_i_628 != 0 {
            mythread_mutex_unlock(::core::ptr::addr_of_mut!((*(*coder).thr).mutex));
            0
        } else {
            mythread_mutex_lock(::core::ptr::addr_of_mut!((*(*coder).thr).mutex));
            1
        } != 0
        {
            let mut mythread_j_628: c_uint = 0;
            while mythread_j_628 == 0 {
                if (*(*coder).thr).state == THR_IDLE {
                    block_error = true;
                } else {
                    (*(*coder).thr).in_size = thr_in_size;
                    if finish {
                        (*(*coder).thr).state = THR_FINISH;
                    }
                    mythread_cond_signal(::core::ptr::addr_of_mut!((*(*coder).thr).cond));
                }
                mythread_j_628 = 1;
            }
            mythread_i_628 = 1;
        }
        if block_error {
            let mut ret_0: lzma_ret = LZMA_OK;
            let mut mythread_i_649: c_uint = 0;
            while if mythread_i_649 != 0 {
                mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
                0
            } else {
                mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
                1
            } != 0
            {
                let mut mythread_j_649: c_uint = 0;
                while mythread_j_649 == 0 {
                    ret_0 = (*coder).thread_error;
                    mythread_j_649 = 1;
                }
                mythread_i_649 = 1;
            }
            return ret_0;
        }
        if finish {
            (*coder).thr = core::ptr::null_mut();
        }
    }
    LZMA_OK
}
unsafe fn wait_for_work(
    coder: *mut lzma_stream_coder,
    wait_abs: *mut mythread_condtime,
    has_blocked: *mut bool,
    has_input: bool,
) -> bool {
    if (*coder).timeout != 0 && !*has_blocked {
        *has_blocked = true;
        mythread_condtime_set(
            wait_abs,
            ::core::ptr::addr_of_mut!((*coder).cond),
            (*coder).timeout,
        );
    }
    let mut timed_out: bool = false;
    let mut mythread_i_689: c_uint = 0;
    while if mythread_i_689 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_689: c_uint = 0;
        while mythread_j_689 == 0 {
            while (!has_input
                || (*coder).threads_free.is_null()
                || !lzma_outq_has_buf(&(*coder).outq))
                && !lzma_outq_is_readable(::core::ptr::addr_of_mut!((*coder).outq))
                && (*coder).thread_error == LZMA_OK
                && !timed_out
            {
                if (*coder).timeout != 0 {
                    timed_out = mythread_cond_timedwait(
                        ::core::ptr::addr_of_mut!((*coder).cond),
                        ::core::ptr::addr_of_mut!((*coder).mutex),
                        wait_abs,
                    ) != 0;
                } else {
                    mythread_cond_wait(
                        ::core::ptr::addr_of_mut!((*coder).cond),
                        ::core::ptr::addr_of_mut!((*coder).mutex),
                    );
                }
            }
            mythread_j_689 = 1;
        }
        mythread_i_689 = 1;
    }
    timed_out
}

#[inline(never)]
unsafe fn stream_encode_mt_blocks(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
    in_0: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    action: lzma_action,
) -> lzma_ret {
    let mut unpadded_size: lzma_vli = 0;
    let mut uncompressed_size: lzma_vli = 0;
    let mut ret: lzma_ret = LZMA_OK;
    let mut has_blocked: bool = false;
    let mut wait_abs: mythread_condtime = unsafe { core::mem::zeroed() };
    loop {
        let mut mythread_i_747: c_uint = 0;
        while if mythread_i_747 != 0 {
            mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
            0
        } else {
            mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
            1
        } != 0
        {
            let mut mythread_j_747: c_uint = 0;
            while mythread_j_747 == 0 {
                ret = (*coder).thread_error;
                if ret != LZMA_OK {
                    break;
                }
                ret = lzma_outq_read(
                    ::core::ptr::addr_of_mut!((*coder).outq),
                    allocator,
                    out,
                    out_pos,
                    out_size,
                    ::core::ptr::addr_of_mut!(unpadded_size),
                    ::core::ptr::addr_of_mut!(uncompressed_size),
                );
                mythread_j_747 = 1;
            }
            mythread_i_747 = 1;
        }
        if ret == LZMA_STREAM_END {
            ret = lzma_index_append((*coder).index, allocator, unpadded_size, uncompressed_size);
            if ret != LZMA_OK {
                threads_stop(coder, false);
                return ret;
            }
            if *out_pos < out_size {
                continue;
            }
        }
        if ret != LZMA_OK {
            threads_stop(coder, false);
            return ret;
        }
        ret = stream_encode_in(coder, allocator, in_0, in_pos, in_size, action);
        if ret != LZMA_OK {
            threads_stop(coder, false);
            return ret;
        }
        if *in_pos == in_size {
            if action == LZMA_RUN {
                return LZMA_OK;
            }
            if action == LZMA_FULL_BARRIER {
                return LZMA_STREAM_END;
            }
            if lzma_outq_is_empty(&(*coder).outq) {
                if action == LZMA_FINISH {
                    break;
                }
                if action == LZMA_FULL_FLUSH {
                    return LZMA_STREAM_END;
                }
            }
        }
        if *out_pos == out_size {
            return LZMA_OK;
        }
        if wait_for_work(
            coder,
            ::core::ptr::addr_of_mut!(wait_abs),
            ::core::ptr::addr_of_mut!(has_blocked),
            *in_pos < in_size,
        ) {
            return LZMA_RET_INTERNAL1;
        }
    }
    let ret_: lzma_ret = lzma_index_encoder_init(
        ::core::ptr::addr_of_mut!((*coder).index_encoder),
        allocator,
        (*coder).index,
    );
    if ret_ != LZMA_OK {
        return ret_;
    }
    (*coder).sequence = SEQ_INDEX;
    (*coder).progress_out +=
        (lzma_index_size(&*(*coder).index) + LZMA_STREAM_HEADER_SIZE as lzma_vli) as u64;
    LZMA_STREAM_END
}

#[inline(never)]
unsafe fn stream_encode_mt_index(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    let Some(code) = (*coder).index_encoder.code else {
        return LZMA_PROG_ERROR;
    };
    let ret: lzma_ret = code(
        (*coder).index_encoder.coder,
        allocator,
        core::ptr::null(),
        core::ptr::null_mut(),
        0,
        out,
        out_pos,
        out_size,
        LZMA_RUN,
    );
    if ret != LZMA_STREAM_END {
        return ret;
    }
    (*coder).stream_flags.backward_size = lzma_index_size(&*(*coder).index);
    if lzma_stream_footer_encode(&(*coder).stream_flags, &mut (*coder).header) != LZMA_OK {
        return LZMA_PROG_ERROR;
    }
    (*coder).sequence = SEQ_STREAM_FOOTER;
    LZMA_STREAM_END
}

unsafe fn stream_encode_mt(
    coder_ptr: *mut c_void,
    allocator: *const lzma_allocator,
    in_0: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    action: lzma_action,
) -> lzma_ret {
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    match (*coder).sequence {
        SEQ_STREAM_HEADER => {
            lzma_bufcpy(
                ::core::ptr::addr_of_mut!((*coder).header) as *mut u8,
                ::core::ptr::addr_of_mut!((*coder).header_pos),
                core::mem::size_of::<[u8; 12]>(),
                out,
                out_pos,
                out_size,
            );
            if (*coder).header_pos < core::mem::size_of::<[u8; 12]>() {
                return LZMA_OK;
            }
            (*coder).header_pos = 0;
            (*coder).sequence = SEQ_BLOCK;
        }
        SEQ_BLOCK | SEQ_INDEX | SEQ_STREAM_FOOTER => {}
        _ => return LZMA_PROG_ERROR,
    }
    if (*coder).sequence == SEQ_BLOCK {
        let ret: lzma_ret = stream_encode_mt_blocks(
            coder, allocator, in_0, in_pos, in_size, out, out_pos, out_size, action,
        );
        if ret != LZMA_STREAM_END {
            return ret;
        }
        // FullFlush/FullBarrier complete while staying in SEQ_BLOCK. Only
        // Finish advances to SEQ_INDEX and should fall through below.
        if (*coder).sequence == SEQ_BLOCK {
            return LZMA_STREAM_END;
        }
    }
    if (*coder).sequence == SEQ_INDEX {
        let ret: lzma_ret = stream_encode_mt_index(coder, allocator, out, out_pos, out_size);
        if ret != LZMA_STREAM_END {
            return ret;
        }
    }
    lzma_bufcpy(
        ::core::ptr::addr_of_mut!((*coder).header) as *mut u8,
        ::core::ptr::addr_of_mut!((*coder).header_pos),
        core::mem::size_of::<[u8; 12]>(),
        out,
        out_pos,
        out_size,
    );
    if (*coder).header_pos < core::mem::size_of::<[u8; 12]>() {
        LZMA_OK
    } else {
        LZMA_STREAM_END
    }
}
unsafe fn stream_encoder_mt_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    threads_end(coder, allocator);
    lzma_outq_end(::core::ptr::addr_of_mut!((*coder).outq), allocator);
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
        allocator,
    );
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*coder).filters_cache) as *mut lzma_filter,
        allocator,
    );
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).index_encoder), allocator);
    lzma_index_end((*coder).index, allocator);
    mythread_cond_destroy(::core::ptr::addr_of_mut!((*coder).cond));
    mythread_mutex_destroy(::core::ptr::addr_of_mut!((*coder).mutex));
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn stream_encoder_mt_update(
    coder_ptr: *mut c_void,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter,
    _reversed_filters: *const lzma_filter,
) -> lzma_ret {
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    if (*coder).sequence > SEQ_BLOCK {
        return LZMA_PROG_ERROR;
    }
    if !(*coder).thr.is_null() {
        return LZMA_PROG_ERROR;
    }
    if lzma_raw_encoder_memusage(filters) == UINT64_MAX {
        return LZMA_OPTIONS_ERROR;
    }
    let mut temp: [lzma_filter; 5] = [lzma_filter {
        id: 0,
        options: core::ptr::null_mut(),
    }; 5];
    let ret_: lzma_ret = lzma_filters_copy(
        filters,
        ::core::ptr::addr_of_mut!(temp) as *mut lzma_filter,
        allocator,
    );
    if ret_ != LZMA_OK {
        return ret_;
    }
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
        allocator,
    );
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*coder).filters_cache) as *mut lzma_filter,
        allocator,
    );
    core::ptr::copy_nonoverlapping(
        ::core::ptr::addr_of_mut!(temp) as *const u8,
        ::core::ptr::addr_of_mut!((*coder).filters) as *mut u8,
        core::mem::size_of::<[lzma_filter; 5]>(),
    );
    LZMA_OK
}
unsafe fn get_options(
    options: *const lzma_mt,
    opt_easy: *mut lzma_options_easy,
    filters: *mut *const lzma_filter,
    block_size: *mut u64,
    outbuf_size_max: *mut u64,
) -> lzma_ret {
    if options.is_null() {
        return LZMA_PROG_ERROR;
    }
    if (*options).flags != 0 || (*options).threads == 0 || (*options).threads > LZMA_THREADS_MAX {
        return LZMA_OPTIONS_ERROR;
    }
    if !(*options).filters.is_null() {
        *filters = (*options).filters;
    } else {
        if lzma_easy_preset(&mut *opt_easy, (*options).preset) {
            return LZMA_OPTIONS_ERROR;
        }
        *filters = ::core::ptr::addr_of_mut!((*opt_easy).filters) as *mut lzma_filter;
    }
    if (*options).block_size > 0 {
        *block_size = (*options).block_size;
    } else {
        *block_size = lzma_mt_block_size(*filters);
    }
    if *block_size > BLOCK_SIZE_MAX as u64 || *block_size == UINT64_MAX {
        return LZMA_OPTIONS_ERROR;
    }
    *outbuf_size_max = lzma_block_buffer_bound64(*block_size);
    if *outbuf_size_max == 0 {
        return LZMA_MEM_ERROR;
    }
    LZMA_OK
}
unsafe fn get_progress(coder_ptr: *mut c_void, progress_in: &mut u64, progress_out: &mut u64) {
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    let mut mythread_i_1010: c_uint = 0;
    while if mythread_i_1010 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_1010: c_uint = 0;
        while mythread_j_1010 == 0 {
            *progress_in = (*coder).progress_in;
            *progress_out = (*coder).progress_out;
            let mut i: size_t = 0;
            while i < (*coder).threads_initialized as size_t {
                let mut mythread_i_1015: c_uint = 0;
                while if mythread_i_1015 != 0 {
                    mythread_mutex_unlock(::core::ptr::addr_of_mut!(
                        (*(*coder).threads.add(i)).mutex
                    ));
                    0
                } else {
                    mythread_mutex_lock(::core::ptr::addr_of_mut!(
                        (*(*coder).threads.add(i)).mutex
                    ));
                    1
                } != 0
                {
                    let mut mythread_j_1015: c_uint = 0;
                    while mythread_j_1015 == 0 {
                        *progress_in += (*(*coder).threads.add(i)).progress_in;
                        *progress_out += (*(*coder).threads.add(i)).progress_out;
                        mythread_j_1015 = 1;
                    }
                    mythread_i_1015 = 1;
                }
                i += 1;
            }
            mythread_j_1010 = 1;
        }
        mythread_i_1010 = 1;
    }
}
#[inline(never)]
unsafe fn stream_encoder_mt_create_coder(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
) -> *mut lzma_stream_coder {
    let coder = crate::alloc::internal_alloc_object::<lzma_stream_coder>(allocator);
    if coder.is_null() {
        return core::ptr::null_mut();
    }
    (*next).coder = coder as *mut c_void;
    if mythread_mutex_init(::core::ptr::addr_of_mut!((*coder).mutex)) != 0 {
        crate::alloc::internal_free(coder, allocator);
        (*next).coder = core::ptr::null_mut();
        return core::ptr::null_mut();
    }
    if mythread_cond_init(::core::ptr::addr_of_mut!((*coder).cond)) != 0 {
        mythread_mutex_destroy(::core::ptr::addr_of_mut!((*coder).mutex));
        crate::alloc::internal_free(coder, allocator);
        (*next).coder = core::ptr::null_mut();
        return core::ptr::null_mut();
    }
    (*next).code = Some(
        stream_encode_mt
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
    (*next).end = Some(stream_encoder_mt_end as unsafe fn(*mut c_void, *const lzma_allocator));
    (*next).get_progress = Some(get_progress as unsafe fn(*mut c_void, &mut u64, &mut u64) -> ());
    (*next).update = Some(
        stream_encoder_mt_update
            as unsafe fn(
                *mut c_void,
                *const lzma_allocator,
                *const lzma_filter,
                *const lzma_filter,
            ) -> lzma_ret,
    );
    (*coder).filters[0].id = LZMA_VLI_UNKNOWN;
    (*coder).filters_cache[0].id = LZMA_VLI_UNKNOWN;
    (*coder).index_encoder = lzma_next_coder_s {
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
    (*coder).index = core::ptr::null_mut();
    core::ptr::write_bytes(
        ::core::ptr::addr_of_mut!((*coder).outq) as *mut u8,
        0 as u8,
        core::mem::size_of::<lzma_outq>(),
    );
    (*coder).threads = core::ptr::null_mut();
    (*coder).threads_max = 0;
    (*coder).threads_initialized = 0;
    coder
}

#[inline(never)]
unsafe fn stream_encoder_mt_prepare_threads(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
    threads: u32,
) -> lzma_ret {
    if (*coder).threads_max != threads {
        threads_end(coder, allocator);
        (*coder).threads = core::ptr::null_mut();
        (*coder).threads_max = 0;
        (*coder).threads_initialized = 0;
        (*coder).threads_free = core::ptr::null_mut();
        (*coder).threads =
            crate::alloc::internal_alloc_array::<worker_thread>(threads as size_t, allocator);
        if (*coder).threads.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*coder).threads_max = threads;
    } else {
        threads_stop(coder, true);
    }
    LZMA_OK
}

unsafe fn stream_encoder_mt_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    options: *const lzma_mt,
) -> lzma_ret {
    if core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_mt) -> lzma_ret>,
        uintptr_t,
    >(Some(
        stream_encoder_mt_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_mt) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_mt) -> lzma_ret>,
        uintptr_t,
    >(Some(
        stream_encoder_mt_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_mt) -> lzma_ret,
    ));
    let mut easy: lzma_options_easy = lzma_options_easy {
        filters: [lzma_filter {
            id: 0,
            options: core::ptr::null_mut(),
        }; 5],
        opt_lzma: lzma_options_lzma {
            dict_size: 0,
            preset_dict: core::ptr::null(),
            preset_dict_size: 0,
            lc: 0,
            lp: 0,
            pb: 0,
            mode: 0,
            nice_len: 0,
            mf: 0,
            depth: 0,
            ext_flags: 0,
            ext_size_low: 0,
            ext_size_high: 0,
            reserved_int4: 0,
            reserved_int5: 0,
            reserved_int6: 0,
            reserved_int7: 0,
            reserved_int8: 0,
            reserved_enum1: LZMA_RESERVED_ENUM,
            reserved_enum2: LZMA_RESERVED_ENUM,
            reserved_enum3: LZMA_RESERVED_ENUM,
            reserved_enum4: LZMA_RESERVED_ENUM,
            reserved_ptr1: core::ptr::null_mut(),
            reserved_ptr2: core::ptr::null_mut(),
        },
    };
    let mut filters: *const lzma_filter = core::ptr::null();
    let mut block_size: u64 = 0;
    let mut outbuf_size_max: u64 = 0;
    let ret_: lzma_ret = get_options(
        options,
        ::core::ptr::addr_of_mut!(easy),
        ::core::ptr::addr_of_mut!(filters),
        ::core::ptr::addr_of_mut!(block_size),
        ::core::ptr::addr_of_mut!(outbuf_size_max),
    );
    if ret_ != LZMA_OK {
        return ret_;
    }
    #[cfg(not(target_pointer_width = "64"))]
    if block_size > usize::MAX as u64 || outbuf_size_max > usize::MAX as u64 {
        return LZMA_MEM_ERROR;
    }
    if lzma_raw_encoder_memusage(filters) == UINT64_MAX {
        return LZMA_OPTIONS_ERROR;
    }
    if (*options).check as c_uint > LZMA_CHECK_ID_MAX as c_uint {
        return LZMA_PROG_ERROR;
    }
    if lzma_check_is_supported((*options).check) == 0 {
        return LZMA_UNSUPPORTED_CHECK;
    }
    let mut coder: *mut lzma_stream_coder = (*next).coder as *mut lzma_stream_coder;
    if coder.is_null() {
        coder = stream_encoder_mt_create_coder(next, allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
    }
    (*coder).sequence = SEQ_STREAM_HEADER;
    (*coder).block_size = block_size as size_t;
    (*coder).outbuf_alloc_size = outbuf_size_max as size_t;
    (*coder).thread_error = LZMA_OK;
    (*coder).thr = core::ptr::null_mut();
    let ret__0: lzma_ret = stream_encoder_mt_prepare_threads(coder, allocator, (*options).threads);
    if ret__0 != LZMA_OK {
        return ret__0;
    }
    let ret__1: lzma_ret = lzma_outq_init(
        ::core::ptr::addr_of_mut!((*coder).outq),
        allocator,
        (*options).threads,
    );
    if ret__1 != LZMA_OK {
        return ret__1;
    }
    (*coder).timeout = (*options).timeout;
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
        allocator,
    );
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*coder).filters_cache) as *mut lzma_filter,
        allocator,
    );
    let ret__2: lzma_ret = lzma_filters_copy(
        filters,
        ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
        allocator,
    );
    if ret__2 != LZMA_OK {
        return ret__2;
    }
    lzma_index_end((*coder).index, allocator);
    (*coder).index = lzma_index_init(allocator);
    if (*coder).index.is_null() {
        return LZMA_MEM_ERROR;
    }
    (*coder).stream_flags.version = 0;
    (*coder).stream_flags.check = (*options).check;
    let ret__3: lzma_ret = lzma_stream_header_encode(&(*coder).stream_flags, &mut (*coder).header);
    if ret__3 != LZMA_OK {
        return ret__3;
    }
    (*coder).header_pos = 0;
    (*coder).progress_in = 0;
    (*coder).progress_out = LZMA_STREAM_HEADER_SIZE as u64;
    LZMA_OK
}
pub unsafe fn lzma_stream_encoder_mt(strm: *mut lzma_stream, options: *const lzma_mt) -> lzma_ret {
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = stream_encoder_mt_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        options,
    );
    if ret__0 != LZMA_OK {
        lzma_end(strm);
        return ret__0;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FULL_FLUSH as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FULL_BARRIER as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
pub unsafe fn lzma_stream_encoder_mt_memusage(options: *const lzma_mt) -> u64 {
    let mut easy: lzma_options_easy = lzma_options_easy {
        filters: [lzma_filter {
            id: 0,
            options: core::ptr::null_mut(),
        }; 5],
        opt_lzma: lzma_options_lzma {
            dict_size: 0,
            preset_dict: core::ptr::null(),
            preset_dict_size: 0,
            lc: 0,
            lp: 0,
            pb: 0,
            mode: 0,
            nice_len: 0,
            mf: 0,
            depth: 0,
            ext_flags: 0,
            ext_size_low: 0,
            ext_size_high: 0,
            reserved_int4: 0,
            reserved_int5: 0,
            reserved_int6: 0,
            reserved_int7: 0,
            reserved_int8: 0,
            reserved_enum1: LZMA_RESERVED_ENUM,
            reserved_enum2: LZMA_RESERVED_ENUM,
            reserved_enum3: LZMA_RESERVED_ENUM,
            reserved_enum4: LZMA_RESERVED_ENUM,
            reserved_ptr1: core::ptr::null_mut(),
            reserved_ptr2: core::ptr::null_mut(),
        },
    };
    let mut filters: *const lzma_filter = core::ptr::null();
    let mut block_size: u64 = 0;
    let mut outbuf_size_max: u64 = 0;
    if get_options(
        options,
        ::core::ptr::addr_of_mut!(easy),
        ::core::ptr::addr_of_mut!(filters),
        ::core::ptr::addr_of_mut!(block_size),
        ::core::ptr::addr_of_mut!(outbuf_size_max),
    ) != LZMA_OK
    {
        return UINT64_MAX;
    }
    let inbuf_memusage: u64 = ((*options).threads as u64).wrapping_mul(block_size);
    let mut filters_memusage: u64 = lzma_raw_encoder_memusage(filters);
    if filters_memusage == UINT64_MAX {
        return UINT64_MAX;
    }
    filters_memusage = filters_memusage.wrapping_mul((*options).threads as u64);
    let outq_memusage: u64 = lzma_outq_memusage(outbuf_size_max, (*options).threads) as u64;
    if outq_memusage == UINT64_MAX {
        return UINT64_MAX;
    }
    let mut total_memusage: u64 = (LZMA_MEMUSAGE_BASE)
        .wrapping_add(core::mem::size_of::<lzma_stream_coder>() as u64)
        .wrapping_add(
            ((*options).threads as usize).wrapping_mul(core::mem::size_of::<worker_thread>())
                as u64,
        );
    if (UINT64_MAX).wrapping_sub(total_memusage) < inbuf_memusage {
        return UINT64_MAX;
    }
    total_memusage = total_memusage.wrapping_add(inbuf_memusage);
    if (UINT64_MAX).wrapping_sub(total_memusage) < filters_memusage {
        return UINT64_MAX;
    }
    total_memusage = total_memusage.wrapping_add(filters_memusage);
    if (UINT64_MAX).wrapping_sub(total_memusage) < outq_memusage {
        return UINT64_MAX;
    }
    total_memusage.wrapping_add(outq_memusage)
}
