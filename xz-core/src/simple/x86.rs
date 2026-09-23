use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_simple_x86 {
    pub prev_mask: u32,
    pub prev_pos: u32,
}
fn x86_code_impl(
    simple: &mut lzma_simple_x86,
    now_pos: u32,
    is_encoder: bool,
    buffer: &mut [u8],
) -> size_t {
    const MASK_TO_BIT_NUMBER: [u32; 5] = [0, 1, 2, 2, 3];
    let mut prev_mask: u32 = simple.prev_mask;
    let mut prev_pos: u32 = simple.prev_pos;
    if buffer.len() < 5 {
        return 0;
    }
    if now_pos.wrapping_sub(prev_pos) > 5 {
        prev_pos = now_pos.wrapping_sub(5);
    }
    let limit: size_t = buffer.len() - 5;
    let ptr = buffer.as_mut_ptr();
    let mut buffer_pos: size_t = 0;
    while buffer_pos <= limit {
        let cur = unsafe { ptr.add(buffer_pos) };
        let mut b: u8 = unsafe { *cur };
        if b != 0xe8 && b != 0xe9 {
            buffer_pos += 1;
        } else {
            let offset: u32 = now_pos
                .wrapping_add(buffer_pos as u32)
                .wrapping_sub(prev_pos);
            prev_pos = now_pos.wrapping_add(buffer_pos as u32);
            if offset > 5 {
                prev_mask = 0;
            } else {
                let mut i: u32 = 0;
                while i < offset {
                    prev_mask &= 0x77;
                    prev_mask <<= 1;
                    i += 1;
                }
            }
            b = unsafe { *cur.add(4) };
            if (b == 0 || b == 0xff) && prev_mask >> 1 <= 4 && prev_mask >> 1 != 3 {
                let mut src: u32 = unsafe {
                    (b as u32) << 24
                        | (*cur.add(3) as u32) << 16
                        | (*cur.add(2) as u32) << 8
                        | *cur.add(1) as u32
                };
                let dest: u32 = loop {
                    let dest = if is_encoder {
                        src.wrapping_add(now_pos.wrapping_add(buffer_pos as u32).wrapping_add(5))
                    } else {
                        src.wrapping_sub(now_pos.wrapping_add(buffer_pos as u32).wrapping_add(5))
                    };
                    if prev_mask == 0 {
                        break dest;
                    }
                    let i_0: u32 =
                        unsafe { *MASK_TO_BIT_NUMBER.as_ptr().add((prev_mask >> 1) as usize) };
                    b = (dest >> (24u32).wrapping_sub(i_0.wrapping_mul(8))) as u8;
                    if b != 0 && b != 0xff {
                        break dest;
                    }
                    src =
                        dest ^ (1u32 << (32u32).wrapping_sub(i_0.wrapping_mul(8))).wrapping_sub(1);
                };
                unsafe {
                    *cur.add(4) = !(dest >> 24 & 1).wrapping_sub(1) as u8;
                    *cur.add(3) = (dest >> 16) as u8;
                    *cur.add(2) = (dest >> 8) as u8;
                    *cur.add(1) = dest as u8;
                }
                buffer_pos += 5;
                prev_mask = 0;
            } else {
                buffer_pos += 1;
                prev_mask |= 1;
                if b == 0 || b == 0xff {
                    prev_mask |= 0x10;
                }
            }
        }
    }
    simple.prev_mask = prev_mask;
    simple.prev_pos = prev_pos;
    buffer_pos
}
unsafe fn x86_code(
    simple_ptr: *mut c_void,
    now_pos: u32,
    is_encoder: bool,
    buffer: *mut u8,
    size: size_t,
) -> size_t {
    if size == 0 {
        return 0;
    }
    x86_code_impl(
        &mut *(simple_ptr as *mut lzma_simple_x86),
        now_pos,
        is_encoder,
        core::slice::from_raw_parts_mut(buffer, size),
    )
}
unsafe fn x86_coder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
    is_encoder: bool,
) -> lzma_ret {
    let ret: lzma_ret = lzma_simple_coder_init(
        next,
        allocator,
        filters,
        x86_code as lzma_simple_filter_function,
        core::mem::size_of::<lzma_simple_x86>(),
        5,
        1,
        is_encoder,
    );
    if ret == LZMA_OK {
        let coder: *mut lzma_simple_coder = (*next).coder_as::<lzma_simple_coder>();
        let simple: *mut lzma_simple_x86 = (*coder).simple as *mut lzma_simple_x86;
        (*simple).prev_mask = 0;
        (*simple).prev_pos = (-5_i32) as u32;
    }
    ret
}
pub(crate) unsafe fn lzma_simple_x86_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
) -> lzma_ret {
    x86_coder_init(next, allocator, filters, true)
}
pub unsafe fn lzma_bcj_x86_encode(start_offset: u32, buf: *mut u8, size: size_t) -> size_t {
    let mut simple: lzma_simple_x86 = lzma_simple_x86 {
        prev_mask: 0,
        prev_pos: (-5_i32) as u32,
    };
    if size == 0 {
        return 0;
    }
    x86_code_impl(
        &mut simple,
        start_offset,
        true,
        core::slice::from_raw_parts_mut(buf, size),
    )
}
pub(crate) unsafe fn lzma_simple_x86_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
) -> lzma_ret {
    x86_coder_init(next, allocator, filters, false)
}
pub unsafe fn lzma_bcj_x86_decode(start_offset: u32, buf: *mut u8, size: size_t) -> size_t {
    let mut simple: lzma_simple_x86 = lzma_simple_x86 {
        prev_mask: 0,
        prev_pos: (-5_i32) as u32,
    };
    if size == 0 {
        return 0;
    }
    x86_code_impl(
        &mut simple,
        start_offset,
        false,
        core::slice::from_raw_parts_mut(buf, size),
    )
}
