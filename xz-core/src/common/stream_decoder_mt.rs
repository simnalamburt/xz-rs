use crate::common::outqueue::{
    lzma_outq_clear_cache, lzma_outq_clear_cache2, lzma_outq_enable_partial_output,
};
use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
struct lzma_stream_coder {
    pub sequence: stream_decoder_mt_seq,
    pub block_decoder: lzma_next_coder,
    pub block_options: lzma_block,
    pub filters: [lzma_filter; 5],
    pub stream_flags: lzma_stream_flags,
    pub index_hash: *mut lzma_index_hash,
    pub timeout: u32,
    pub thread_error: lzma_ret,
    pub pending_error: lzma_ret,
    pub threads_max: u32,
    pub threads_initialized: u32,
    pub threads: *mut worker_thread,
    pub threads_free: *mut worker_thread,
    pub thr: *mut worker_thread,
    pub outq: lzma_outq,
    pub mutex: mythread_mutex,
    pub cond: mythread_cond,
    pub memlimit_threading: u64,
    pub memlimit_stop: u64,
    pub mem_direct_mode: u64,
    pub mem_in_use: u64,
    pub mem_cached: u64,
    pub mem_next_filters: u64,
    pub mem_next_in: u64,
    pub mem_next_block: u64,
    pub progress_in: u64,
    pub progress_out: u64,
    pub tell_no_check: bool,
    pub tell_unsupported_check: bool,
    pub tell_any_check: bool,
    pub ignore_check: bool,
    pub concatenated: bool,
    pub fail_fast: bool,
    pub first_stream: bool,
    pub out_was_filled: bool,
    pub pos: size_t,
    pub buffer: [u8; LZMA_BLOCK_HEADER_SIZE_MAX as usize],
}
#[derive(Copy, Clone)]
#[repr(C)]
struct worker_thread {
    pub state: worker_state,
    pub in_0: *mut u8,
    pub in_size: size_t,
    pub in_filled: size_t,
    pub in_pos: size_t,
    pub out_pos: size_t,
    pub coder: *mut lzma_stream_coder,
    #[cfg(feature = "custom_allocator")]
    pub allocator: *const lzma_allocator,
    pub outbuf: *mut lzma_outbuf,
    pub progress_in: size_t,
    pub progress_out: size_t,
    pub partial_update_enabled: bool,
    pub partial_update_started: bool,
    pub block_decoder: lzma_next_coder,
    pub block_options: lzma_block,
    pub mem_filters: u64,
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
pub const THR_EXIT: worker_state = 2;
pub type stream_decoder_mt_seq = c_uint;
pub const SEQ_ERROR: stream_decoder_mt_seq = 11;
pub const SEQ_STREAM_PADDING: stream_decoder_mt_seq = 10;
pub const SEQ_STREAM_FOOTER: stream_decoder_mt_seq = 9;
pub const SEQ_INDEX_DECODE: stream_decoder_mt_seq = 8;
pub const SEQ_INDEX_WAIT_OUTPUT: stream_decoder_mt_seq = 7;
pub const SEQ_BLOCK_DIRECT_RUN: stream_decoder_mt_seq = 6;
pub const SEQ_BLOCK_DIRECT_INIT: stream_decoder_mt_seq = 5;
pub const SEQ_BLOCK_THR_RUN: stream_decoder_mt_seq = 4;
pub const SEQ_BLOCK_THR_INIT: stream_decoder_mt_seq = 3;
type StreamMtBlockState = u8;
const STREAM_MT_BLOCK_HEADER: StreamMtBlockState = 0;
const STREAM_MT_BLOCK_INIT: StreamMtBlockState = 1;
const STREAM_MT_BLOCK_THR_INIT: StreamMtBlockState = 2;
const STREAM_MT_BLOCK_THR_RUN: StreamMtBlockState = 3;
const STREAM_MT_BLOCK_DIRECT_RUN: StreamMtBlockState = 4;
const STREAM_MT_INDEX_DECODE: StreamMtBlockState = 5;
const STREAM_MT_STREAM_FOOTER: StreamMtBlockState = 6;
const STREAM_MT_STREAM_PADDING: StreamMtBlockState = 7;
const STREAM_MT_RESTART_LOOP: StreamMtBlockState = 8;
pub const SEQ_BLOCK_INIT: stream_decoder_mt_seq = 2;
pub const SEQ_BLOCK_HEADER: stream_decoder_mt_seq = 1;
pub const SEQ_STREAM_HEADER: stream_decoder_mt_seq = 0;
unsafe fn worker_enable_partial_update(thr_ptr: *mut c_void) {
    let thr: *mut worker_thread = thr_ptr as *mut worker_thread;
    let mut mythread_i_325: c_uint = 0;
    while if mythread_i_325 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*thr).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*thr).mutex));
        1
    } != 0
    {
        let mut mythread_j_325: c_uint = 0;
        while mythread_j_325 == 0 {
            (*thr).partial_update_enabled = true;
            mythread_cond_signal(::core::ptr::addr_of_mut!((*thr).cond));
            mythread_j_325 = 1;
        }
        mythread_i_325 = 1;
    }
}
unsafe extern "C" fn worker_decoder(thr_ptr: *mut c_void) -> *mut c_void {
    let thr: *mut worker_thread = thr_ptr as *mut worker_thread;
    let mut in_filled: size_t = 0;
    let mut partial_update_enabled: bool = false;
    let mut ret: lzma_ret = LZMA_OK;
    loop {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*thr).mutex));
        loop {
            if (*thr).state == THR_IDLE {
                mythread_cond_wait(
                    ::core::ptr::addr_of_mut!((*thr).cond),
                    ::core::ptr::addr_of_mut!((*thr).mutex),
                );
            } else {
                if (*thr).state == THR_EXIT {
                    mythread_mutex_unlock(::core::ptr::addr_of_mut!((*thr).mutex));
                    crate::alloc::internal_free_array(
                        (*thr).in_0,
                        (*thr).in_size,
                        worker_allocator(thr),
                    );
                    lzma_next_end(
                        ::core::ptr::addr_of_mut!((*thr).block_decoder),
                        worker_allocator(thr),
                    );
                    mythread_mutex_destroy(::core::ptr::addr_of_mut!((*thr).mutex));
                    mythread_cond_destroy(::core::ptr::addr_of_mut!((*thr).cond));
                    return MYTHREAD_RET_VALUE;
                }
                (*thr).progress_in = (*thr).in_pos;
                (*thr).progress_out = (*thr).out_pos;
                in_filled = (*thr).in_filled;
                partial_update_enabled = (*thr).partial_update_enabled;
                if in_filled != (*thr).in_pos
                    || (partial_update_enabled && !(*thr).partial_update_started)
                {
                    break;
                }
                mythread_cond_wait(
                    ::core::ptr::addr_of_mut!((*thr).cond),
                    ::core::ptr::addr_of_mut!((*thr).mutex),
                );
            }
        }
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*thr).mutex));
        let chunk_size: size_t = 16384;
        if in_filled - (*thr).in_pos > chunk_size {
            in_filled = (*thr).in_pos + chunk_size;
        }
        ret = match (*thr).block_decoder.code {
            Some(code) => code(
                (*thr).block_decoder.coder,
                worker_allocator(thr),
                (*thr).in_0,
                ::core::ptr::addr_of_mut!((*thr).in_pos),
                in_filled,
                ::core::ptr::addr_of_mut!((*(*thr).outbuf).buf) as *mut u8,
                ::core::ptr::addr_of_mut!((*thr).out_pos),
                (*(*thr).outbuf).allocated,
                LZMA_RUN,
            ),
            None => LZMA_PROG_ERROR,
        };
        if ret == LZMA_OK {
            if partial_update_enabled {
                (*thr).partial_update_started = true;
                let mut mythread_i_415: c_uint = 0;
                while if mythread_i_415 != 0 {
                    mythread_mutex_unlock(::core::ptr::addr_of_mut!((*(*thr).coder).mutex));
                    0
                } else {
                    mythread_mutex_lock(::core::ptr::addr_of_mut!((*(*thr).coder).mutex));
                    1
                } != 0
                {
                    let mut mythread_j_415: c_uint = 0;
                    while mythread_j_415 == 0 {
                        (*(*thr).outbuf).pos = (*thr).out_pos;
                        (*(*thr).outbuf).decoder_in_pos = (*thr).in_pos;
                        mythread_cond_signal(::core::ptr::addr_of_mut!((*(*thr).coder).cond));
                        mythread_j_415 = 1;
                    }
                    mythread_i_415 = 1;
                }
            }
        } else {
            let mut mythread_i_434: c_uint = 0;
            while if mythread_i_434 != 0 {
                mythread_mutex_unlock(::core::ptr::addr_of_mut!((*thr).mutex));
                0
            } else {
                mythread_mutex_lock(::core::ptr::addr_of_mut!((*thr).mutex));
                1
            } != 0
            {
                let mut mythread_j_434: c_uint = 0;
                while mythread_j_434 == 0 {
                    if ret == LZMA_STREAM_END && (*thr).in_filled != (*thr).in_size {
                        ret = LZMA_PROG_ERROR;
                    }
                    if (*thr).state != THR_EXIT {
                        (*thr).state = THR_IDLE;
                    }
                    mythread_j_434 = 1;
                }
                mythread_i_434 = 1;
            }
            if ret == LZMA_STREAM_END {
                crate::alloc::internal_free_array(
                    (*thr).in_0,
                    (*thr).in_size,
                    worker_allocator(thr),
                );
                (*thr).in_0 = core::ptr::null_mut();
            }
            let mut mythread_i_458: c_uint = 0;
            while if mythread_i_458 != 0 {
                mythread_mutex_unlock(::core::ptr::addr_of_mut!((*(*thr).coder).mutex));
                0
            } else {
                mythread_mutex_lock(::core::ptr::addr_of_mut!((*(*thr).coder).mutex));
                1
            } != 0
            {
                let mut mythread_j_458: c_uint = 0;
                while mythread_j_458 == 0 {
                    (*(*thr).coder).progress_in += (*thr).in_pos as u64;
                    (*(*thr).coder).progress_out += (*thr).out_pos as u64;
                    (*thr).progress_in = 0;
                    (*thr).progress_out = 0;
                    (*(*thr).outbuf).pos = (*thr).out_pos;
                    (*(*thr).outbuf).decoder_in_pos = (*thr).in_pos;
                    (*(*thr).outbuf).finished = true;
                    (*(*thr).outbuf).finish_ret = ret;
                    (*thr).outbuf = core::ptr::null_mut();
                    if ret != LZMA_STREAM_END && (*(*thr).coder).thread_error == LZMA_OK {
                        (*(*thr).coder).thread_error = ret;
                    }
                    if ret == LZMA_STREAM_END {
                        (*(*thr).coder).mem_in_use -= (*thr).in_size as u64;
                        (*(*thr).coder).mem_in_use -= (*thr).mem_filters;
                        (*(*thr).coder).mem_cached += (*thr).mem_filters;
                        (*thr).next = (*(*thr).coder).threads_free;
                        (*(*thr).coder).threads_free = thr;
                    }
                    mythread_cond_signal(::core::ptr::addr_of_mut!((*(*thr).coder).cond));
                    mythread_j_458 = 1;
                }
                mythread_i_458 = 1;
            }
        }
    }
}
unsafe fn threads_end(coder: *mut lzma_stream_coder, allocator: *const lzma_allocator) {
    let mut i: u32 = 0;
    while i < (*coder).threads_initialized {
        let mut mythread_i_502: c_uint = 0;
        while if mythread_i_502 != 0 {
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
            let mut mythread_j_502: c_uint = 0;
            while mythread_j_502 == 0 {
                (*(*coder).threads.offset(i as isize)).state = THR_EXIT;
                mythread_cond_signal(::core::ptr::addr_of_mut!(
                    (*(*coder).threads.offset(i as isize)).cond
                ));
                mythread_j_502 = 1;
            }
            mythread_i_502 = 1;
        }
        i += 1;
    }
    let mut i_0: u32 = 0;
    while i_0 < (*coder).threads_initialized {
        mythread_join((*(*coder).threads.offset(i_0 as isize)).thread_id);
        i_0 += 1;
    }
    crate::alloc::internal_free_array((*coder).threads, (*coder).threads_max as size_t, allocator);
    (*coder).threads_initialized = 0;
    (*coder).threads = core::ptr::null_mut();
    (*coder).threads_free = core::ptr::null_mut();
    (*coder).mem_in_use = 0;
    (*coder).mem_cached = 0;
}
unsafe fn threads_stop(coder: *mut lzma_stream_coder) {
    let mut i: u32 = 0;
    while i < (*coder).threads_initialized {
        let mut mythread_i_538: c_uint = 0;
        while if mythread_i_538 != 0 {
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
            let mut mythread_j_538: c_uint = 0;
            while mythread_j_538 == 0 {
                (*(*coder).threads.offset(i as isize)).state = THR_IDLE;
                mythread_j_538 = 1;
            }
            mythread_i_538 = 1;
        }
        i += 1;
    }
}
unsafe fn initialize_new_thread(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    if (*coder).threads.is_null() {
        (*coder).threads = crate::alloc::internal_alloc_array::<worker_thread>(
            (*coder).threads_max as size_t,
            allocator,
        );
        if (*coder).threads.is_null() {
            return LZMA_MEM_ERROR;
        }
    }
    let thr: *mut worker_thread = (*coder)
        .threads
        .offset((*coder).threads_initialized as isize)
        as *mut worker_thread;
    if mythread_mutex_init(::core::ptr::addr_of_mut!((*thr).mutex)) == 0 {
        if mythread_cond_init(::core::ptr::addr_of_mut!((*thr).cond)) == 0 {
            (*thr).state = THR_IDLE;
            (*thr).in_0 = core::ptr::null_mut();
            (*thr).in_size = 0;
            set_worker_allocator(thr, allocator);
            (*thr).coder = coder as *mut lzma_stream_coder;
            (*thr).outbuf = core::ptr::null_mut();
            (*thr).block_decoder = lzma_next_coder_s {
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
            (*thr).mem_filters = 0;
            if mythread_create(
                ::core::ptr::addr_of_mut!((*thr).thread_id),
                worker_decoder as unsafe extern "C" fn(*mut c_void) -> *mut c_void,
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
    LZMA_MEM_ERROR
}
unsafe fn get_thread(coder: *mut lzma_stream_coder, allocator: *const lzma_allocator) -> lzma_ret {
    let mut mythread_i_608: c_uint = 0;
    while if mythread_i_608 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_608: c_uint = 0;
        while mythread_j_608 == 0 {
            if !(*coder).threads_free.is_null() {
                (*coder).thr = (*coder).threads_free;
                (*coder).threads_free = (*(*coder).threads_free).next;
                (*coder).mem_cached -= (*(*coder).thr).mem_filters;
            }
            mythread_j_608 = 1;
        }
        mythread_i_608 = 1;
    }
    if (*coder).thr.is_null() {
        let ret_: lzma_ret = initialize_new_thread(coder, allocator);
        if ret_ != LZMA_OK {
            return ret_;
        }
    }
    (*(*coder).thr).in_filled = 0;
    (*(*coder).thr).in_pos = 0;
    (*(*coder).thr).out_pos = 0;
    (*(*coder).thr).progress_in = 0;
    (*(*coder).thr).progress_out = 0;
    (*(*coder).thr).partial_update_enabled = false;
    (*(*coder).thr).partial_update_started = false;
    LZMA_OK
}
unsafe fn read_output_and_wait(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    input_is_possible: *mut bool,
    waiting_allowed: bool,
    wait_abs: *mut mythread_condtime,
    has_blocked: *mut bool,
) -> lzma_ret {
    let mut ret: lzma_ret = LZMA_OK;
    let mut mythread_i_654: c_uint = 0;
    while if mythread_i_654 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_654: c_uint = 0;
        while mythread_j_654 == 0 {
            loop {
                let out_start: size_t = *out_pos;
                loop {
                    ret = lzma_outq_read(
                        ::core::ptr::addr_of_mut!((*coder).outq),
                        allocator,
                        out,
                        out_pos,
                        out_size,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    if ret == LZMA_STREAM_END {
                        lzma_outq_enable_partial_output(
                            ::core::ptr::addr_of_mut!((*coder).outq),
                            worker_enable_partial_update as unsafe fn(*mut c_void) -> (),
                        );
                    }
                    if ret != LZMA_STREAM_END {
                        break;
                    }
                }
                if ret != LZMA_OK {
                    break;
                }
                if *out_pos == out_size && *out_pos != out_start {
                    (*coder).out_was_filled = true;
                }
                if (*coder).thread_error != LZMA_OK {
                    if (*coder).fail_fast {
                        ret = (*coder).thread_error;
                        break;
                    } else {
                        (*coder).pending_error = LZMA_PROG_ERROR;
                    }
                }
                if !input_is_possible.is_null()
                    && (*coder).memlimit_threading - (*coder).mem_in_use - (*coder).outq.mem_in_use
                        >= (*coder).mem_next_block
                    && lzma_outq_has_buf(&(*coder).outq)
                    && ((*coder).threads_initialized < (*coder).threads_max
                        || !(*coder).threads_free.is_null())
                {
                    *input_is_possible = true;
                    break;
                } else {
                    if !waiting_allowed {
                        break;
                    }
                    if lzma_outq_is_empty(&(*coder).outq) {
                        break;
                    }
                    if lzma_outq_is_readable(::core::ptr::addr_of_mut!((*coder).outq)) {
                        break;
                    }
                    if !(*coder).thr.is_null() && (*(*coder).thr).partial_update_enabled {
                        if (*(*(*coder).thr).outbuf).decoder_in_pos == (*(*coder).thr).in_filled {
                            break;
                        }
                    }
                    if (*coder).timeout != 0 {
                        if !*has_blocked {
                            *has_blocked = true;
                            mythread_condtime_set(
                                wait_abs,
                                ::core::ptr::addr_of_mut!((*coder).cond),
                                (*coder).timeout,
                            );
                        }
                        if mythread_cond_timedwait(
                            ::core::ptr::addr_of_mut!((*coder).cond),
                            ::core::ptr::addr_of_mut!((*coder).mutex),
                            wait_abs,
                        ) != 0
                        {
                            ret = LZMA_RET_INTERNAL1;
                            break;
                        }
                    } else {
                        mythread_cond_wait(
                            ::core::ptr::addr_of_mut!((*coder).cond),
                            ::core::ptr::addr_of_mut!((*coder).mutex),
                        );
                    }
                    if ret != LZMA_OK {
                        break;
                    }
                }
            }
            mythread_j_654 = 1;
        }
        mythread_i_654 = 1;
    }
    if ret != LZMA_OK && ret != LZMA_RET_INTERNAL1 {
        threads_stop(coder);
    }
    ret
}
unsafe fn decode_block_header(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
    in_0: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
) -> lzma_ret {
    if *in_pos >= in_size {
        return LZMA_OK;
    }
    if (*coder).pos == 0 {
        if *in_0.add(*in_pos) == INDEX_INDICATOR {
            return LZMA_RET_INTERNAL2;
        }
        (*coder).block_options.header_size = ((*in_0.add(*in_pos) as u32) + 1) * 4;
    }
    lzma_bufcpy(
        in_0,
        in_pos,
        in_size,
        ::core::ptr::addr_of_mut!((*coder).buffer) as *mut u8,
        ::core::ptr::addr_of_mut!((*coder).pos),
        (*coder).block_options.header_size as size_t,
    );
    if (*coder).pos < (*coder).block_options.header_size as size_t {
        return LZMA_OK;
    }
    (*coder).pos = 0;
    (*coder).block_options.version = 1;
    (*coder).block_options.filters =
        ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter;
    let ret_: lzma_ret =
        lzma_block_header_decode(&mut (*coder).block_options, allocator, &(*coder).buffer);
    if ret_ != LZMA_OK {
        return ret_;
    }
    (*coder).block_options.ignore_check = (*coder).ignore_check as lzma_bool;
    LZMA_STREAM_END
}
fn comp_blk_size(coder: *const lzma_stream_coder) -> size_t {
    unsafe {
        (vli_ceil4((*coder).block_options.compressed_size)
            + lzma_check_size((*coder).stream_flags.check) as lzma_vli) as size_t
    }
}
fn is_direct_mode_needed(size: lzma_vli) -> bool {
    size == LZMA_VLI_UNKNOWN || size > SIZE_MAX.wrapping_div(3 as size_t) as lzma_vli
}
unsafe fn stream_decoder_reset(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    (*coder).index_hash = lzma_index_hash_init((*coder).index_hash, allocator);
    if (*coder).index_hash.is_null() {
        return LZMA_MEM_ERROR;
    }
    (*coder).sequence = SEQ_STREAM_HEADER;
    (*coder).pos = 0;
    LZMA_OK
}

#[inline(never)]
unsafe fn stream_decode_mt_block_init(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    wait_abs: *mut mythread_condtime,
    has_blocked: *mut bool,
) -> Option<lzma_ret> {
    if (*coder).mem_next_filters > (*coder).memlimit_stop {
        let ret: lzma_ret = read_output_and_wait(
            coder,
            allocator,
            out,
            out_pos,
            out_size,
            core::ptr::null_mut(),
            true,
            wait_abs,
            has_blocked,
        );
        if ret != LZMA_OK {
            return Some(ret);
        }
        if !lzma_outq_is_empty(&(*coder).outq) {
            return Some(LZMA_OK);
        }
        return Some(LZMA_MEMLIMIT_ERROR);
    }
    if is_direct_mode_needed((*coder).block_options.compressed_size)
        || is_direct_mode_needed((*coder).block_options.uncompressed_size)
    {
        (*coder).sequence = SEQ_BLOCK_DIRECT_INIT;
        return None;
    }
    (*coder).mem_next_in = comp_blk_size(coder) as u64;
    let mem_buffers: u64 =
        (*coder).mem_next_in + lzma_outq_outbuf_memusage((*coder).block_options.uncompressed_size);
    if (UINT64_MAX).wrapping_sub(mem_buffers) < (*coder).mem_next_filters {
        (*coder).sequence = SEQ_BLOCK_DIRECT_INIT;
        return None;
    }
    (*coder).mem_next_block = (*coder).mem_next_filters + mem_buffers;
    if (*coder).mem_next_block > (*coder).memlimit_threading {
        (*coder).sequence = SEQ_BLOCK_DIRECT_INIT;
        return None;
    }
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).block_decoder), allocator);
    (*coder).mem_direct_mode = 0;
    let ret: lzma_ret = lzma_index_hash_append(
        (*coder).index_hash,
        lzma_block_unpadded_size(::core::ptr::addr_of_mut!((*coder).block_options)),
        (*coder).block_options.uncompressed_size,
    );
    if ret != LZMA_OK {
        (*coder).pending_error = ret;
        (*coder).sequence = SEQ_ERROR;
        return None;
    }
    (*coder).sequence = SEQ_BLOCK_THR_INIT;
    None
}

#[inline(never)]
unsafe fn stream_decode_mt_thread_init(
    coder: *mut lzma_stream_coder,
    allocator: *const lzma_allocator,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    wait_abs: *mut mythread_condtime,
    has_blocked: *mut bool,
) -> Option<lzma_ret> {
    let mut block_can_start: bool = false;
    let ret: lzma_ret = read_output_and_wait(
        coder,
        allocator,
        out,
        out_pos,
        out_size,
        ::core::ptr::addr_of_mut!(block_can_start),
        true,
        wait_abs,
        has_blocked,
    );
    if ret != LZMA_OK {
        return Some(ret);
    }
    if (*coder).pending_error != LZMA_OK {
        (*coder).sequence = SEQ_ERROR;
        return None;
    }
    if !block_can_start {
        return Some(LZMA_OK);
    }
    let mut mem_in_use: u64 = 0;
    let mut mem_cached: u64 = 0;
    let mut thr: *mut worker_thread = core::ptr::null_mut();
    let mut mythread_i_1347: c_uint = 0;
    while if mythread_i_1347 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_1347: c_uint = 0;
        while mythread_j_1347 == 0 {
            mem_in_use = (*coder).mem_in_use;
            mem_cached = (*coder).mem_cached;
            thr = (*coder).threads_free;
            mythread_j_1347 = 1;
        }
        mythread_i_1347 = 1;
    }
    let mem_max: u64 = (*coder).memlimit_threading - (*coder).mem_next_block;
    if mem_in_use + mem_cached + (*coder).outq.mem_allocated > mem_max {
        lzma_outq_clear_cache2(
            ::core::ptr::addr_of_mut!((*coder).outq),
            allocator,
            (*coder).block_options.uncompressed_size as size_t,
        );
    }
    let mut mem_freed: u64 = 0;
    if !thr.is_null() && mem_in_use + mem_cached + (*coder).outq.mem_in_use > mem_max {
        if (*thr).mem_filters <= (*coder).mem_next_filters {
            thr = (*thr).next;
        }
        while !thr.is_null() {
            lzma_next_end(::core::ptr::addr_of_mut!((*thr).block_decoder), allocator);
            mem_freed += (*thr).mem_filters;
            (*thr).mem_filters = 0;
            thr = (*thr).next;
        }
    }
    let mut mythread_i_1410: c_uint = 0;
    while if mythread_i_1410 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_1410: c_uint = 0;
        while mythread_j_1410 == 0 {
            (*coder).mem_cached -= mem_freed;
            (*coder).mem_in_use += (*coder).mem_next_in + (*coder).mem_next_filters;
            mythread_j_1410 = 1;
        }
        mythread_i_1410 = 1;
    }
    let mut ret: lzma_ret = lzma_outq_prealloc_buf(
        ::core::ptr::addr_of_mut!((*coder).outq),
        allocator,
        (*coder).block_options.uncompressed_size as size_t,
    );
    if ret != LZMA_OK {
        threads_stop(coder);
        return Some(ret);
    }
    ret = get_thread(coder, allocator);
    if ret != LZMA_OK {
        threads_stop(coder);
        return Some(ret);
    }
    (*(*coder).thr).mem_filters = (*coder).mem_next_filters;
    (*(*coder).thr).block_options = (*coder).block_options;
    ret = lzma_block_decoder_init(
        ::core::ptr::addr_of_mut!((*(*coder).thr).block_decoder),
        allocator,
        ::core::ptr::addr_of_mut!((*(*coder).thr).block_options),
    );
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
        allocator,
    );
    (*(*coder).thr).block_options.filters = core::ptr::null_mut();
    if ret != LZMA_OK {
        (*coder).pending_error = ret;
        (*coder).sequence = SEQ_ERROR;
        return None;
    }
    (*(*coder).thr).in_size = (*coder).mem_next_in as size_t;
    (*(*coder).thr).in_0 =
        crate::alloc::internal_alloc_array::<u8>((*(*coder).thr).in_size, allocator);
    if (*(*coder).thr).in_0.is_null() {
        threads_stop(coder);
        return Some(LZMA_MEM_ERROR);
    }
    (*(*coder).thr).outbuf = lzma_outq_get_buf(
        ::core::ptr::addr_of_mut!((*coder).outq),
        (*coder).thr as *mut c_void,
    );
    let mut mythread_i_1478: c_uint = 0;
    while if mythread_i_1478 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*(*coder).thr).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*(*coder).thr).mutex));
        1
    } != 0
    {
        let mut mythread_j_1478: c_uint = 0;
        while mythread_j_1478 == 0 {
            (*(*coder).thr).state = THR_RUN;
            mythread_cond_signal(::core::ptr::addr_of_mut!((*(*coder).thr).cond));
            mythread_j_1478 = 1;
        }
        mythread_i_1478 = 1;
    }
    let mut mythread_i_1486: c_uint = 0;
    while if mythread_i_1486 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_1486: c_uint = 0;
        while mythread_j_1486 == 0 {
            lzma_outq_enable_partial_output(
                ::core::ptr::addr_of_mut!((*coder).outq),
                worker_enable_partial_update as unsafe fn(*mut c_void) -> (),
            );
            mythread_j_1486 = 1;
        }
        mythread_i_1486 = 1;
    }
    (*coder).sequence = SEQ_BLOCK_THR_RUN;
    None
}

unsafe fn stream_decode_mt(
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
    let mut wait_abs: mythread_condtime = unsafe { core::mem::zeroed() };
    let mut has_blocked: bool = false;
    let waiting_allowed: bool =
        action == LZMA_FINISH || *in_pos == in_size && !(*coder).out_was_filled;
    (*coder).out_was_filled = false;
    loop {
        let mut block_state: StreamMtBlockState;
        match (*coder).sequence {
            0 => {
                let in_old: size_t = *in_pos;
                lzma_bufcpy(
                    in_0,
                    in_pos,
                    in_size,
                    ::core::ptr::addr_of_mut!((*coder).buffer) as *mut u8,
                    ::core::ptr::addr_of_mut!((*coder).pos),
                    LZMA_STREAM_HEADER_SIZE as size_t,
                );
                (*coder).progress_in += (*in_pos - in_old) as u64;
                if (*coder).pos < LZMA_STREAM_HEADER_SIZE as size_t {
                    return LZMA_OK;
                }
                (*coder).pos = 0;
                let ret: lzma_ret = lzma_stream_header_decode(
                    &mut (*coder).stream_flags,
                    (*coder).buffer.subarray::<0, STREAM_HEADER_SIZE>(),
                );
                if ret != LZMA_OK {
                    return if ret == LZMA_FORMAT_ERROR && !(*coder).first_stream {
                        LZMA_DATA_ERROR
                    } else {
                        ret
                    };
                }
                (*coder).first_stream = false;
                (*coder).block_options.check = (*coder).stream_flags.check;
                (*coder).sequence = SEQ_BLOCK_HEADER;
                if (*coder).tell_no_check && (*coder).stream_flags.check == LZMA_CHECK_NONE {
                    return LZMA_NO_CHECK;
                }
                if (*coder).tell_unsupported_check
                    && lzma_check_is_supported((*coder).stream_flags.check) == 0
                {
                    return LZMA_UNSUPPORTED_CHECK;
                }
                if (*coder).tell_any_check {
                    return LZMA_GET_CHECK;
                }
                block_state = STREAM_MT_BLOCK_HEADER;
            }
            1 => {
                block_state = STREAM_MT_BLOCK_HEADER;
            }
            2 => {
                block_state = STREAM_MT_BLOCK_INIT;
            }
            3 => {
                block_state = STREAM_MT_BLOCK_THR_INIT;
            }
            4 => {
                block_state = STREAM_MT_BLOCK_THR_RUN;
            }
            5 => {
                let ret__3: lzma_ret = read_output_and_wait(
                    coder,
                    allocator,
                    out,
                    out_pos,
                    out_size,
                    core::ptr::null_mut(),
                    true,
                    ::core::ptr::addr_of_mut!(wait_abs),
                    ::core::ptr::addr_of_mut!(has_blocked),
                );
                if ret__3 != LZMA_OK {
                    return ret__3;
                }
                if !lzma_outq_is_empty(&(*coder).outq) {
                    return LZMA_OK;
                }
                lzma_outq_clear_cache(::core::ptr::addr_of_mut!((*coder).outq), allocator);
                threads_end(coder, allocator);
                let ret_3: lzma_ret = lzma_block_decoder_init(
                    ::core::ptr::addr_of_mut!((*coder).block_decoder),
                    allocator,
                    ::core::ptr::addr_of_mut!((*coder).block_options),
                );
                lzma_filters_free(
                    ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
                    allocator,
                );
                (*coder).block_options.filters = core::ptr::null_mut();
                if ret_3 != LZMA_OK {
                    return ret_3;
                }
                (*coder).mem_direct_mode = (*coder).mem_next_filters;
                (*coder).sequence = SEQ_BLOCK_DIRECT_RUN;
                block_state = STREAM_MT_BLOCK_DIRECT_RUN;
            }
            6 => {
                block_state = STREAM_MT_BLOCK_DIRECT_RUN;
            }
            7 => {
                let ret__5: lzma_ret = read_output_and_wait(
                    coder,
                    allocator,
                    out,
                    out_pos,
                    out_size,
                    core::ptr::null_mut(),
                    true,
                    ::core::ptr::addr_of_mut!(wait_abs),
                    ::core::ptr::addr_of_mut!(has_blocked),
                );
                if ret__5 != LZMA_OK {
                    return ret__5;
                }
                if !lzma_outq_is_empty(&(*coder).outq) {
                    return LZMA_OK;
                }
                (*coder).sequence = SEQ_INDEX_DECODE;
                block_state = STREAM_MT_INDEX_DECODE;
            }
            8 => {
                block_state = STREAM_MT_INDEX_DECODE;
            }
            9 => {
                block_state = STREAM_MT_STREAM_FOOTER;
            }
            10 => {
                block_state = STREAM_MT_STREAM_PADDING;
            }
            11 => {
                if !(*coder).fail_fast {
                    let ret__8: lzma_ret = read_output_and_wait(
                        coder,
                        allocator,
                        out,
                        out_pos,
                        out_size,
                        core::ptr::null_mut(),
                        true,
                        ::core::ptr::addr_of_mut!(wait_abs),
                        ::core::ptr::addr_of_mut!(has_blocked),
                    );
                    if ret__8 != LZMA_OK {
                        return ret__8;
                    }
                    if !lzma_outq_is_empty(&(*coder).outq) {
                        return LZMA_OK;
                    }
                }
                return (*coder).pending_error;
            }
            _ => return LZMA_PROG_ERROR,
        }
        match block_state {
            STREAM_MT_BLOCK_DIRECT_RUN => {
                let in_old_1: size_t = *in_pos;
                let out_old: size_t = *out_pos;
                let Some(code) = (*coder).block_decoder.code else {
                    return LZMA_PROG_ERROR;
                };
                let ret_4: lzma_ret = code(
                    (*coder).block_decoder.coder,
                    allocator,
                    in_0,
                    in_pos,
                    in_size,
                    out,
                    out_pos,
                    out_size,
                    action,
                );
                (*coder).progress_in += (*in_pos - in_old_1) as u64;
                (*coder).progress_out += (*out_pos - out_old) as u64;
                if ret_4 != LZMA_STREAM_END {
                    return ret_4;
                }
                let ret__4: lzma_ret = lzma_index_hash_append(
                    (*coder).index_hash,
                    lzma_block_unpadded_size(::core::ptr::addr_of_mut!((*coder).block_options)),
                    (*coder).block_options.uncompressed_size,
                );
                if ret__4 != LZMA_OK {
                    return ret__4;
                }
                (*coder).sequence = SEQ_BLOCK_HEADER;
                block_state = STREAM_MT_RESTART_LOOP;
            }
            STREAM_MT_BLOCK_HEADER => {
                let in_old_0: size_t = *in_pos;
                let ret_0: lzma_ret = decode_block_header(coder, allocator, in_0, in_pos, in_size);
                (*coder).progress_in += (*in_pos - in_old_0) as u64;
                if ret_0 == LZMA_OK {
                    if action == LZMA_FINISH && (*coder).fail_fast {
                        threads_stop(coder);
                        return LZMA_DATA_ERROR;
                    }
                    let ret_: lzma_ret = read_output_and_wait(
                        coder,
                        allocator,
                        out,
                        out_pos,
                        out_size,
                        core::ptr::null_mut(),
                        waiting_allowed,
                        ::core::ptr::addr_of_mut!(wait_abs),
                        ::core::ptr::addr_of_mut!(has_blocked),
                    );
                    if ret_ != LZMA_OK {
                        return ret_;
                    }
                    if (*coder).pending_error != LZMA_OK {
                        (*coder).sequence = SEQ_ERROR;
                    } else {
                        return LZMA_OK;
                    }
                    block_state = STREAM_MT_RESTART_LOOP;
                } else if ret_0 == LZMA_RET_INTERNAL2 {
                    (*coder).sequence = SEQ_INDEX_WAIT_OUTPUT;
                    block_state = STREAM_MT_RESTART_LOOP;
                } else if ret_0 != LZMA_STREAM_END {
                    (*coder).pending_error = ret_0;
                    (*coder).sequence = SEQ_ERROR;
                    block_state = STREAM_MT_RESTART_LOOP;
                } else {
                    (*coder).mem_next_filters = lzma_raw_decoder_memusage(
                        ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
                    );
                    if (*coder).mem_next_filters == UINT64_MAX {
                        (*coder).pending_error = LZMA_OPTIONS_ERROR;
                        (*coder).sequence = SEQ_ERROR;
                        block_state = STREAM_MT_RESTART_LOOP;
                    } else {
                        (*coder).sequence = SEQ_BLOCK_INIT;
                        block_state = STREAM_MT_BLOCK_INIT;
                    }
                }
            }
            STREAM_MT_INDEX_DECODE => {
                if *in_pos >= in_size {
                    return LZMA_OK;
                }
                let in_old_2: size_t = *in_pos;
                let ret_5: lzma_ret =
                    lzma_index_hash_decode((*coder).index_hash, in_0, in_pos, in_size);
                (*coder).progress_in += (*in_pos - in_old_2) as u64;
                if ret_5 != LZMA_STREAM_END {
                    return ret_5;
                }
                (*coder).sequence = SEQ_STREAM_FOOTER;
                block_state = STREAM_MT_STREAM_FOOTER;
            }
            _ => {}
        }
        match block_state {
            STREAM_MT_BLOCK_INIT => {
                if let Some(ret) = stream_decode_mt_block_init(
                    coder,
                    allocator,
                    out,
                    out_pos,
                    out_size,
                    ::core::ptr::addr_of_mut!(wait_abs),
                    ::core::ptr::addr_of_mut!(has_blocked),
                ) {
                    return ret;
                }
                continue;
            }
            STREAM_MT_STREAM_FOOTER => {
                let in_old_3: size_t = *in_pos;
                lzma_bufcpy(
                    in_0,
                    in_pos,
                    in_size,
                    ::core::ptr::addr_of_mut!((*coder).buffer) as *mut u8,
                    ::core::ptr::addr_of_mut!((*coder).pos),
                    LZMA_STREAM_HEADER_SIZE as size_t,
                );
                (*coder).progress_in += (*in_pos - in_old_3) as u64;
                if (*coder).pos < LZMA_STREAM_HEADER_SIZE as size_t {
                    return LZMA_OK;
                }
                (*coder).pos = 0;
                let mut footer_flags: lzma_stream_flags = lzma_stream_flags {
                    version: 0,
                    backward_size: 0,
                    check: LZMA_CHECK_NONE,
                    reserved_enum1: LZMA_RESERVED_ENUM,
                    reserved_enum2: LZMA_RESERVED_ENUM,
                    reserved_enum3: LZMA_RESERVED_ENUM,
                    reserved_enum4: LZMA_RESERVED_ENUM,
                    reserved_bool1: 0,
                    reserved_bool2: 0,
                    reserved_bool3: 0,
                    reserved_bool4: 0,
                    reserved_bool5: 0,
                    reserved_bool6: 0,
                    reserved_bool7: 0,
                    reserved_bool8: 0,
                    reserved_int1: 0,
                    reserved_int2: 0,
                };
                let ret_6: lzma_ret = lzma_stream_footer_decode(
                    &mut footer_flags,
                    (*coder).buffer.subarray::<0, STREAM_HEADER_SIZE>(),
                );
                if ret_6 != LZMA_OK {
                    return if ret_6 == LZMA_FORMAT_ERROR {
                        LZMA_DATA_ERROR
                    } else {
                        ret_6
                    };
                }
                if lzma_index_hash_size(&*(*coder).index_hash) != footer_flags.backward_size {
                    return LZMA_DATA_ERROR;
                }
                let ret__6: lzma_ret =
                    lzma_stream_flags_compare(&(*coder).stream_flags, &footer_flags);
                if ret__6 != LZMA_OK {
                    return ret__6;
                }
                if !(*coder).concatenated {
                    return LZMA_STREAM_END;
                }
                (*coder).sequence = SEQ_STREAM_PADDING;
                block_state = STREAM_MT_STREAM_PADDING;
            }
            _ => {}
        }
        match block_state {
            STREAM_MT_STREAM_PADDING => {
                loop {
                    if *in_pos >= in_size {
                        if action != LZMA_FINISH {
                            return LZMA_OK;
                        }
                        return if (*coder).pos == 0 {
                            LZMA_STREAM_END
                        } else {
                            LZMA_DATA_ERROR
                        };
                    }
                    if *in_0.add(*in_pos) != 0 {
                        break;
                    }
                    *in_pos += 1;
                    (*coder).progress_in += 1;
                    (*coder).pos = ((*coder).pos + 1) & 3;
                }
                if (*coder).pos != 0 {
                    *in_pos += 1;
                    (*coder).progress_in += 1;
                    return LZMA_DATA_ERROR;
                }
                let ret__7: lzma_ret = stream_decoder_reset(coder, allocator);
                if ret__7 != LZMA_OK {
                    return ret__7;
                }
                block_state = STREAM_MT_RESTART_LOOP;
            }
            STREAM_MT_BLOCK_THR_INIT => {
                if let Some(ret) = stream_decode_mt_thread_init(
                    coder,
                    allocator,
                    out,
                    out_pos,
                    out_size,
                    ::core::ptr::addr_of_mut!(wait_abs),
                    ::core::ptr::addr_of_mut!(has_blocked),
                ) {
                    return ret;
                }
                continue;
            }
            _ => {}
        }
        match block_state {
            STREAM_MT_BLOCK_THR_RUN => {
                if action == LZMA_FINISH && (*coder).fail_fast {
                    let in_avail: size_t = in_size - *in_pos;
                    let in_needed: size_t = (*(*coder).thr).in_size - (*(*coder).thr).in_filled;
                    if in_avail < in_needed {
                        threads_stop(coder);
                        return LZMA_DATA_ERROR;
                    }
                }
                let mut cur_in_filled: size_t = (*(*coder).thr).in_filled;
                lzma_bufcpy(
                    in_0,
                    in_pos,
                    in_size,
                    (*(*coder).thr).in_0,
                    ::core::ptr::addr_of_mut!(cur_in_filled),
                    (*(*coder).thr).in_size,
                );
                let mut mythread_i_1517: c_uint = 0;
                while if mythread_i_1517 != 0 {
                    mythread_mutex_unlock(::core::ptr::addr_of_mut!((*(*coder).thr).mutex));
                    0
                } else {
                    mythread_mutex_lock(::core::ptr::addr_of_mut!((*(*coder).thr).mutex));
                    1
                } != 0
                {
                    let mut mythread_j_1517: c_uint = 0;
                    while mythread_j_1517 == 0 {
                        (*(*coder).thr).in_filled = cur_in_filled;
                        mythread_cond_signal(::core::ptr::addr_of_mut!((*(*coder).thr).cond));
                        mythread_j_1517 = 1;
                    }
                    mythread_i_1517 = 1;
                }
                let ret__2: lzma_ret = read_output_and_wait(
                    coder,
                    allocator,
                    out,
                    out_pos,
                    out_size,
                    core::ptr::null_mut(),
                    waiting_allowed && *in_pos == in_size,
                    ::core::ptr::addr_of_mut!(wait_abs),
                    ::core::ptr::addr_of_mut!(has_blocked),
                );
                if ret__2 != LZMA_OK {
                    return ret__2;
                }
                if (*coder).pending_error != LZMA_OK {
                    (*coder).sequence = SEQ_ERROR;
                } else {
                    if (*(*coder).thr).in_filled < (*(*coder).thr).in_size {
                        return LZMA_OK;
                    }
                    (*coder).thr = core::ptr::null_mut();
                    (*coder).sequence = SEQ_BLOCK_HEADER;
                }
            }
            _ => {}
        }
    }
}
unsafe fn stream_decoder_mt_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    // stream_decoder_mt_end() in stream_decoder_mt.c likewise frees the coder
    // without destroying coder->mutex and coder->cond. Kept that way to stay
    // identical to the C original.
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    threads_end(coder, allocator);
    lzma_outq_end(::core::ptr::addr_of_mut!((*coder).outq), allocator);
    lzma_next_end(::core::ptr::addr_of_mut!((*coder).block_decoder), allocator);
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
        allocator,
    );
    lzma_index_hash_end((*coder).index_hash, allocator);
    crate::alloc::internal_free(coder, allocator);
}
unsafe fn stream_decoder_mt_get_check(coder_ptr: *const c_void) -> lzma_check {
    return unsafe {
        let coder: *const lzma_stream_coder = coder_ptr as *const lzma_stream_coder;
        (*coder).stream_flags.check
    };
}
unsafe fn stream_decoder_mt_memconfig(
    coder_ptr: *mut c_void,
    memusage: *mut u64,
    old_memlimit: *mut u64,
    new_memlimit: u64,
) -> lzma_ret {
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    let mut mythread_i_1829: c_uint = 0;
    while if mythread_i_1829 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_1829: c_uint = 0;
        while mythread_j_1829 == 0 {
            *memusage = (*coder).mem_direct_mode
                + (*coder).mem_in_use
                + (*coder).mem_cached
                + (*coder).outq.mem_allocated;
            mythread_j_1829 = 1;
        }
        mythread_i_1829 = 1;
    }
    if *memusage < LZMA_MEMUSAGE_BASE {
        *memusage = LZMA_MEMUSAGE_BASE;
    }
    *old_memlimit = (*coder).memlimit_stop;
    if new_memlimit != 0 {
        if new_memlimit < *memusage {
            return LZMA_MEMLIMIT_ERROR;
        }
        (*coder).memlimit_stop = new_memlimit;
    }
    LZMA_OK
}
unsafe fn stream_decoder_mt_get_progress(
    coder_ptr: *mut c_void,
    progress_in: &mut u64,
    progress_out: &mut u64,
) {
    let coder: *mut lzma_stream_coder = coder_ptr as *mut lzma_stream_coder;
    let mut mythread_i_1862: c_uint = 0;
    while if mythread_i_1862 != 0 {
        mythread_mutex_unlock(::core::ptr::addr_of_mut!((*coder).mutex));
        0
    } else {
        mythread_mutex_lock(::core::ptr::addr_of_mut!((*coder).mutex));
        1
    } != 0
    {
        let mut mythread_j_1862: c_uint = 0;
        while mythread_j_1862 == 0 {
            *progress_in = (*coder).progress_in;
            *progress_out = (*coder).progress_out;
            let mut i: size_t = 0;
            while i < (*coder).threads_initialized as size_t {
                let mut mythread_i_1867: c_uint = 0;
                while if mythread_i_1867 != 0 {
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
                    let mut mythread_j_1867: c_uint = 0;
                    while mythread_j_1867 == 0 {
                        *progress_in += (*(*coder).threads.add(i)).progress_in as u64;
                        *progress_out += (*(*coder).threads.add(i)).progress_out as u64;
                        mythread_j_1867 = 1;
                    }
                    mythread_i_1867 = 1;
                }
                i += 1;
            }
            mythread_j_1862 = 1;
        }
        mythread_i_1862 = 1;
    }
}
unsafe fn stream_decoder_mt_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    options: *const lzma_mt,
) -> lzma_ret {
    let mut coder: *mut lzma_stream_coder = core::ptr::null_mut();
    if (*options).threads == 0 || (*options).threads > LZMA_THREADS_MAX {
        return LZMA_OPTIONS_ERROR;
    }
    if (*options).flags & !(LZMA_SUPPORTED_FLAGS as u32) != 0 {
        return LZMA_OPTIONS_ERROR;
    }
    if core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_mt) -> lzma_ret>,
        uintptr_t,
    >(Some(
        stream_decoder_mt_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_mt) -> lzma_ret,
    )) != (*next).init
    {
        lzma_next_end(next, allocator);
    }
    (*next).init = core::mem::transmute::<
        Option<unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_mt) -> lzma_ret>,
        uintptr_t,
    >(Some(
        stream_decoder_mt_init
            as unsafe fn(*mut lzma_next_coder, *const lzma_allocator, *const lzma_mt) -> lzma_ret,
    ));
    coder = (*next).coder as *mut lzma_stream_coder;
    if coder.is_null() {
        coder = crate::alloc::internal_alloc_object::<lzma_stream_coder>(allocator);
        if coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*next).coder = coder as *mut c_void;
        if mythread_mutex_init(::core::ptr::addr_of_mut!((*coder).mutex)) != 0 {
            crate::alloc::internal_free(coder, allocator);
            return LZMA_MEM_ERROR;
        }
        if mythread_cond_init(::core::ptr::addr_of_mut!((*coder).cond)) != 0 {
            mythread_mutex_destroy(::core::ptr::addr_of_mut!((*coder).mutex));
            crate::alloc::internal_free(coder, allocator);
            return LZMA_MEM_ERROR;
        }
        (*next).code = Some(
            stream_decode_mt
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
        (*next).end =
            Some(stream_decoder_mt_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*next).get_check =
            Some(stream_decoder_mt_get_check as unsafe fn(*const c_void) -> lzma_check);
        (*next).memconfig = Some(
            stream_decoder_mt_memconfig
                as unsafe fn(*mut c_void, *mut u64, *mut u64, u64) -> lzma_ret,
        );
        (*next).get_progress = Some(
            stream_decoder_mt_get_progress as unsafe fn(*mut c_void, &mut u64, &mut u64) -> (),
        );
        (*coder).filters[0].id = LZMA_VLI_UNKNOWN;
        core::ptr::write_bytes(
            ::core::ptr::addr_of_mut!((*coder).outq) as *mut u8,
            0 as u8,
            core::mem::size_of::<lzma_outq>(),
        );
        (*coder).block_decoder = lzma_next_coder_s {
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
        (*coder).mem_direct_mode = 0;
        (*coder).index_hash = core::ptr::null_mut();
        (*coder).threads = core::ptr::null_mut();
        (*coder).threads_free = core::ptr::null_mut();
        (*coder).threads_initialized = 0;
        // threads_end frees `threads` with `threads_max` as the element
        // count, so it must not be read uninitialized on the first init.
        (*coder).threads_max = 0;
    }
    lzma_filters_free(
        ::core::ptr::addr_of_mut!((*coder).filters) as *mut lzma_filter,
        allocator,
    );
    threads_end(coder, allocator);
    (*coder).mem_in_use = 0;
    (*coder).mem_cached = 0;
    (*coder).mem_next_block = 0;
    (*coder).progress_in = 0;
    (*coder).progress_out = 0;
    (*coder).sequence = SEQ_STREAM_HEADER;
    (*coder).thread_error = LZMA_OK;
    (*coder).pending_error = LZMA_OK;
    (*coder).thr = core::ptr::null_mut();
    (*coder).timeout = (*options).timeout;
    (*coder).memlimit_threading = if 1 > (*options).memlimit_threading {
        1
    } else {
        (*options).memlimit_threading
    };
    (*coder).memlimit_stop = if 1 > (*options).memlimit_stop {
        1
    } else {
        (*options).memlimit_stop
    };
    if (*coder).memlimit_threading > (*coder).memlimit_stop {
        (*coder).memlimit_threading = (*coder).memlimit_stop;
    }
    (*coder).tell_no_check = (*options).flags & LZMA_TELL_NO_CHECK as u32 != 0;
    (*coder).tell_unsupported_check = (*options).flags & LZMA_TELL_UNSUPPORTED_CHECK as u32 != 0;
    (*coder).tell_any_check = (*options).flags & LZMA_TELL_ANY_CHECK as u32 != 0;
    (*coder).ignore_check = (*options).flags & LZMA_IGNORE_CHECK as u32 != 0;
    (*coder).concatenated = (*options).flags & LZMA_CONCATENATED as u32 != 0;
    (*coder).fail_fast = (*options).flags & LZMA_FAIL_FAST as u32 != 0;
    (*coder).first_stream = true;
    (*coder).out_was_filled = false;
    (*coder).pos = 0;
    (*coder).threads_max = (*options).threads;
    let ret_: lzma_ret = lzma_outq_init(
        ::core::ptr::addr_of_mut!((*coder).outq),
        allocator,
        (*coder).threads_max,
    );
    if ret_ != LZMA_OK {
        return ret_;
    }
    stream_decoder_reset(coder, allocator)
}
pub unsafe fn lzma_stream_decoder_mt(strm: *mut lzma_stream, options: *const lzma_mt) -> lzma_ret {
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = stream_decoder_mt_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        options,
    );
    if ret__0 != LZMA_OK {
        lzma_end(strm);
        return ret__0;
    }
    (*(*strm).internal).supported_actions[LZMA_RUN as usize] = true;
    (*(*strm).internal).supported_actions[LZMA_FINISH as usize] = true;
    LZMA_OK
}
