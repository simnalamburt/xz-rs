use crate::types::*;
fn copy_and_encode(coder: &mut lzma_delta_coder, input: &[u8], output: &mut [u8]) {
    let distance: size_t = coder.distance;
    debug_assert_eq!(input.len(), output.len());
    let mut i: usize = 0;
    while i < input.len() {
        let tmp: u8 = coder.history[(distance.wrapping_add(coder.pos as size_t) & 0xff) as usize];
        let byte = input[i];
        coder.history[(coder.pos & 0xff) as usize] = byte;
        coder.pos = coder.pos.wrapping_sub(1);
        output[i] = byte.wrapping_sub(tmp);
        i += 1;
    }
}
fn encode_in_place(coder: &mut lzma_delta_coder, buffer: &mut [u8]) {
    let distance: size_t = coder.distance;
    let mut i: usize = 0;
    while i < buffer.len() {
        let tmp: u8 = coder.history[(distance.wrapping_add(coder.pos as size_t) & 0xff) as usize];
        let byte = buffer[i];
        coder.history[(coder.pos & 0xff) as usize] = byte;
        coder.pos = coder.pos.wrapping_sub(1);
        buffer[i] = byte.wrapping_sub(tmp);
        i += 1;
    }
}
unsafe fn delta_encode(
    coder: &mut lzma_delta_coder,
    allocator: *const lzma_allocator,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    action: lzma_action,
) -> lzma_ret {
    match coder.next.code {
        None => {
            debug_assert!(in_size >= *in_pos);
            debug_assert!(out_size >= *out_pos);
            let in_avail: size_t = in_size - *in_pos;
            let out_avail: size_t = out_size - *out_pos;
            let size: size_t = if in_avail < out_avail {
                in_avail
            } else {
                out_avail
            };
            if size > 0 {
                copy_and_encode(
                    coder,
                    core::slice::from_raw_parts(input.add(*in_pos), size),
                    core::slice::from_raw_parts_mut(out.add(*out_pos), size),
                );
            }
            *in_pos += size;
            *out_pos += size;
            if action != LZMA_RUN && *in_pos == in_size {
                LZMA_STREAM_END
            } else {
                LZMA_OK
            }
        }
        Some(code) => {
            let out_start: size_t = *out_pos;
            let ret = code(
                coder.next.coder,
                allocator,
                input,
                in_pos,
                in_size,
                out,
                out_pos,
                out_size,
                action,
            );
            debug_assert!(*out_pos >= out_start);
            let size_0: size_t = *out_pos - out_start;
            if size_0 > 0 {
                encode_in_place(
                    coder,
                    core::slice::from_raw_parts_mut(out.add(out_start), size_0),
                );
            }
            ret
        }
    }
}
unsafe fn delta_encoder_update(
    coder: &mut lzma_delta_coder,
    allocator: *const lzma_allocator,
    _filters_null: *const lzma_filter,
    reversed_filters: *const lzma_filter,
) -> lzma_ret {
    lzma_next_filter_update(
        ::core::ptr::addr_of_mut!(coder.next),
        allocator,
        reversed_filters.offset(1),
    )
}
pub(crate) unsafe fn lzma_delta_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
) -> lzma_ret {
    (*next).code = coder_code_fn!(delta_encode, lzma_delta_coder);
    (*next).update = coder_update_fn!(delta_encoder_update, lzma_delta_coder);
    lzma_delta_coder_init(next, allocator, filters)
}
pub(crate) unsafe fn lzma_delta_props_encode(options: *const c_void, out: *mut u8) -> lzma_ret {
    if lzma_delta_coder_memusage(options) == UINT64_MAX {
        return LZMA_PROG_ERROR;
    }
    let opt: *const lzma_options_delta = options as *const lzma_options_delta;
    debug_assert!((*opt).dist >= LZMA_DELTA_DIST_MIN);
    *out = ((*opt).dist - LZMA_DELTA_DIST_MIN) as u8;
    LZMA_OK
}
