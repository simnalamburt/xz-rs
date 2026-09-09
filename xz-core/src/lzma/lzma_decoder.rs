use crate::lz::lz_decoder::{lzma_lz_decoder_init, lzma_lz_decoder_memusage, lzma_lz_options};
use crate::types::*;
#[cfg(all(target_arch = "x86", target_feature = "sse2"))]
use core::arch::x86::{__m128i, _mm_loadu_si128, _mm_storeu_si128};
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
use core::arch::x86_64::{__m128i, _mm_loadu_si128, _mm_storeu_si128};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_lzma1_decoder {
    pub literal: [probability; 12288],
    pub is_match: [[probability; 16]; 12],
    pub is_rep: [probability; 12],
    pub is_rep0: [probability; 12],
    pub is_rep1: [probability; 12],
    pub is_rep2: [probability; 12],
    pub is_rep0_long: [[probability; 16]; 12],
    pub dist_slot: [[probability; 64]; 4],
    pub pos_special: [probability; 114],
    pub pos_align: [probability; 16],
    pub match_len_decoder: lzma_length_decoder,
    pub rep_len_decoder: lzma_length_decoder,
    pub rc: lzma_range_decoder,
    pub state: lzma_lzma_state,
    pub rep0: u32,
    pub rep1: u32,
    pub rep2: u32,
    pub rep3: u32,
    pub pos_mask: u32,
    pub literal_context_bits: u32,
    pub literal_mask: u32,
    pub uncompressed_size: lzma_vli,
    pub allow_eopm: bool,
    pub sequence: lzma_decoder_seq,
    pub probs: *mut probability,
    pub symbol: u32,
    pub limit: u32,
    pub offset: u32,
    pub len: u32,
}
pub type lzma_decoder_seq = c_uint;
pub const SEQ_COPY: lzma_decoder_seq = 22;
pub const SEQ_REP_LEN_BITTREE: lzma_decoder_seq = 21;
pub const SEQ_REP_LEN_CHOICE2: lzma_decoder_seq = 20;
pub const SEQ_REP_LEN_CHOICE: lzma_decoder_seq = 19;
pub const SEQ_IS_REP2: lzma_decoder_seq = 18;
pub const SEQ_IS_REP1: lzma_decoder_seq = 17;
pub const SEQ_IS_REP0_LONG: lzma_decoder_seq = 16;
pub const SEQ_SHORTREP: lzma_decoder_seq = 15;
pub const SEQ_IS_REP0: lzma_decoder_seq = 14;
pub const SEQ_EOPM: lzma_decoder_seq = 13;
pub const SEQ_ALIGN: lzma_decoder_seq = 12;
pub const SEQ_DIRECT: lzma_decoder_seq = 11;
pub const SEQ_DIST_MODEL: lzma_decoder_seq = 10;
pub const SEQ_DIST_SLOT: lzma_decoder_seq = 9;
pub const SEQ_MATCH_LEN_BITTREE: lzma_decoder_seq = 8;
pub const SEQ_MATCH_LEN_CHOICE2: lzma_decoder_seq = 7;
pub const SEQ_MATCH_LEN_CHOICE: lzma_decoder_seq = 6;
pub const SEQ_IS_REP: lzma_decoder_seq = 5;
pub const SEQ_LITERAL_WRITE: lzma_decoder_seq = 4;
pub const SEQ_LITERAL_MATCHED: lzma_decoder_seq = 3;
pub const SEQ_LITERAL: lzma_decoder_seq = 2;
pub const SEQ_IS_MATCH: lzma_decoder_seq = 1;
pub const SEQ_NORMALIZE: lzma_decoder_seq = 0;
type DecoderBlockState = u8;
const BLOCK_NORMALIZE_OR_IS_MATCH: DecoderBlockState = 0;
const BLOCK_LITERAL: DecoderBlockState = 1;
const BLOCK_LITERAL_MATCHED: DecoderBlockState = 2;
const BLOCK_LITERAL_WRITE: DecoderBlockState = 3;
const BLOCK_IS_REP: DecoderBlockState = 4;
const BLOCK_MATCH_LEN_CHOICE: DecoderBlockState = 5;
const BLOCK_MATCH_LEN_CHOICE2: DecoderBlockState = 6;
const BLOCK_MATCH_LEN_BITTREE: DecoderBlockState = 7;
const BLOCK_DIST_SLOT: DecoderBlockState = 8;
const BLOCK_DIST_MODEL: DecoderBlockState = 9;
const BLOCK_DIRECT: DecoderBlockState = 10;
const BLOCK_ALIGN: DecoderBlockState = 11;
const BLOCK_EOPM: DecoderBlockState = 12;
const BLOCK_IS_REP0: DecoderBlockState = 13;
const BLOCK_IS_REP0_LONG: DecoderBlockState = 14;
const BLOCK_SHORTREP: DecoderBlockState = 15;
const BLOCK_IS_REP1: DecoderBlockState = 16;
const BLOCK_IS_REP2: DecoderBlockState = 17;
const BLOCK_REP_LEN_CHOICE: DecoderBlockState = 18;
const BLOCK_REP_LEN_CHOICE2: DecoderBlockState = 19;
const BLOCK_REP_LEN_BITTREE: DecoderBlockState = 20;
const BLOCK_COPY: DecoderBlockState = 21;
const BLOCK_RETURN: DecoderBlockState = 22;
const BLOCK_LEN_BITTREE_INIT: DecoderBlockState = 23;
const BLOCK_REP_LEN_PREPARE: DecoderBlockState = 24;
const BLOCK_DIST_SLOT_INIT: DecoderBlockState = 25;
const BLOCK_EOPM_IF_VALID: DecoderBlockState = 26;
const BLOCK_VALIDATE_DISTANCE: DecoderBlockState = 27;
const BLOCK_MAIN_LOOP: DecoderBlockState = 28;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_range_decoder {
    pub range: u32,
    pub code: u32,
    pub init_bytes_left: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_length_decoder {
    pub choice: probability,
    pub choice2: probability,
    pub low: [[probability; 8]; 16],
    pub mid: [[probability; 8]; 16],
    pub high: [probability; 256],
}
#[inline]
unsafe fn dict_get(dict: *const lzma_dict, distance: u32) -> u8 {
    *(*dict).buf.add(
        (*dict)
            .pos
            .wrapping_sub(distance as size_t)
            .wrapping_sub(1)
            .wrapping_add(if (distance as size_t) < (*dict).pos {
                0
            } else {
                (*dict).size.wrapping_sub(LZ_DICT_REPEAT_MAX as size_t)
            }),
    )
}
#[inline]
unsafe fn dict_get0(dict: *const lzma_dict) -> u8 {
    *(*dict).buf.add((*dict).pos.wrapping_sub(1))
}
#[inline]
unsafe fn dict_is_distance_valid(dict: *const lzma_dict, distance: size_t) -> bool {
    (*dict).full > distance
}
#[inline]
unsafe fn dict_repeat(dict: *mut lzma_dict, distance: u32, len: *mut u32) -> bool {
    let dict_avail: size_t = (*dict).limit.wrapping_sub((*dict).pos);
    let mut left: u32 = (if dict_avail < *len as size_t {
        dict_avail
    } else {
        *len as size_t
    }) as u32;
    *len = (*len).wrapping_sub(left);
    let mut back: size_t = (*dict).pos.wrapping_sub(distance as size_t).wrapping_sub(1);
    if distance as size_t >= (*dict).pos {
        back = back.wrapping_add((*dict).size.wrapping_sub(LZ_DICT_REPEAT_MAX as size_t));
    }
    if distance < left {
        loop {
            *(*dict).buf.add((*dict).pos) = *(*dict).buf.add(back);
            back += 1;
            (*dict).pos = (*dict).pos.wrapping_add(1);
            left -= 1;
            if left == 0 {
                break;
            }
        }
    } else {
        #[cfg(not(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2"
        )))]
        {
            const _: () = assert!(crate::lz::lz_decoder::LZ_DICT_EXTRA == 0);
            core::ptr::copy_nonoverlapping(
                (*dict).buf.add(back) as *const u8,
                (*dict).buf.add((*dict).pos) as *mut u8,
                left as size_t,
            );
            (*dict).pos = (*dict).pos.wrapping_add(left as size_t);
        }
        // This can copy up to 32 bytes more than required (if left == 0,
        // still 32 bytes); LZ_DICT_EXTRA in lz_decoder.rs reserves the
        // tail space and its cfg must match this branch. The assert below
        // turns a cfg that drifted apart into a build failure.
        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse2"
        ))]
        {
            const _: () = assert!(crate::lz::lz_decoder::LZ_DICT_EXTRA == 32);
            let mut pos: size_t = (*dict).pos;
            (*dict).pos = (*dict).pos.wrapping_add(left as size_t);
            loop {
                let x0 = _mm_loadu_si128((*dict).buf.add(back) as *const __m128i);
                let x1 = _mm_loadu_si128((*dict).buf.add(back + 16) as *const __m128i);
                back = back.wrapping_add(32);
                _mm_storeu_si128((*dict).buf.add(pos) as *mut __m128i, x0);
                _mm_storeu_si128((*dict).buf.add(pos + 16) as *mut __m128i, x1);
                pos = pos.wrapping_add(32);
                if pos >= (*dict).pos {
                    break;
                }
            }
        }
    }
    if !(*dict).has_wrapped {
        (*dict).full = (*dict).pos.wrapping_sub(LZ_DICT_INIT_POS as size_t);
    }
    *len != 0
}
#[inline]
unsafe fn dict_put(dict: *mut lzma_dict, byte: u8) {
    *(*dict).buf.add((*dict).pos) = byte;
    (*dict).pos = (*dict).pos.wrapping_add(1);
    if !(*dict).has_wrapped {
        (*dict).full = (*dict).pos.wrapping_sub(LZ_DICT_INIT_POS as size_t);
    }
}
#[inline]
unsafe fn dict_put_safe(dict: *mut lzma_dict, byte: u8) -> bool {
    if (*dict).pos == (*dict).limit {
        return true;
    }
    dict_put(dict, byte);
    false
}
pub const DIST_SLOTS: u32 = 1 << DIST_SLOT_BITS;
#[inline]
unsafe fn rc_read_init(
    rc: *mut lzma_range_decoder,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
) -> lzma_ret {
    while (*rc).init_bytes_left > 0 {
        if *in_pos == in_size {
            return LZMA_OK;
        }
        if (*rc).init_bytes_left == 5 && *input.add(*in_pos) != 0 {
            return LZMA_DATA_ERROR;
        }
        (*rc).code = (*rc).code << 8 | *input.add(*in_pos) as u32;
        *in_pos = (*in_pos).wrapping_add(1);
        (*rc).init_bytes_left = (*rc).init_bytes_left.wrapping_sub(1);
    }
    LZMA_STREAM_END
}
#[inline(never)]
fn resume_block_for_sequence(sequence: lzma_decoder_seq) -> DecoderBlockState {
    match sequence {
        0 | 1 => BLOCK_NORMALIZE_OR_IS_MATCH,
        2 => BLOCK_LITERAL,
        3 => BLOCK_LITERAL_MATCHED,
        4 => BLOCK_LITERAL_WRITE,
        5 => BLOCK_IS_REP,
        6 => BLOCK_MATCH_LEN_CHOICE,
        7 => BLOCK_MATCH_LEN_CHOICE2,
        8 => BLOCK_MATCH_LEN_BITTREE,
        9 => BLOCK_DIST_SLOT,
        10 => BLOCK_DIST_MODEL,
        11 => BLOCK_DIRECT,
        12 => BLOCK_ALIGN,
        13 => BLOCK_EOPM,
        14 => BLOCK_IS_REP0,
        16 => BLOCK_IS_REP0_LONG,
        15 => BLOCK_SHORTREP,
        17 => BLOCK_IS_REP1,
        18 => BLOCK_IS_REP2,
        19 => BLOCK_REP_LEN_CHOICE,
        20 => BLOCK_REP_LEN_CHOICE2,
        21 => BLOCK_REP_LEN_BITTREE,
        22 => BLOCK_COPY,
        _ => BLOCK_RETURN,
    }
}
#[inline(always)]
unsafe fn decoder_is_match_row(coder: *mut lzma_lzma1_decoder, state: u32) -> *mut probability {
    debug_assert!((state as usize) < STATES as usize);
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*coder).is_match) as *mut [probability; 16])
            .add(state as usize)
    ) as *mut probability
}
#[inline(always)]
unsafe fn decoder_is_rep_prob(coder: *mut lzma_lzma1_decoder, state: u32) -> *mut probability {
    debug_assert!((state as usize) < STATES as usize);
    (::core::ptr::addr_of_mut!((*coder).is_rep) as *mut probability).add(state as usize)
}
#[inline(always)]
unsafe fn decoder_is_rep0_prob(coder: *mut lzma_lzma1_decoder, state: u32) -> *mut probability {
    debug_assert!((state as usize) < STATES as usize);
    (::core::ptr::addr_of_mut!((*coder).is_rep0) as *mut probability).add(state as usize)
}
#[inline(always)]
unsafe fn decoder_is_rep1_prob(coder: *mut lzma_lzma1_decoder, state: u32) -> *mut probability {
    debug_assert!((state as usize) < STATES as usize);
    (::core::ptr::addr_of_mut!((*coder).is_rep1) as *mut probability).add(state as usize)
}
#[inline(always)]
unsafe fn decoder_is_rep2_prob(coder: *mut lzma_lzma1_decoder, state: u32) -> *mut probability {
    debug_assert!((state as usize) < STATES as usize);
    (::core::ptr::addr_of_mut!((*coder).is_rep2) as *mut probability).add(state as usize)
}
#[inline(always)]
unsafe fn decoder_is_rep0_long_row(coder: *mut lzma_lzma1_decoder, state: u32) -> *mut probability {
    debug_assert!((state as usize) < STATES as usize);
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*coder).is_rep0_long) as *mut [probability; 16])
            .add(state as usize)
    ) as *mut probability
}
#[inline(always)]
unsafe fn decoder_dist_slot_row(
    coder: *mut lzma_lzma1_decoder,
    dist_state: u32,
) -> *mut probability {
    debug_assert!((dist_state as usize) < DIST_STATES as usize);
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*coder).dist_slot) as *mut [probability; 64])
            .add(dist_state as usize)
    ) as *mut probability
}
#[inline(always)]
unsafe fn decoder_pos_align_prob(coder: *mut lzma_lzma1_decoder, index: u32) -> *mut probability {
    debug_assert!((index as usize) < ALIGN_SIZE as usize);
    (::core::ptr::addr_of_mut!((*coder).pos_align) as *mut probability).add(index as usize)
}
#[inline(always)]
unsafe fn length_low_row(
    len_decoder: *mut lzma_length_decoder,
    pos_state: u32,
) -> *mut probability {
    debug_assert!((pos_state as usize) < (1 << LZMA_PB_MAX) as usize);
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*len_decoder).low) as *mut [probability; 8])
            .add(pos_state as usize)
    ) as *mut probability
}
#[inline(always)]
unsafe fn length_mid_row(
    len_decoder: *mut lzma_length_decoder,
    pos_state: u32,
) -> *mut probability {
    debug_assert!((pos_state as usize) < (1 << LZMA_PB_MAX) as usize);
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*len_decoder).mid) as *mut [probability; 8])
            .add(pos_state as usize)
    ) as *mut probability
}
#[inline(always)]
unsafe fn length_high_probs(len_decoder: *mut lzma_length_decoder) -> *mut probability {
    ::core::ptr::addr_of_mut!((*len_decoder).high) as *mut probability
}
#[inline(always)]
unsafe fn prob_update_0(prob: *mut probability) {
    *prob = (*prob as u32)
        .wrapping_add(RC_BIT_MODEL_TOTAL.wrapping_sub(*prob as u32) >> RC_MOVE_BITS)
        as probability;
}
#[inline(always)]
unsafe fn prob_update_1(prob: *mut probability) {
    *prob = *prob - (*prob >> RC_MOVE_BITS);
}

macro_rules! rc_normalize {
    ($rc:ident, $rc_in_ptr:ident) => {
        if $rc.range < RC_TOP_VALUE as u32 {
            $rc.range <<= RC_SHIFT_BITS;
            $rc.code = $rc.code << RC_SHIFT_BITS | *$rc_in_ptr as u32;
            $rc_in_ptr = $rc_in_ptr.offset(1);
        }
    };
}

#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "64")))]
macro_rules! rc_bittree_step {
    ($rc:ident, $rc_bound:ident, $prob:expr, $symbol:ident) => {{
        let prob = $prob;
        $rc_bound = ($rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*prob as u32);
        if $rc.code < $rc_bound {
            $rc.range = $rc_bound;
            prob_update_0(prob);
            $symbol <<= 1;
        } else {
            $rc.range = $rc.range.wrapping_sub($rc_bound);
            $rc.code = $rc.code.wrapping_sub($rc_bound);
            prob_update_1(prob);
            $symbol = ($symbol << 1).wrapping_add(1);
        }
    }};
}

#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "64")))]
macro_rules! rc_bittree8 {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:expr, $final_add:expr, $symbol:ident) => {{
        let probs_base = $probs_base;
        $symbol = 1;
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        $symbol = $symbol.wrapping_add(($final_add) as u32);
    }};
}

#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "64")))]
macro_rules! rc_matched_literal_step {
    (
        $rc:ident,
        $rc_in_ptr:ident,
        $rc_bound:ident,
        $probs_base:ident,
        $t_match_byte:ident,
        $t_match_bit:ident,
        $t_subcoder_index:ident,
        $t_offset:ident,
        $symbol:ident
    ) => {{
        $t_match_byte <<= 1;
        $t_match_bit = $t_match_byte & $t_offset;
        $t_subcoder_index = $t_offset.wrapping_add($t_match_bit).wrapping_add($symbol);
        rc_normalize!($rc, $rc_in_ptr);
        let prob = $probs_base.add($t_subcoder_index as usize);
        $rc_bound = ($rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*prob as u32);
        if $rc.code < $rc_bound {
            $rc.range = $rc_bound;
            prob_update_0(prob);
            $symbol <<= 1;
            $t_offset &= !$t_match_bit;
        } else {
            $rc.range = $rc.range.wrapping_sub($rc_bound);
            $rc.code = $rc.code.wrapping_sub($rc_bound);
            prob_update_1(prob);
            $symbol = ($symbol << 1).wrapping_add(1);
            $t_offset &= $t_match_bit;
        }
    }};
}

#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "64")))]
macro_rules! rc_matched_literal {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:expr, $match_byte:expr, $symbol:ident) => {{
        let probs_base = $probs_base;
        let mut t_match_byte = ($match_byte) as u32;
        let mut t_match_bit: u32;
        let mut t_subcoder_index: u32;
        let mut t_offset: u32 = 0x100;
        $symbol = 1;
        rc_matched_literal_step!(
            $rc,
            $rc_in_ptr,
            $rc_bound,
            probs_base,
            t_match_byte,
            t_match_bit,
            t_subcoder_index,
            t_offset,
            $symbol
        );
        rc_matched_literal_step!(
            $rc,
            $rc_in_ptr,
            $rc_bound,
            probs_base,
            t_match_byte,
            t_match_bit,
            t_subcoder_index,
            t_offset,
            $symbol
        );
        rc_matched_literal_step!(
            $rc,
            $rc_in_ptr,
            $rc_bound,
            probs_base,
            t_match_byte,
            t_match_bit,
            t_subcoder_index,
            t_offset,
            $symbol
        );
        rc_matched_literal_step!(
            $rc,
            $rc_in_ptr,
            $rc_bound,
            probs_base,
            t_match_byte,
            t_match_bit,
            t_subcoder_index,
            t_offset,
            $symbol
        );
        rc_matched_literal_step!(
            $rc,
            $rc_in_ptr,
            $rc_bound,
            probs_base,
            t_match_byte,
            t_match_bit,
            t_subcoder_index,
            t_offset,
            $symbol
        );
        rc_matched_literal_step!(
            $rc,
            $rc_in_ptr,
            $rc_bound,
            probs_base,
            t_match_byte,
            t_match_bit,
            t_subcoder_index,
            t_offset,
            $symbol
        );
        rc_matched_literal_step!(
            $rc,
            $rc_in_ptr,
            $rc_bound,
            probs_base,
            t_match_byte,
            t_match_bit,
            t_subcoder_index,
            t_offset,
            $symbol
        );
        rc_matched_literal_step!(
            $rc,
            $rc_in_ptr,
            $rc_bound,
            probs_base,
            t_match_byte,
            t_match_bit,
            t_subcoder_index,
            t_offset,
            $symbol
        );
    }};
}

// x86-64 inline assembly variants of the multi-bit range decoder
// operations (range_decoder.h, LZMA_RANGE_DECODER_CONFIG 0x1F0),
// generated from the C asm macros via c2rust and cleaned up. The
// normalization label is 8 and the rc_direct loop label is 9 because
// LLVM can misparse asm labels 0 and 1.
#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
const RC_BIT_MODEL_OFFSET: i32 =
    ((1u32 << RC_MOVE_BITS) - 1).wrapping_sub(RC_BIT_MODEL_TOTAL) as i32;

#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "64")))]
macro_rules! rc_bittree3 {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:expr, $final_add:expr, $symbol:ident) => {{
        let probs_base = $probs_base;
        $symbol = 1;
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        $symbol = $symbol.wrapping_add(($final_add) as u32);
    }};
}

#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "64")))]
macro_rules! rc_bittree6 {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:expr, $final_add:expr, $symbol:ident) => {{
        let probs_base = $probs_base;
        $symbol = 1;
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        rc_normalize!($rc, $rc_in_ptr);
        rc_bittree_step!($rc, $rc_bound, probs_base.add($symbol as usize), $symbol);
        $symbol = $symbol.wrapping_add(($final_add) as u32);
    }};
}

#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
macro_rules! rc_bittree3 {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:expr, $final_add:expr, $symbol:ident) => {{
        let probs_base: *mut probability = $probs_base;
        core::arch::asm!(
            "movzwl 2({probs_base}), {prob0:e}",
            "mov $2, {symbol:e}",
            "movzwl 4({probs_base}), {prob1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 6({probs_base}), {t0:e}",
            "cmovae {t0:e}, {prob1:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob0:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob0:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob1:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob0:e}",
            "lea {bit_model_offset}({prob1:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob1:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob1:e}",
            "mov {prob1:x}, ({probs_base}, {t1:r}, 1)",
            "add {symbol:e}, {symbol:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob0:e}, {t0:e}",
            "sbb ${last_sbb}, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, ({probs_base}, {t1:r}, 1)",
            range = inout(reg) $rc.range,
            code = inout(reg) $rc.code,
            t0 = out(reg) _,
            t1 = out(reg) _,
            prob0 = out(reg) _,
            prob1 = out(reg) _,
            symbol = out(reg) $symbol,
            in_ptr = inout(reg) $rc_in_ptr,
            probs_base = in(reg) probs_base,
            last_sbb = const -1i32 - ($final_add),
            top_value = const RC_TOP_VALUE,
            shift_bits = const RC_SHIFT_BITS,
            bit_model_total_bits = const RC_BIT_MODEL_TOTAL_BITS,
            bit_model_offset = const RC_BIT_MODEL_OFFSET,
            move_bits = const RC_MOVE_BITS,
            options(att_syntax),
        );
    }};
}

#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
macro_rules! rc_bittree6 {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:expr, $final_add:expr, $symbol:ident) => {{
        let probs_base: *mut probability = $probs_base;
        core::arch::asm!(
            "movzwl 2({probs_base}), {prob0:e}",
            "mov $2, {symbol:e}",
            "movzwl 4({probs_base}), {prob1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 6({probs_base}), {t0:e}",
            "cmovae {t0:e}, {prob1:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob0:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob0:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob1:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob0:e}",
            "lea {bit_model_offset}({prob1:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob1:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob1:e}",
            "mov {prob1:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob1:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob0:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob0:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob1:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob0:e}",
            "lea {bit_model_offset}({prob1:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob1:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob1:e}",
            "mov {prob1:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob1:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob0:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, ({probs_base}, {t1:r}, 1)",
            "add {symbol:e}, {symbol:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob1:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob1:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob1:e}, {t0:e}",
            "sbb ${last_sbb}, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob1:e}",
            "mov {prob1:x}, ({probs_base}, {t1:r}, 1)",
            range = inout(reg) $rc.range,
            code = inout(reg) $rc.code,
            t0 = out(reg) _,
            t1 = out(reg) _,
            prob0 = out(reg) _,
            prob1 = out(reg) _,
            symbol = out(reg) $symbol,
            in_ptr = inout(reg) $rc_in_ptr,
            probs_base = in(reg) probs_base,
            last_sbb = const -1i32 - ($final_add),
            top_value = const RC_TOP_VALUE,
            shift_bits = const RC_SHIFT_BITS,
            bit_model_total_bits = const RC_BIT_MODEL_TOTAL_BITS,
            bit_model_offset = const RC_BIT_MODEL_OFFSET,
            move_bits = const RC_MOVE_BITS,
            options(att_syntax),
        );
    }};
}

#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
macro_rules! rc_bittree8 {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:expr, $final_add:expr, $symbol:ident) => {{
        let probs_base: *mut probability = $probs_base;
        core::arch::asm!(
            "movzwl 2({probs_base}), {prob0:e}",
            "mov $2, {symbol:e}",
            "movzwl 4({probs_base}), {prob1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 6({probs_base}), {t0:e}",
            "cmovae {t0:e}, {prob1:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob0:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob0:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob1:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob0:e}",
            "lea {bit_model_offset}({prob1:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob1:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob1:e}",
            "mov {prob1:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob1:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob0:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob0:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob1:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob0:e}",
            "lea {bit_model_offset}({prob1:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob1:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob1:e}",
            "mov {prob1:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob1:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob0:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob0:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob1:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob0:e}",
            "lea {bit_model_offset}({prob1:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob1:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob1:e}",
            "mov {prob1:x}, ({probs_base}, {t1:r}, 1)",
            "movzwl ({probs_base}, {symbol:r}, 4), {prob1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 2({probs_base}, {symbol:r}, 4), {t0:e}",
            "lea ({symbol:r}, {symbol:r}), {symbol:e}",
            "cmovae {t0:e}, {prob1:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob0:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, ({probs_base}, {t1:r}, 1)",
            "add {symbol:e}, {symbol:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob1:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob1:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob1:e}, {t0:e}",
            "sbb ${last_sbb}, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob1:e}",
            "mov {prob1:x}, ({probs_base}, {t1:r}, 1)",
            range = inout(reg) $rc.range,
            code = inout(reg) $rc.code,
            t0 = out(reg) _,
            t1 = out(reg) _,
            prob0 = out(reg) _,
            prob1 = out(reg) _,
            symbol = out(reg) $symbol,
            in_ptr = inout(reg) $rc_in_ptr,
            probs_base = in(reg) probs_base,
            last_sbb = const -1i32 - ($final_add),
            top_value = const RC_TOP_VALUE,
            shift_bits = const RC_SHIFT_BITS,
            bit_model_total_bits = const RC_BIT_MODEL_TOTAL_BITS,
            bit_model_offset = const RC_BIT_MODEL_OFFSET,
            move_bits = const RC_MOVE_BITS,
            options(att_syntax),
        );
    }};
}

#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "64")))]
macro_rules! rc_bittree_rev4_step {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:ident, $add:expr, $symbol:ident) => {{
        rc_normalize!($rc, $rc_in_ptr);
        let prob = $probs_base.add($symbol.wrapping_add($add) as usize);
        $rc_bound = ($rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*prob as u32);
        if $rc.code < $rc_bound {
            $rc.range = $rc_bound;
            prob_update_0(prob);
        } else {
            $rc.range = $rc.range.wrapping_sub($rc_bound);
            $rc.code = $rc.code.wrapping_sub($rc_bound);
            prob_update_1(prob);
            $symbol = $symbol.wrapping_add($add);
        }
    }};
}

#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "64")))]
macro_rules! rc_bittree_rev4 {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:expr, $symbol:ident) => {{
        let probs_base = $probs_base;
        $symbol = 0;
        rc_bittree_rev4_step!($rc, $rc_in_ptr, $rc_bound, probs_base, 1, $symbol);
        rc_bittree_rev4_step!($rc, $rc_in_ptr, $rc_bound, probs_base, 2, $symbol);
        rc_bittree_rev4_step!($rc, $rc_in_ptr, $rc_bound, probs_base, 4, $symbol);
        rc_bittree_rev4_step!($rc, $rc_in_ptr, $rc_bound, probs_base, 8, $symbol);
    }};
}

#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
macro_rules! rc_bittree_rev4 {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:expr, $symbol:ident) => {{
        let probs_base: *mut probability = $probs_base;
        core::arch::asm!(
            "movzwl 2({probs_base}), {prob0:e}",
            "xor {symbol:e}, {symbol:e}",
            "movzwl 4({probs_base}), {prob1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 6({probs_base}), {t0:e}",
            "cmovae {t0:e}, {prob1:e}",
            "lea 1({symbol:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "cmovae {t0:e}, {symbol:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovae {prob0:e}, {t0:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, 2({probs_base})",
            "movzwl 8({probs_base}, {symbol:r}, 2), {prob0:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob1:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 12({probs_base}, {symbol:r}, 2), {t0:e}",
            "cmovae {t0:e}, {prob0:e}",
            "lea 2({symbol:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {t0:e}, {symbol:e}",
            "lea {bit_model_offset}({prob1:r}), {t0:e}",
            "cmovae {prob1:e}, {t0:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob1:e}",
            "mov {prob1:x}, 4({probs_base}, {t1:r}, 2)",
            "movzwl 16({probs_base}, {symbol:r}, 2), {prob1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob0:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "movzwl 24({probs_base}, {symbol:r}, 2), {t0:e}",
            "cmovae {t0:e}, {prob1:e}",
            "lea 4({symbol:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {t0:e}, {symbol:e}",
            "lea {bit_model_offset}({prob0:r}), {t0:e}",
            "cmovae {prob0:e}, {t0:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob0:e}",
            "mov {prob0:x}, 8({probs_base}, {t1:r}, 2)",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob1:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea 8({symbol:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {t0:e}, {symbol:e}",
            "lea {bit_model_offset}({prob1:r}), {t0:e}",
            "cmovae {prob1:e}, {t0:e}",
            "shr ${move_bits}, {t0:e}",
            "sub {t0:e}, {prob1:e}",
            "mov {prob1:x}, 16({probs_base}, {t1:r}, 2)",
            range = inout(reg) $rc.range,
            code = inout(reg) $rc.code,
            t0 = out(reg) _,
            t1 = out(reg) _,
            prob0 = out(reg) _,
            prob1 = out(reg) _,
            symbol = out(reg) $symbol,
            in_ptr = inout(reg) $rc_in_ptr,
            probs_base = in(reg) probs_base,
            top_value = const RC_TOP_VALUE,
            shift_bits = const RC_SHIFT_BITS,
            bit_model_total_bits = const RC_BIT_MODEL_TOTAL_BITS,
            bit_model_offset = const RC_BIT_MODEL_OFFSET,
            move_bits = const RC_MOVE_BITS,
            options(att_syntax),
        );
    }};
}

#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
macro_rules! rc_matched_literal {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $probs_base:expr, $match_byte:expr, $symbol:ident) => {{
        let probs_base: *mut probability = $probs_base;
        // match_bit and match_byte are separate registers that both start
        // out as match_byte << 1, like t_match_bit and t_match_byte in
        // range_decoder.h. Keep them in distinct locals so neither operand
        // can be mistaken for an alias of the other.
        let t_match_byte: u32 = (($match_byte) as u32) << 1;
        let t_match_bit: u32 = t_match_byte;
        $symbol = 1;
        core::arch::asm!(
            "add {offset:e}, {symbol:e}",
            "and {offset:e}, {match_bit:e}",
            "add {match_bit:e}, {symbol:e}",
            "movzwl ({probs_base}, {symbol:r}, 2), {prob:e}",
            "add {symbol:e}, {symbol:e}",
            "xor {match_bit:e}, {offset:e}",
            "add {match_byte:e}, {match_byte:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob:e}, {t0:e}",
            "cmovae {match_bit:e}, {offset:e}",
            "mov {match_byte:e}, {match_bit:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "and $0x1FF, {symbol:e}",
            "sub {t0:e}, {prob:e}",
            "mov {prob:x}, ({probs_base}, {t1:r}, 1)",
            "add {offset:e}, {symbol:e}",
            "and {offset:e}, {match_bit:e}",
            "add {match_bit:e}, {symbol:e}",
            "movzwl ({probs_base}, {symbol:r}, 2), {prob:e}",
            "add {symbol:e}, {symbol:e}",
            "xor {match_bit:e}, {offset:e}",
            "add {match_byte:e}, {match_byte:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob:e}, {t0:e}",
            "cmovae {match_bit:e}, {offset:e}",
            "mov {match_byte:e}, {match_bit:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "and $0x1FF, {symbol:e}",
            "sub {t0:e}, {prob:e}",
            "mov {prob:x}, ({probs_base}, {t1:r}, 1)",
            "add {offset:e}, {symbol:e}",
            "and {offset:e}, {match_bit:e}",
            "add {match_bit:e}, {symbol:e}",
            "movzwl ({probs_base}, {symbol:r}, 2), {prob:e}",
            "add {symbol:e}, {symbol:e}",
            "xor {match_bit:e}, {offset:e}",
            "add {match_byte:e}, {match_byte:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob:e}, {t0:e}",
            "cmovae {match_bit:e}, {offset:e}",
            "mov {match_byte:e}, {match_bit:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "and $0x1FF, {symbol:e}",
            "sub {t0:e}, {prob:e}",
            "mov {prob:x}, ({probs_base}, {t1:r}, 1)",
            "add {offset:e}, {symbol:e}",
            "and {offset:e}, {match_bit:e}",
            "add {match_bit:e}, {symbol:e}",
            "movzwl ({probs_base}, {symbol:r}, 2), {prob:e}",
            "add {symbol:e}, {symbol:e}",
            "xor {match_bit:e}, {offset:e}",
            "add {match_byte:e}, {match_byte:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob:e}, {t0:e}",
            "cmovae {match_bit:e}, {offset:e}",
            "mov {match_byte:e}, {match_bit:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "and $0x1FF, {symbol:e}",
            "sub {t0:e}, {prob:e}",
            "mov {prob:x}, ({probs_base}, {t1:r}, 1)",
            "add {offset:e}, {symbol:e}",
            "and {offset:e}, {match_bit:e}",
            "add {match_bit:e}, {symbol:e}",
            "movzwl ({probs_base}, {symbol:r}, 2), {prob:e}",
            "add {symbol:e}, {symbol:e}",
            "xor {match_bit:e}, {offset:e}",
            "add {match_byte:e}, {match_byte:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob:e}, {t0:e}",
            "cmovae {match_bit:e}, {offset:e}",
            "mov {match_byte:e}, {match_bit:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "and $0x1FF, {symbol:e}",
            "sub {t0:e}, {prob:e}",
            "mov {prob:x}, ({probs_base}, {t1:r}, 1)",
            "add {offset:e}, {symbol:e}",
            "and {offset:e}, {match_bit:e}",
            "add {match_bit:e}, {symbol:e}",
            "movzwl ({probs_base}, {symbol:r}, 2), {prob:e}",
            "add {symbol:e}, {symbol:e}",
            "xor {match_bit:e}, {offset:e}",
            "add {match_byte:e}, {match_byte:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob:e}, {t0:e}",
            "cmovae {match_bit:e}, {offset:e}",
            "mov {match_byte:e}, {match_bit:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "and $0x1FF, {symbol:e}",
            "sub {t0:e}, {prob:e}",
            "mov {prob:x}, ({probs_base}, {t1:r}, 1)",
            "add {offset:e}, {symbol:e}",
            "and {offset:e}, {match_bit:e}",
            "add {match_bit:e}, {symbol:e}",
            "movzwl ({probs_base}, {symbol:r}, 2), {prob:e}",
            "add {symbol:e}, {symbol:e}",
            "xor {match_bit:e}, {offset:e}",
            "add {match_byte:e}, {match_byte:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob:e}, {t0:e}",
            "cmovae {match_bit:e}, {offset:e}",
            "mov {match_byte:e}, {match_bit:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "and $0x1FF, {symbol:e}",
            "sub {t0:e}, {prob:e}",
            "mov {prob:x}, ({probs_base}, {t1:r}, 1)",
            "add {offset:e}, {symbol:e}",
            "and {offset:e}, {match_bit:e}",
            "add {match_bit:e}, {symbol:e}",
            "movzwl ({probs_base}, {symbol:r}, 2), {prob:e}",
            "add {symbol:e}, {symbol:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "mov {range:e}, {t0:e}",
            "shr ${bit_model_total_bits}, {range:e}",
            "imul {prob:e}, {range:e}",
            "sub {range:e}, {t0:e}",
            "mov {code:e}, {t1:e}",
            "sub {range:e}, {code:e}",
            "cmovae {t0:e}, {range:e}",
            "lea {bit_model_offset}({prob:r}), {t0:e}",
            "cmovb {t1:e}, {code:e}",
            "mov {symbol:e}, {t1:e}",
            "cmovae {prob:e}, {t0:e}",
            "sbb $-1, {symbol:e}",
            "shr ${move_bits}, {t0:e}",
            "and $0x1FF, {symbol:e}",
            "sub {t0:e}, {prob:e}",
            "mov {prob:x}, ({probs_base}, {t1:r}, 1)",
            range = inout(reg) $rc.range,
            code = inout(reg) $rc.code,
            t0 = out(reg) _,
            t1 = out(reg) _,
            prob = out(reg) _,
            match_bit = inout(reg) t_match_bit => _,
            symbol = inout(reg) $symbol,
            match_byte = inout(reg) t_match_byte => _,
            offset = inout(reg) 0x100u32 => _,
            in_ptr = inout(reg) $rc_in_ptr,
            probs_base = in(reg) probs_base,
            top_value = const RC_TOP_VALUE,
            shift_bits = const RC_SHIFT_BITS,
            bit_model_total_bits = const RC_BIT_MODEL_TOTAL_BITS,
            bit_model_offset = const RC_BIT_MODEL_OFFSET,
            move_bits = const RC_MOVE_BITS,
            options(att_syntax),
        );
    }};
}

#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "64")))]
macro_rules! rc_direct {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $dest:ident, $count:ident) => {{
        loop {
            $dest = ($dest << 1).wrapping_add(1);
            rc_normalize!($rc, $rc_in_ptr);
            $rc.range >>= 1;
            $rc.code = $rc.code.wrapping_sub($rc.range);
            $rc_bound = 0u32.wrapping_sub($rc.code >> 31);
            $dest = $dest.wrapping_add($rc_bound);
            $rc.code = $rc.code.wrapping_add($rc.range & $rc_bound);
            $count -= 1;
            if $count == 0 {
                break;
            }
        }
    }};
}

#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
macro_rules! rc_direct {
    ($rc:ident, $rc_in_ptr:ident, $rc_bound:ident, $dest:ident, $count:ident) => {{
        core::arch::asm!(
            "9:",
            "add {dest:e}, {dest:e}",
            "lea 1({dest:r}), {t1:e}",
            "cmp ${top_value}, {range:e}",
            "jae 8f",
            "shl ${shift_bits}, {code:e}",
            "mov ({in_ptr}), {code:l}",
            "shl ${shift_bits}, {range:e}",
            "inc {in_ptr}",
            "8:",
            "shr $1, {range:e}",
            "mov {code:e}, {t0:e}",
            "sub {range:e}, {code:e}",
            "cmovns {t1:e}, {dest:e}",
            "cmovs {t0:e}, {code:e}",
            "dec {count:e}",
            "jnz 9b",
            range = inout(reg) $rc.range,
            code = inout(reg) $rc.code,
            t0 = out(reg) _,
            t1 = out(reg) _,
            dest = inout(reg) $dest,
            count = inout(reg) $count,
            in_ptr = inout(reg) $rc_in_ptr,
            top_value = const RC_TOP_VALUE,
            shift_bits = const RC_SHIFT_BITS,
            options(att_syntax),
        );
    }};
}
unsafe fn lzma_decode(
    coder_ptr: *mut c_void,
    dictptr: *mut lzma_dict,
    input: *const u8,
    in_pos: *mut size_t,
    in_size: size_t,
) -> lzma_ret {
    let mut block_state: DecoderBlockState;
    let coder: *mut lzma_lzma1_decoder = coder_ptr as *mut lzma_lzma1_decoder;
    let init_ret: lzma_ret = rc_read_init(
        ::core::ptr::addr_of_mut!((*coder).rc),
        input,
        in_pos,
        in_size,
    );
    if init_ret != LZMA_STREAM_END {
        return init_ret;
    }
    let mut dict: lzma_dict = *dictptr;
    let dict_start: size_t = dict.pos;
    let mut rc: lzma_range_decoder = (*coder).rc;
    let mut rc_in_ptr: *const u8 = input.add(*in_pos);
    let rc_in_end: *const u8 = input.add(in_size);
    // LZMA_IN_REQUIRED from lzma_decoder.c. Decoding one symbol uses at most
    // 22 probability bits and 26 direct bits, which the range coder normalizes
    // out of no more than 20 input bytes. The fast path below reads and
    // advances rc_in_ptr without ever comparing it against rc_in_end, so the
    // soundness of every unsafe read there rests on this reservation alone.
    // The bound is taken from the C source as-is and is not re-derived here.
    let rc_in_fast_end: *const u8 = if rc_in_end.offset_from(rc_in_ptr) <= 20 {
        rc_in_ptr
    } else {
        rc_in_end.offset(-20)
    };
    let mut rc_bound: u32 = 0;
    let mut state: u32 = (*coder).state as u32;
    let mut rep0: u32 = (*coder).rep0;
    let mut rep1: u32 = (*coder).rep1;
    let mut rep2: u32 = (*coder).rep2;
    let mut rep3: u32 = (*coder).rep3;
    let pos_mask: u32 = (*coder).pos_mask;
    let mut probs: *mut probability = (*coder).probs;
    let mut symbol: u32 = (*coder).symbol;
    let mut limit: u32 = (*coder).limit;
    let mut offset: u32 = (*coder).offset;
    let mut len: u32 = (*coder).len;
    let literal_probs: *mut probability =
        ::core::ptr::addr_of_mut!((*coder).literal) as *mut probability;
    let match_len_decoder: *mut lzma_length_decoder =
        ::core::ptr::addr_of_mut!((*coder).match_len_decoder);
    let rep_len_decoder: *mut lzma_length_decoder =
        ::core::ptr::addr_of_mut!((*coder).rep_len_decoder);
    let literal_mask: u32 = (*coder).literal_mask;
    let literal_context_bits: u32 = (*coder).literal_context_bits;
    let mut pos_state: u32 = (dict.pos & pos_mask as size_t) as u32;
    let mut ret: lzma_ret = LZMA_OK;
    let mut eopm_is_valid: bool = (*coder).uncompressed_size == LZMA_VLI_UNKNOWN;
    let mut might_finish_without_eopm: bool = false;
    if (*coder).uncompressed_size != LZMA_VLI_UNKNOWN
        && (*coder).uncompressed_size <= dict.limit.wrapping_sub(dict.pos) as lzma_vli
    {
        dict.limit = dict.pos.wrapping_add((*coder).uncompressed_size as size_t);
        might_finish_without_eopm = true;
    }
    block_state = resume_block_for_sequence((*coder).sequence);
    'c_9380: loop {
        match block_state {
            BLOCK_RETURN => {
                (*dictptr).pos = dict.pos;
                break;
            }
            BLOCK_REP_LEN_CHOICE => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_REP_LEN_CHOICE;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                    .wrapping_mul((*coder).rep_len_decoder.choice as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    (*coder).rep_len_decoder.choice = ((*coder).rep_len_decoder.choice as u32)
                        .wrapping_add(
                            RC_BIT_MODEL_TOTAL.wrapping_sub((*coder).rep_len_decoder.choice as u32)
                                >> RC_MOVE_BITS,
                        ) as probability;
                    probs = ::core::ptr::addr_of_mut!(
                        *(::core::ptr::addr_of_mut!((*coder).rep_len_decoder.low)
                            as *mut [probability; 8])
                            .offset(pos_state as isize)
                    ) as *mut probability;
                    limit = LEN_LOW_SYMBOLS;
                    len = MATCH_LEN_MIN;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    (*coder).rep_len_decoder.choice = (*coder).rep_len_decoder.choice
                        - ((*coder).rep_len_decoder.choice >> RC_MOVE_BITS);
                    block_state = BLOCK_REP_LEN_CHOICE2;
                    continue;
                }
                block_state = BLOCK_LEN_BITTREE_INIT;
            }
            BLOCK_IS_REP2 => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_IS_REP2;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                let is_rep2_prob = decoder_is_rep2_prob(coder, state);
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*is_rep2_prob as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *is_rep2_prob = (*is_rep2_prob as u32).wrapping_add(
                        RC_BIT_MODEL_TOTAL.wrapping_sub(*is_rep2_prob as u32) >> RC_MOVE_BITS,
                    ) as probability;
                    let distance_3: u32 = rep2;
                    rep2 = rep1;
                    rep1 = rep0;
                    rep0 = distance_3;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *is_rep2_prob = *is_rep2_prob - (*is_rep2_prob >> RC_MOVE_BITS);
                    let distance_4: u32 = rep3;
                    rep3 = rep2;
                    rep2 = rep1;
                    rep1 = rep0;
                    rep0 = distance_4;
                }
                block_state = BLOCK_REP_LEN_PREPARE;
            }
            BLOCK_IS_REP1 => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_IS_REP1;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                let is_rep1_prob = decoder_is_rep1_prob(coder, state);
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*is_rep1_prob as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *is_rep1_prob = (*is_rep1_prob as u32).wrapping_add(
                        RC_BIT_MODEL_TOTAL.wrapping_sub(*is_rep1_prob as u32) >> RC_MOVE_BITS,
                    ) as probability;
                    let distance_2: u32 = rep1;
                    rep1 = rep0;
                    rep0 = distance_2;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *is_rep1_prob = *is_rep1_prob - (*is_rep1_prob >> RC_MOVE_BITS);
                    block_state = BLOCK_IS_REP2;
                    continue;
                }
                block_state = BLOCK_REP_LEN_PREPARE;
            }
            BLOCK_SHORTREP => {
                if dict_put_safe(
                    ::core::ptr::addr_of_mut!(dict),
                    dict_get(::core::ptr::addr_of_mut!(dict), rep0),
                ) {
                    (*coder).sequence = SEQ_SHORTREP;
                    block_state = BLOCK_RETURN;
                    continue;
                } else {
                    block_state = BLOCK_MAIN_LOOP;
                }
            }
            BLOCK_IS_REP0_LONG => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_IS_REP0_LONG;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                let is_rep0_long_prob =
                    decoder_is_rep0_long_row(coder, state).add(pos_state as usize);
                rc_bound =
                    (rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*is_rep0_long_prob as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *is_rep0_long_prob = (*is_rep0_long_prob as u32).wrapping_add(
                        RC_BIT_MODEL_TOTAL.wrapping_sub(*is_rep0_long_prob as u32) >> RC_MOVE_BITS,
                    ) as probability;
                    state = (if state < LIT_STATES {
                        STATE_LIT_SHORTREP
                    } else {
                        STATE_NONLIT_REP
                    }) as u32;
                    block_state = BLOCK_SHORTREP;
                    continue;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *is_rep0_long_prob = *is_rep0_long_prob - (*is_rep0_long_prob >> RC_MOVE_BITS);
                }
                block_state = BLOCK_REP_LEN_PREPARE;
            }
            BLOCK_IS_REP0 => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_IS_REP0;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                let is_rep0_prob = decoder_is_rep0_prob(coder, state);
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*is_rep0_prob as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *is_rep0_prob = (*is_rep0_prob as u32).wrapping_add(
                        RC_BIT_MODEL_TOTAL.wrapping_sub(*is_rep0_prob as u32) >> RC_MOVE_BITS,
                    ) as probability;
                    block_state = BLOCK_IS_REP0_LONG;
                    continue;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *is_rep0_prob = *is_rep0_prob - (*is_rep0_prob >> RC_MOVE_BITS);
                    block_state = BLOCK_IS_REP1;
                    continue;
                }
            }
            BLOCK_IS_REP => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_IS_REP;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                let is_rep_prob = decoder_is_rep_prob(coder, state);
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*is_rep_prob as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *is_rep_prob = (*is_rep_prob as u32).wrapping_add(
                        RC_BIT_MODEL_TOTAL.wrapping_sub(*is_rep_prob as u32) >> RC_MOVE_BITS,
                    ) as probability;
                    state = (if state < LIT_STATES {
                        STATE_LIT_MATCH
                    } else {
                        STATE_NONLIT_MATCH
                    }) as u32;
                    rep3 = rep2;
                    rep2 = rep1;
                    rep1 = rep0;
                    block_state = BLOCK_MATCH_LEN_CHOICE;
                    continue;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *is_rep_prob = *is_rep_prob - (*is_rep_prob >> RC_MOVE_BITS);
                    if dict_is_distance_valid(::core::ptr::addr_of_mut!(dict), 0) {
                        block_state = BLOCK_IS_REP0;
                        continue;
                    }
                    ret = LZMA_DATA_ERROR;
                    block_state = BLOCK_RETURN;
                    continue;
                }
            }
            BLOCK_EOPM => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_EOPM;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                ret = if rc.code == 0 {
                    LZMA_STREAM_END
                } else {
                    LZMA_DATA_ERROR
                };
                block_state = BLOCK_RETURN;
                continue;
            }
            BLOCK_ALIGN => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_ALIGN;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                let pos_align_prob = decoder_pos_align_prob(coder, offset.wrapping_add(symbol));
                rc_bound =
                    (rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*pos_align_prob as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *pos_align_prob = (*pos_align_prob as u32).wrapping_add(
                        RC_BIT_MODEL_TOTAL.wrapping_sub(*pos_align_prob as u32) >> RC_MOVE_BITS,
                    ) as probability;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *pos_align_prob = *pos_align_prob - (*pos_align_prob >> RC_MOVE_BITS);
                    symbol = symbol.wrapping_add(offset);
                }
                offset <<= 1;
                if offset < ALIGN_SIZE {
                    block_state = BLOCK_ALIGN;
                    continue;
                }
                rep0 = rep0.wrapping_add(symbol);
                if rep0 == UINT32_MAX {
                    block_state = BLOCK_EOPM_IF_VALID;
                } else {
                    block_state = BLOCK_VALIDATE_DISTANCE;
                }
            }
            BLOCK_DIRECT => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_DIRECT;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc.range >>= 1;
                rc.code = rc.code.wrapping_sub(rc.range);
                rc_bound = 0u32.wrapping_sub(rc.code >> 31);
                rc.code = rc.code.wrapping_add(rc.range & rc_bound);
                rep0 = (rep0 << 1).wrapping_add(rc_bound.wrapping_add(1));
                limit -= 1;
                if limit > 0 {
                    block_state = BLOCK_DIRECT;
                    continue;
                }
                rep0 <<= ALIGN_BITS;
                symbol = 0;
                offset = 1;
                block_state = BLOCK_ALIGN;
                continue;
            }
            BLOCK_DIST_MODEL => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_DIST_MODEL;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                    .wrapping_mul(*probs.offset(symbol as isize) as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *probs.offset(symbol as isize) = (*probs.offset(symbol as isize) as u32)
                        .wrapping_add(
                            RC_BIT_MODEL_TOTAL.wrapping_sub(*probs.offset(symbol as isize) as u32)
                                >> RC_MOVE_BITS,
                        ) as probability;
                    symbol <<= 1;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *probs.offset(symbol as isize) -=
                        *probs.offset(symbol as isize) >> RC_MOVE_BITS;
                    symbol = (symbol << 1).wrapping_add(1);
                    rep0 = (rep0 as u32).wrapping_add(1u32 << offset) as u32;
                }
                offset += 1;
                if offset < limit {
                    block_state = BLOCK_DIST_MODEL;
                    continue;
                }
                block_state = BLOCK_VALIDATE_DISTANCE;
            }
            BLOCK_DIST_SLOT => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_DIST_SLOT;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                    .wrapping_mul(*probs.offset(symbol as isize) as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *probs.offset(symbol as isize) = (*probs.offset(symbol as isize) as u32)
                        .wrapping_add(
                            RC_BIT_MODEL_TOTAL.wrapping_sub(*probs.offset(symbol as isize) as u32)
                                >> RC_MOVE_BITS,
                        ) as probability;
                    symbol <<= 1;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *probs.offset(symbol as isize) -=
                        *probs.offset(symbol as isize) >> RC_MOVE_BITS;
                    symbol = (symbol << 1).wrapping_add(1);
                }
                if symbol < DIST_SLOTS {
                    block_state = BLOCK_DIST_SLOT;
                    continue;
                }
                symbol = symbol.wrapping_sub(DIST_SLOTS);
                if symbol < DIST_MODEL_START {
                    rep0 = symbol;
                } else {
                    limit = (symbol >> 1).wrapping_sub(1);
                    rep0 = (2u32).wrapping_add(symbol & 1);
                    if symbol < DIST_MODEL_END {
                        rep0 <<= limit;
                        probs = (::core::ptr::addr_of_mut!((*coder).pos_special)
                            as *mut probability)
                            .offset(rep0 as isize)
                            .offset(-(symbol as isize))
                            .offset(-1);
                        symbol = 1;
                        offset = 0;
                        block_state = BLOCK_DIST_MODEL;
                        continue;
                    } else {
                        limit = limit.wrapping_sub(ALIGN_BITS);
                        block_state = BLOCK_DIRECT;
                        continue;
                    }
                }
                block_state = BLOCK_VALIDATE_DISTANCE;
            }
            BLOCK_MATCH_LEN_BITTREE => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_MATCH_LEN_BITTREE;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                    .wrapping_mul(*probs.offset(symbol as isize) as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *probs.offset(symbol as isize) = (*probs.offset(symbol as isize) as u32)
                        .wrapping_add(
                            RC_BIT_MODEL_TOTAL.wrapping_sub(*probs.offset(symbol as isize) as u32)
                                >> RC_MOVE_BITS,
                        ) as probability;
                    symbol <<= 1;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *probs.offset(symbol as isize) -=
                        *probs.offset(symbol as isize) >> RC_MOVE_BITS;
                    symbol = (symbol << 1).wrapping_add(1);
                }
                if symbol < limit {
                    block_state = BLOCK_MATCH_LEN_BITTREE;
                    continue;
                }
                len = len.wrapping_add(symbol.wrapping_sub(limit));
                probs = ::core::ptr::addr_of_mut!(
                    *(::core::ptr::addr_of_mut!((*coder).dist_slot) as *mut [probability; 64])
                        .offset(
                            (if len < (DIST_STATES + MATCH_LEN_MIN) as u32 {
                                len.wrapping_sub(MATCH_LEN_MIN)
                            } else {
                                (DIST_STATES - 1) as u32
                            }) as isize,
                        )
                ) as *mut probability;
                symbol = 1;
                block_state = BLOCK_DIST_SLOT;
                continue;
            }
            BLOCK_MATCH_LEN_CHOICE2 => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_MATCH_LEN_CHOICE2;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                    .wrapping_mul((*coder).match_len_decoder.choice2 as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    (*coder).match_len_decoder.choice2 = ((*coder).match_len_decoder.choice2 as u32)
                        .wrapping_add(
                            RC_BIT_MODEL_TOTAL
                                .wrapping_sub((*coder).match_len_decoder.choice2 as u32)
                                >> RC_MOVE_BITS,
                        ) as probability;
                    probs = ::core::ptr::addr_of_mut!(
                        *(::core::ptr::addr_of_mut!((*coder).match_len_decoder.mid)
                            as *mut [probability; 8])
                            .offset(pos_state as isize)
                    ) as *mut probability;
                    limit = LEN_MID_SYMBOLS;
                    len = (MATCH_LEN_MIN + LEN_LOW_SYMBOLS) as u32;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    (*coder).match_len_decoder.choice2 = (*coder).match_len_decoder.choice2
                        - ((*coder).match_len_decoder.choice2 >> RC_MOVE_BITS);
                    probs = ::core::ptr::addr_of_mut!((*coder).match_len_decoder.high)
                        as *mut probability;
                    limit = LEN_HIGH_SYMBOLS;
                    len = (MATCH_LEN_MIN + LEN_LOW_SYMBOLS + LEN_MID_SYMBOLS) as u32;
                }
                block_state = BLOCK_DIST_SLOT_INIT;
            }
            BLOCK_MATCH_LEN_CHOICE => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_MATCH_LEN_CHOICE;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                    .wrapping_mul((*coder).match_len_decoder.choice as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    (*coder).match_len_decoder.choice = ((*coder).match_len_decoder.choice as u32)
                        .wrapping_add(
                            RC_BIT_MODEL_TOTAL
                                .wrapping_sub((*coder).match_len_decoder.choice as u32)
                                >> RC_MOVE_BITS,
                        ) as probability;
                    probs = ::core::ptr::addr_of_mut!(
                        *(::core::ptr::addr_of_mut!((*coder).match_len_decoder.low)
                            as *mut [probability; 8])
                            .offset(pos_state as isize)
                    ) as *mut probability;
                    limit = LEN_LOW_SYMBOLS;
                    len = MATCH_LEN_MIN;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    (*coder).match_len_decoder.choice = (*coder).match_len_decoder.choice
                        - ((*coder).match_len_decoder.choice >> RC_MOVE_BITS);
                    block_state = BLOCK_MATCH_LEN_CHOICE2;
                    continue;
                }
                block_state = BLOCK_DIST_SLOT_INIT;
            }
            BLOCK_LITERAL_WRITE => {
                if dict_put_safe(::core::ptr::addr_of_mut!(dict), symbol as u8) {
                    (*coder).sequence = SEQ_LITERAL_WRITE;
                    block_state = BLOCK_RETURN;
                    continue;
                } else {
                    block_state = BLOCK_MAIN_LOOP;
                }
            }
            BLOCK_LITERAL_MATCHED => {
                let match_bit: u32 = len & offset;
                let subcoder_index: u32 = offset.wrapping_add(match_bit).wrapping_add(symbol);
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_LITERAL_MATCHED;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                    .wrapping_mul(*probs.offset(subcoder_index as isize) as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *probs.offset(subcoder_index as isize) =
                        (*probs.offset(subcoder_index as isize) as u32).wrapping_add(
                            RC_BIT_MODEL_TOTAL
                                .wrapping_sub(*probs.offset(subcoder_index as isize) as u32)
                                >> RC_MOVE_BITS,
                        ) as probability;
                    symbol <<= 1;
                    offset &= !match_bit;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *probs.offset(subcoder_index as isize) -=
                        *probs.offset(subcoder_index as isize) >> RC_MOVE_BITS;
                    symbol = (symbol << 1).wrapping_add(1);
                    offset &= match_bit;
                }
                len <<= 1;
                if symbol < (1 << 8) as u32 {
                    block_state = BLOCK_LITERAL_MATCHED;
                    continue;
                } else {
                    block_state = BLOCK_LITERAL_WRITE;
                    continue;
                }
            }
            BLOCK_NORMALIZE_OR_IS_MATCH => {
                if might_finish_without_eopm && dict.pos == dict.limit {
                    if rc.range < RC_TOP_VALUE as u32 {
                        if rc_in_ptr == rc_in_end {
                            (*coder).sequence = SEQ_NORMALIZE;
                            block_state = BLOCK_RETURN;
                            continue;
                        } else {
                            rc.range <<= RC_SHIFT_BITS;
                            rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                            rc_in_ptr = rc_in_ptr.offset(1);
                        }
                    }
                    if rc.code == 0 {
                        ret = LZMA_STREAM_END;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else if !(*coder).allow_eopm {
                        ret = LZMA_DATA_ERROR;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        eopm_is_valid = true;
                    }
                }
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_IS_MATCH;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                let is_match_prob = decoder_is_match_row(coder, state).add(pos_state as usize);
                rc_bound =
                    (rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*is_match_prob as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *is_match_prob = (*is_match_prob as u32).wrapping_add(
                        RC_BIT_MODEL_TOTAL.wrapping_sub(*is_match_prob as u32) >> RC_MOVE_BITS,
                    ) as probability;
                    probs = literal_probs.add(
                        (3_usize).wrapping_mul(
                            ((dict.pos << 8)
                                .wrapping_add(dict_get0(::core::ptr::addr_of_mut!(dict)) as size_t)
                                & literal_mask as size_t)
                                << literal_context_bits,
                        ),
                    );
                    symbol = 1;
                    if state < LIT_STATES {
                        state = if state <= STATE_SHORTREP_LIT_LIT {
                            STATE_LIT_LIT
                        } else {
                            state.wrapping_sub(3)
                        };
                        block_state = BLOCK_LITERAL;
                        continue;
                    } else {
                        state = if state <= STATE_LIT_SHORTREP {
                            state.wrapping_sub(3)
                        } else {
                            state.wrapping_sub(6)
                        };
                        len = (dict_get(::core::ptr::addr_of_mut!(dict), rep0) as u32) << 1;
                        offset = 0x100;
                        block_state = BLOCK_LITERAL_MATCHED;
                        continue;
                    }
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *is_match_prob = *is_match_prob - (*is_match_prob >> RC_MOVE_BITS);
                    block_state = BLOCK_IS_REP;
                    continue;
                }
            }
            BLOCK_LITERAL => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_LITERAL;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                    .wrapping_mul(*probs.offset(symbol as isize) as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *probs.offset(symbol as isize) = (*probs.offset(symbol as isize) as u32)
                        .wrapping_add(
                            RC_BIT_MODEL_TOTAL.wrapping_sub(*probs.offset(symbol as isize) as u32)
                                >> RC_MOVE_BITS,
                        ) as probability;
                    symbol <<= 1;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *probs.offset(symbol as isize) -=
                        *probs.offset(symbol as isize) >> RC_MOVE_BITS;
                    symbol = (symbol << 1).wrapping_add(1);
                }
                if symbol < (1 << 8) as u32 {
                    block_state = BLOCK_LITERAL;
                    continue;
                } else {
                    block_state = BLOCK_LITERAL_WRITE;
                    continue;
                }
            }
            BLOCK_REP_LEN_BITTREE => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_REP_LEN_BITTREE;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                    .wrapping_mul(*probs.offset(symbol as isize) as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *probs.offset(symbol as isize) = (*probs.offset(symbol as isize) as u32)
                        .wrapping_add(
                            RC_BIT_MODEL_TOTAL.wrapping_sub(*probs.offset(symbol as isize) as u32)
                                >> RC_MOVE_BITS,
                        ) as probability;
                    symbol <<= 1;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *probs.offset(symbol as isize) -=
                        *probs.offset(symbol as isize) >> RC_MOVE_BITS;
                    symbol = (symbol << 1).wrapping_add(1);
                }
                if symbol < limit {
                    block_state = BLOCK_REP_LEN_BITTREE;
                    continue;
                }
                len = len.wrapping_add(symbol.wrapping_sub(limit));
                block_state = BLOCK_COPY;
                continue;
            }
            BLOCK_COPY => {
                if dict_repeat(
                    ::core::ptr::addr_of_mut!(dict),
                    rep0,
                    ::core::ptr::addr_of_mut!(len),
                ) {
                    (*coder).sequence = SEQ_COPY;
                    block_state = BLOCK_RETURN;
                    continue;
                } else {
                    block_state = BLOCK_MAIN_LOOP;
                }
            }
            _ => {
                if rc.range < RC_TOP_VALUE as u32 {
                    if rc_in_ptr == rc_in_end {
                        (*coder).sequence = SEQ_REP_LEN_CHOICE2;
                        block_state = BLOCK_RETURN;
                        continue;
                    } else {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                }
                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                    .wrapping_mul((*coder).rep_len_decoder.choice2 as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    (*coder).rep_len_decoder.choice2 = ((*coder).rep_len_decoder.choice2 as u32)
                        .wrapping_add(
                            RC_BIT_MODEL_TOTAL
                                .wrapping_sub((*coder).rep_len_decoder.choice2 as u32)
                                >> RC_MOVE_BITS,
                        ) as probability;
                    probs = ::core::ptr::addr_of_mut!(
                        *(::core::ptr::addr_of_mut!((*coder).rep_len_decoder.mid)
                            as *mut [probability; 8])
                            .offset(pos_state as isize)
                    ) as *mut probability;
                    limit = LEN_MID_SYMBOLS;
                    len = (MATCH_LEN_MIN + LEN_LOW_SYMBOLS) as u32;
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    (*coder).rep_len_decoder.choice2 = (*coder).rep_len_decoder.choice2
                        - ((*coder).rep_len_decoder.choice2 >> RC_MOVE_BITS);
                    probs = ::core::ptr::addr_of_mut!((*coder).rep_len_decoder.high)
                        as *mut probability;
                    limit = LEN_HIGH_SYMBOLS;
                    len = (MATCH_LEN_MIN + LEN_LOW_SYMBOLS + LEN_MID_SYMBOLS) as u32;
                }
                block_state = BLOCK_LEN_BITTREE_INIT;
            }
        }
        match block_state {
            BLOCK_VALIDATE_DISTANCE => {
                if dict_is_distance_valid(::core::ptr::addr_of_mut!(dict), rep0 as size_t) {
                    block_state = BLOCK_COPY;
                    continue;
                }
                ret = LZMA_DATA_ERROR;
                block_state = BLOCK_RETURN;
                continue;
            }
            BLOCK_MAIN_LOOP => loop {
                pos_state = (dict.pos & pos_mask as size_t) as u32;
                if rc_in_ptr >= rc_in_fast_end || dict.pos == dict.limit {
                    block_state = BLOCK_NORMALIZE_OR_IS_MATCH;
                    continue 'c_9380;
                }
                if rc.range < RC_TOP_VALUE as u32 {
                    rc.range <<= RC_SHIFT_BITS;
                    rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                    rc_in_ptr = rc_in_ptr.offset(1);
                }
                let is_match_prob = decoder_is_match_row(coder, state).add(pos_state as usize);
                rc_bound =
                    (rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*is_match_prob as u32);
                if rc.code < rc_bound {
                    rc.range = rc_bound;
                    *is_match_prob = (*is_match_prob as u32).wrapping_add(
                        RC_BIT_MODEL_TOTAL.wrapping_sub(*is_match_prob as u32) >> RC_MOVE_BITS,
                    ) as probability;
                    probs = literal_probs.add(
                        (3_usize).wrapping_mul(
                            ((dict.pos << 8)
                                .wrapping_add(dict_get0(::core::ptr::addr_of_mut!(dict)) as size_t)
                                & literal_mask as size_t)
                                << literal_context_bits,
                        ),
                    );
                    if state < LIT_STATES {
                        state = if state <= STATE_SHORTREP_LIT_LIT {
                            STATE_LIT_LIT
                        } else {
                            state.wrapping_sub(3)
                        };
                        rc_bittree8!(rc, rc_in_ptr, rc_bound, probs, 0, symbol);
                    } else {
                        state = if state <= STATE_LIT_SHORTREP {
                            state.wrapping_sub(3)
                        } else {
                            state.wrapping_sub(6)
                        };
                        rc_matched_literal!(
                            rc,
                            rc_in_ptr,
                            rc_bound,
                            probs,
                            dict_get(::core::ptr::addr_of_mut!(dict), rep0),
                            symbol
                        );
                    }
                    dict_put(::core::ptr::addr_of_mut!(dict), symbol as u8);
                } else {
                    rc.range = rc.range.wrapping_sub(rc_bound);
                    rc.code = rc.code.wrapping_sub(rc_bound);
                    *is_match_prob = *is_match_prob - (*is_match_prob >> RC_MOVE_BITS);
                    if rc.range < RC_TOP_VALUE as u32 {
                        rc.range <<= RC_SHIFT_BITS;
                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                        rc_in_ptr = rc_in_ptr.offset(1);
                    }
                    let is_rep_prob = decoder_is_rep_prob(coder, state);
                    rc_bound =
                        (rc.range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(*is_rep_prob as u32);
                    if rc.code < rc_bound {
                        rc.range = rc_bound;
                        *is_rep_prob = (*is_rep_prob as u32).wrapping_add(
                            RC_BIT_MODEL_TOTAL.wrapping_sub(*is_rep_prob as u32) >> RC_MOVE_BITS,
                        ) as probability;
                        state = (if state < LIT_STATES {
                            STATE_LIT_MATCH
                        } else {
                            STATE_NONLIT_MATCH
                        }) as u32;
                        rep3 = rep2;
                        rep2 = rep1;
                        rep1 = rep0;
                        symbol = 1;
                        rc_normalize!(rc, rc_in_ptr);
                        rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                            .wrapping_mul((*coder).match_len_decoder.choice as u32);
                        if rc.code < rc_bound {
                            rc.range = rc_bound;
                            prob_update_0(::core::ptr::addr_of_mut!(
                                (*coder).match_len_decoder.choice
                            ));
                            rc_bittree3!(
                                rc,
                                rc_in_ptr,
                                rc_bound,
                                length_low_row(match_len_decoder, pos_state),
                                -(1_i32 << 3) + 2,
                                symbol
                            );
                            len = symbol;
                        } else {
                            rc.range = rc.range.wrapping_sub(rc_bound);
                            rc.code = rc.code.wrapping_sub(rc_bound);
                            prob_update_1(::core::ptr::addr_of_mut!(
                                (*coder).match_len_decoder.choice
                            ));
                            rc_normalize!(rc, rc_in_ptr);
                            rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                                .wrapping_mul((*coder).match_len_decoder.choice2 as u32);
                            if rc.code < rc_bound {
                                rc.range = rc_bound;
                                prob_update_0(::core::ptr::addr_of_mut!(
                                    (*coder).match_len_decoder.choice2
                                ));
                                rc_bittree3!(
                                    rc,
                                    rc_in_ptr,
                                    rc_bound,
                                    length_mid_row(match_len_decoder, pos_state),
                                    -(1_i32 << 3) + 2 + (1 << 3),
                                    symbol
                                );
                                len = symbol;
                            } else {
                                rc.range = rc.range.wrapping_sub(rc_bound);
                                rc.code = rc.code.wrapping_sub(rc_bound);
                                prob_update_1(::core::ptr::addr_of_mut!(
                                    (*coder).match_len_decoder.choice2
                                ));
                                rc_bittree8!(
                                    rc,
                                    rc_in_ptr,
                                    rc_bound,
                                    length_high_probs(match_len_decoder),
                                    -(1_i32 << 8) + 2 + (1 << 3) + (1 << 3),
                                    symbol
                                );
                                len = symbol;
                            }
                        }
                        probs = ::core::ptr::addr_of_mut!(
                            *(::core::ptr::addr_of_mut!((*coder).dist_slot)
                                as *mut [probability; 64])
                                .offset(
                                    (if len < (DIST_STATES + MATCH_LEN_MIN) as u32 {
                                        len.wrapping_sub(MATCH_LEN_MIN)
                                    } else {
                                        (DIST_STATES - 1) as u32
                                    }) as isize,
                                )
                        ) as *mut probability;
                        rc_bittree6!(rc, rc_in_ptr, rc_bound, probs, -(1_i32 << 6), symbol);
                        if symbol < DIST_MODEL_START {
                            rep0 = symbol;
                        } else {
                            limit = (symbol >> 1).wrapping_sub(1);
                            rep0 = (2u32).wrapping_add(symbol & 1);
                            if symbol < DIST_MODEL_END {
                                rep0 <<= limit;
                                probs = (::core::ptr::addr_of_mut!((*coder).pos_special)
                                    as *mut probability)
                                    .offset(rep0 as isize)
                                    .offset(-(symbol as isize))
                                    .offset(-1);
                                symbol = 1;
                                offset = 1;
                                loop {
                                    if rc.range < RC_TOP_VALUE as u32 {
                                        rc.range <<= RC_SHIFT_BITS;
                                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                                        rc_in_ptr = rc_in_ptr.offset(1);
                                    }
                                    rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                                        .wrapping_mul(*probs.offset(symbol as isize) as u32);
                                    if rc.code < rc_bound {
                                        rc.range = rc_bound;
                                        *probs.offset(symbol as isize) =
                                            (*probs.offset(symbol as isize) as u32).wrapping_add(
                                                RC_BIT_MODEL_TOTAL.wrapping_sub(
                                                    *probs.offset(symbol as isize) as u32,
                                                ) >> RC_MOVE_BITS,
                                            )
                                                as probability;
                                        symbol <<= 1;
                                    } else {
                                        rc.range = rc.range.wrapping_sub(rc_bound);
                                        rc.code = rc.code.wrapping_sub(rc_bound);
                                        *probs.offset(symbol as isize) -=
                                            *probs.offset(symbol as isize) >> RC_MOVE_BITS;
                                        symbol = (symbol << 1).wrapping_add(1);
                                        rep0 = rep0.wrapping_add(offset);
                                    }
                                    offset <<= 1;
                                    limit -= 1;
                                    if limit == 0 {
                                        break;
                                    }
                                }
                            } else {
                                limit = limit.wrapping_sub(ALIGN_BITS);
                                rc_direct!(rc, rc_in_ptr, rc_bound, rep0, limit);
                                rep0 <<= ALIGN_BITS;
                                rc_bittree_rev4!(
                                    rc,
                                    rc_in_ptr,
                                    rc_bound,
                                    ::core::ptr::addr_of_mut!((*coder).pos_align)
                                        as *mut probability,
                                    symbol
                                );
                                rep0 = rep0.wrapping_add(symbol);
                                if rep0 == UINT32_MAX {
                                    break;
                                }
                            }
                        }
                        if !dict_is_distance_valid(::core::ptr::addr_of_mut!(dict), rep0 as size_t)
                        {
                            ret = LZMA_DATA_ERROR;
                            block_state = BLOCK_RETURN;
                            continue 'c_9380;
                        }
                    } else {
                        rc.range = rc.range.wrapping_sub(rc_bound);
                        rc.code = rc.code.wrapping_sub(rc_bound);
                        let is_rep_prob = decoder_is_rep_prob(coder, state);
                        *is_rep_prob = *is_rep_prob - (*is_rep_prob >> RC_MOVE_BITS);
                        if !dict_is_distance_valid(::core::ptr::addr_of_mut!(dict), 0) {
                            ret = LZMA_DATA_ERROR;
                            block_state = BLOCK_RETURN;
                            continue 'c_9380;
                        } else {
                            if rc.range < RC_TOP_VALUE as u32 {
                                rc.range <<= RC_SHIFT_BITS;
                                rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                                rc_in_ptr = rc_in_ptr.offset(1);
                            }
                            let is_rep0_prob = decoder_is_rep0_prob(coder, state);
                            rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                                .wrapping_mul(*is_rep0_prob as u32);
                            if rc.code < rc_bound {
                                rc.range = rc_bound;
                                *is_rep0_prob = (*is_rep0_prob as u32).wrapping_add(
                                    RC_BIT_MODEL_TOTAL.wrapping_sub(*is_rep0_prob as u32)
                                        >> RC_MOVE_BITS,
                                ) as probability;
                                if rc.range < RC_TOP_VALUE as u32 {
                                    rc.range <<= RC_SHIFT_BITS;
                                    rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                                    rc_in_ptr = rc_in_ptr.offset(1);
                                }
                                let is_rep0_long_prob =
                                    decoder_is_rep0_long_row(coder, state).add(pos_state as usize);
                                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                                    .wrapping_mul(*is_rep0_long_prob as u32);
                                if rc.code < rc_bound {
                                    rc.range = rc_bound;
                                    *is_rep0_long_prob = (*is_rep0_long_prob as u32).wrapping_add(
                                        RC_BIT_MODEL_TOTAL.wrapping_sub(*is_rep0_long_prob as u32)
                                            >> RC_MOVE_BITS,
                                    )
                                        as probability;
                                    state = (if state < LIT_STATES {
                                        STATE_LIT_SHORTREP
                                    } else {
                                        STATE_NONLIT_REP
                                    }) as u32;
                                    dict_put(
                                        ::core::ptr::addr_of_mut!(dict),
                                        dict_get(::core::ptr::addr_of_mut!(dict), rep0),
                                    );
                                    continue;
                                } else {
                                    rc.range = rc.range.wrapping_sub(rc_bound);
                                    rc.code = rc.code.wrapping_sub(rc_bound);
                                    *is_rep0_long_prob =
                                        *is_rep0_long_prob - (*is_rep0_long_prob >> RC_MOVE_BITS);
                                }
                            } else {
                                rc.range = rc.range.wrapping_sub(rc_bound);
                                rc.code = rc.code.wrapping_sub(rc_bound);
                                *is_rep0_prob = *is_rep0_prob - (*is_rep0_prob >> RC_MOVE_BITS);
                                if rc.range < RC_TOP_VALUE as u32 {
                                    rc.range <<= RC_SHIFT_BITS;
                                    rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                                    rc_in_ptr = rc_in_ptr.offset(1);
                                }
                                let is_rep1_prob = decoder_is_rep1_prob(coder, state);
                                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                                    .wrapping_mul(*is_rep1_prob as u32);
                                if rc.code < rc_bound {
                                    rc.range = rc_bound;
                                    *is_rep1_prob = (*is_rep1_prob as u32).wrapping_add(
                                        RC_BIT_MODEL_TOTAL.wrapping_sub(*is_rep1_prob as u32)
                                            >> RC_MOVE_BITS,
                                    )
                                        as probability;
                                    let distance: u32 = rep1;
                                    rep1 = rep0;
                                    rep0 = distance;
                                } else {
                                    rc.range = rc.range.wrapping_sub(rc_bound);
                                    rc.code = rc.code.wrapping_sub(rc_bound);
                                    *is_rep1_prob = *is_rep1_prob - (*is_rep1_prob >> RC_MOVE_BITS);
                                    if rc.range < RC_TOP_VALUE as u32 {
                                        rc.range <<= RC_SHIFT_BITS;
                                        rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                                        rc_in_ptr = rc_in_ptr.offset(1);
                                    }
                                    let is_rep2_prob = decoder_is_rep2_prob(coder, state);
                                    rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                                        .wrapping_mul(*is_rep2_prob as u32);
                                    if rc.code < rc_bound {
                                        rc.range = rc_bound;
                                        *is_rep2_prob = (*is_rep2_prob as u32).wrapping_add(
                                            RC_BIT_MODEL_TOTAL.wrapping_sub(*is_rep2_prob as u32)
                                                >> RC_MOVE_BITS,
                                        )
                                            as probability;
                                        let distance_0: u32 = rep2;
                                        rep2 = rep1;
                                        rep1 = rep0;
                                        rep0 = distance_0;
                                    } else {
                                        rc.range = rc.range.wrapping_sub(rc_bound);
                                        rc.code = rc.code.wrapping_sub(rc_bound);
                                        *is_rep2_prob =
                                            *is_rep2_prob - (*is_rep2_prob >> RC_MOVE_BITS);
                                        let distance_1: u32 = rep3;
                                        rep3 = rep2;
                                        rep2 = rep1;
                                        rep1 = rep0;
                                        rep0 = distance_1;
                                    }
                                }
                            }
                            state = (if state < LIT_STATES {
                                STATE_LIT_LONGREP
                            } else {
                                STATE_NONLIT_REP
                            }) as u32;
                            symbol = 1;
                            if rc.range < RC_TOP_VALUE as u32 {
                                rc.range <<= RC_SHIFT_BITS;
                                rc.code = rc.code << RC_SHIFT_BITS | *rc_in_ptr as u32;
                                rc_in_ptr = rc_in_ptr.offset(1);
                            }
                            rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                                .wrapping_mul((*rep_len_decoder).choice as u32);
                            if rc.code < rc_bound {
                                rc.range = rc_bound;
                                prob_update_0(::core::ptr::addr_of_mut!((*rep_len_decoder).choice));
                                rc_bittree3!(
                                    rc,
                                    rc_in_ptr,
                                    rc_bound,
                                    length_low_row(rep_len_decoder, pos_state),
                                    -(1_i32 << 3) + 2,
                                    symbol
                                );
                                len = symbol;
                            } else {
                                rc.range = rc.range.wrapping_sub(rc_bound);
                                rc.code = rc.code.wrapping_sub(rc_bound);
                                prob_update_1(::core::ptr::addr_of_mut!((*rep_len_decoder).choice));
                                rc_normalize!(rc, rc_in_ptr);
                                rc_bound = (rc.range >> RC_BIT_MODEL_TOTAL_BITS)
                                    .wrapping_mul((*rep_len_decoder).choice2 as u32);
                                if rc.code < rc_bound {
                                    rc.range = rc_bound;
                                    prob_update_0(::core::ptr::addr_of_mut!(
                                        (*rep_len_decoder).choice2
                                    ));
                                    rc_bittree3!(
                                        rc,
                                        rc_in_ptr,
                                        rc_bound,
                                        length_mid_row(rep_len_decoder, pos_state),
                                        -(1_i32 << 3) + 2 + (1 << 3),
                                        symbol
                                    );
                                    len = symbol;
                                } else {
                                    rc.range = rc.range.wrapping_sub(rc_bound);
                                    rc.code = rc.code.wrapping_sub(rc_bound);
                                    prob_update_1(::core::ptr::addr_of_mut!(
                                        (*rep_len_decoder).choice2
                                    ));
                                    rc_bittree8!(
                                        rc,
                                        rc_in_ptr,
                                        rc_bound,
                                        length_high_probs(rep_len_decoder),
                                        -(1_i32 << 8) + 2 + (1 << 3) + (1 << 3),
                                        symbol
                                    );
                                    len = symbol;
                                }
                            }
                        }
                    }
                    if !dict_repeat(
                        ::core::ptr::addr_of_mut!(dict),
                        rep0,
                        ::core::ptr::addr_of_mut!(len),
                    ) {
                        continue;
                    }
                    (*coder).sequence = SEQ_COPY;
                    block_state = BLOCK_RETURN;
                    continue 'c_9380;
                }
            },
            BLOCK_LEN_BITTREE_INIT => {
                symbol = 1;
                block_state = BLOCK_REP_LEN_BITTREE;
                continue;
            }
            BLOCK_REP_LEN_PREPARE => {
                state = (if state < LIT_STATES {
                    STATE_LIT_LONGREP
                } else {
                    STATE_NONLIT_REP
                }) as u32;
                block_state = BLOCK_REP_LEN_CHOICE;
                continue;
            }
            BLOCK_DIST_SLOT_INIT => {
                symbol = 1;
                block_state = BLOCK_MATCH_LEN_BITTREE;
                continue;
            }
            _ => {}
        }
        if eopm_is_valid {
            block_state = BLOCK_EOPM;
            continue;
        }
        ret = LZMA_DATA_ERROR;
        block_state = BLOCK_RETURN;
    }
    (*dictptr).full = dict.full;
    (*coder).rc = rc;
    *in_pos = rc_in_ptr.offset_from(input) as size_t;
    (*coder).state = state as lzma_lzma_state;
    (*coder).rep0 = rep0;
    (*coder).rep1 = rep1;
    (*coder).rep2 = rep2;
    (*coder).rep3 = rep3;
    (*coder).probs = probs;
    (*coder).symbol = symbol;
    (*coder).limit = limit;
    (*coder).offset = offset;
    (*coder).len = len;
    if (*coder).uncompressed_size != LZMA_VLI_UNKNOWN {
        (*coder).uncompressed_size = (*coder)
            .uncompressed_size
            .wrapping_sub(dict.pos.wrapping_sub(dict_start) as lzma_vli);
        if (*coder).uncompressed_size == 0
            && ret == LZMA_OK
            && ((*coder).sequence == SEQ_LITERAL_WRITE
                || (*coder).sequence == SEQ_SHORTREP
                || (*coder).sequence == SEQ_COPY)
        {
            ret = LZMA_DATA_ERROR;
        }
    }
    if ret == LZMA_STREAM_END {
        (*coder).rc.range = UINT32_MAX;
        (*coder).rc.code = 0;
        (*coder).rc.init_bytes_left = 5;
        (*coder).sequence = SEQ_IS_MATCH;
    }
    ret
}
unsafe fn lzma_decoder_uncompressed(
    coder_ptr: *mut c_void,
    uncompressed_size: lzma_vli,
    allow_eopm: bool,
) {
    let coder: *mut lzma_lzma1_decoder = coder_ptr as *mut lzma_lzma1_decoder;
    (*coder).uncompressed_size = uncompressed_size;
    (*coder).allow_eopm = allow_eopm;
}
unsafe fn lzma_decoder_reset(coder_ptr: *mut c_void, opt: *const c_void) {
    let coder: *mut lzma_lzma1_decoder = coder_ptr as *mut lzma_lzma1_decoder;
    let options: *const lzma_options_lzma = opt as *const lzma_options_lzma;
    (*coder).pos_mask = (1u32 << (*options).pb).wrapping_sub(1) as u32;
    literal_init(
        ::core::ptr::addr_of_mut!((*coder).literal) as *mut probability,
        (*options).lc,
        (*options).lp,
    );
    (*coder).literal_context_bits = (*options).lc;
    (*coder).literal_mask = (0x100u32 << (*options).lp).wrapping_sub(0x100 >> (*options).lc);
    (*coder).state = STATE_LIT_LIT;
    (*coder).rep0 = 0;
    (*coder).rep1 = 0;
    (*coder).rep2 = 0;
    (*coder).rep3 = 0;
    (*coder).rc.range = UINT32_MAX;
    (*coder).rc.code = 0;
    (*coder).rc.init_bytes_left = 5;
    let match_len_decoder: *mut lzma_length_decoder =
        ::core::ptr::addr_of_mut!((*coder).match_len_decoder);
    let rep_len_decoder: *mut lzma_length_decoder =
        ::core::ptr::addr_of_mut!((*coder).rep_len_decoder);
    let mut i: u32 = 0;
    while i < STATES {
        let is_match = decoder_is_match_row(coder, i);
        let is_rep0_long = decoder_is_rep0_long_row(coder, i);
        let mut j: u32 = 0;
        while j <= (*coder).pos_mask {
            *is_match.add(j as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            *is_rep0_long.add(j as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            j += 1;
        }
        *decoder_is_rep_prob(coder, i) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        *decoder_is_rep0_prob(coder, i) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        *decoder_is_rep1_prob(coder, i) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        *decoder_is_rep2_prob(coder, i) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        i += 1;
    }
    let mut dist_state: u32 = 0;
    while dist_state < DIST_STATES {
        let dist_slot = decoder_dist_slot_row(coder, dist_state);
        let mut bt_i: u32 = 0;
        while bt_i < (1 << 6) as u32 {
            *dist_slot.add(bt_i as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            bt_i += 1;
        }
        dist_state += 1;
    }
    let mut special_distance: u32 = 0;
    while special_distance < (FULL_DISTANCES - DIST_MODEL_END) as u32 {
        (*coder).pos_special[special_distance as usize] = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        special_distance += 1;
    }
    let mut bt_i_0: u32 = 0;
    while bt_i_0 < (1 << 4) as u32 {
        *decoder_pos_align_prob(coder, bt_i_0) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        bt_i_0 += 1;
    }
    let num_pos_states: u32 = 1 << (*options).pb;
    (*coder).match_len_decoder.choice = (RC_BIT_MODEL_TOTAL >> 1) as probability;
    (*coder).match_len_decoder.choice2 = (RC_BIT_MODEL_TOTAL >> 1) as probability;
    (*coder).rep_len_decoder.choice = (RC_BIT_MODEL_TOTAL >> 1) as probability;
    (*coder).rep_len_decoder.choice2 = (RC_BIT_MODEL_TOTAL >> 1) as probability;
    let mut pos_state: u32 = 0;
    while pos_state < num_pos_states {
        let match_len_low = length_low_row(match_len_decoder, pos_state);
        let match_len_mid = length_mid_row(match_len_decoder, pos_state);
        let rep_len_low = length_low_row(rep_len_decoder, pos_state);
        let rep_len_mid = length_mid_row(rep_len_decoder, pos_state);
        let mut bt_i_1: u32 = 0;
        while bt_i_1 < (1 << 3) as u32 {
            *match_len_low.add(bt_i_1 as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            bt_i_1 += 1;
        }
        let mut bt_i_2: u32 = 0;
        while bt_i_2 < (1 << 3) as u32 {
            *match_len_mid.add(bt_i_2 as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            bt_i_2 += 1;
        }
        let mut bt_i_3: u32 = 0;
        while bt_i_3 < (1 << 3) as u32 {
            *rep_len_low.add(bt_i_3 as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            bt_i_3 += 1;
        }
        let mut bt_i_4: u32 = 0;
        while bt_i_4 < (1 << 3) as u32 {
            *rep_len_mid.add(bt_i_4 as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            bt_i_4 += 1;
        }
        pos_state += 1;
    }
    let match_len_high = length_high_probs(match_len_decoder);
    let mut bt_i_5: u32 = 0;
    while bt_i_5 < (1 << 8) as u32 {
        *match_len_high.add(bt_i_5 as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        bt_i_5 += 1;
    }
    let rep_len_high = length_high_probs(rep_len_decoder);
    let mut bt_i_6: u32 = 0;
    while bt_i_6 < (1 << 8) as u32 {
        *rep_len_high.add(bt_i_6 as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        bt_i_6 += 1;
    }
    (*coder).sequence = SEQ_IS_MATCH;
    (*coder).probs = core::ptr::null_mut();
    (*coder).symbol = 0;
    (*coder).limit = 0;
    (*coder).offset = 0;
    (*coder).len = 0;
}
pub unsafe fn lzma_lzma_decoder_create(
    lz: *mut lzma_lz_decoder,
    allocator: *const lzma_allocator,
    options: *const lzma_options_lzma,
    lz_options: *mut lzma_lz_options,
) -> lzma_ret {
    if (*lz).coder.is_null() {
        (*lz).coder = crate::alloc::internal_alloc_object::<lzma_lzma1_decoder>(allocator).cast();
        if (*lz).coder.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*lz).code = lzma_decode as lzma_lz_decoder_code_function;
        (*lz).end = Some(lzma_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
        (*lz).reset = Some(lzma_decoder_reset as unsafe fn(*mut c_void, *const c_void) -> ());
        (*lz).set_uncompressed =
            Some(lzma_decoder_uncompressed as unsafe fn(*mut c_void, lzma_vli, bool) -> ());
    }
    (*lz_options).dict_size = (*options).dict_size as size_t;
    (*lz_options).preset_dict = (*options).preset_dict;
    (*lz_options).preset_dict_size = (*options).preset_dict_size as size_t;
    LZMA_OK
}
unsafe fn lzma_decoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    crate::alloc::internal_free(coder_ptr as *mut lzma_lzma1_decoder, allocator);
}
unsafe fn lzma_decoder_init(
    lz: *mut lzma_lz_decoder,
    allocator: *const lzma_allocator,
    id: lzma_vli,
    options: *const c_void,
    lz_options: *mut lzma_lz_options,
) -> lzma_ret {
    if !is_lclppb_valid(options as *const lzma_options_lzma) {
        return LZMA_PROG_ERROR;
    }
    let mut uncomp_size: lzma_vli = LZMA_VLI_UNKNOWN;
    let mut allow_eopm: bool = true;
    if id == LZMA_FILTER_LZMA1EXT {
        let opt: *const lzma_options_lzma = options as *const lzma_options_lzma;
        if (*opt).ext_flags & !(LZMA_LZMA1EXT_ALLOW_EOPM as u32) != 0 {
            return LZMA_OPTIONS_ERROR;
        }
        uncomp_size = ((*opt).ext_size_low as u64).wrapping_add(((*opt).ext_size_high as u64) << 32)
            as lzma_vli;
        allow_eopm = (*opt).ext_flags & LZMA_LZMA1EXT_ALLOW_EOPM as u32 != 0
            || uncomp_size == LZMA_VLI_UNKNOWN;
    }
    let ret: lzma_ret = lzma_lzma_decoder_create(
        lz,
        allocator,
        options as *const lzma_options_lzma,
        lz_options,
    );
    if ret != LZMA_OK {
        return ret;
    }
    (*lz).end = Some(lzma_decoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
    lzma_decoder_reset((*lz).coder, options);
    lzma_decoder_uncompressed((*lz).coder, uncomp_size, allow_eopm);
    LZMA_OK
}
pub(crate) unsafe fn lzma_lzma_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
) -> lzma_ret {
    lzma_lz_decoder_init(
        next,
        allocator,
        filters,
        lzma_decoder_init
            as unsafe fn(
                *mut lzma_lz_decoder,
                *const lzma_allocator,
                lzma_vli,
                *const c_void,
                *mut lzma_lz_options,
            ) -> lzma_ret,
    )
}
pub unsafe fn lzma_lzma_lclppb_decode(options: *mut lzma_options_lzma, mut byte: u8) -> bool {
    if byte > (4 * 5 + 4) * 9 + 8 {
        return true;
    }
    (*options).pb = (byte / (9 * 5)) as u32;
    byte = (byte as u32).wrapping_sub((*options).pb.wrapping_mul(9u32).wrapping_mul(5)) as u8;
    (*options).lp = (byte / 9) as u32;
    (*options).lc = (byte as u32).wrapping_sub((*options).lp.wrapping_mul(9));
    (*options).lc.wrapping_add((*options).lp) > LZMA_LCLP_MAX
}
pub(crate) unsafe fn lzma_lzma_decoder_memusage_nocheck(options: *const c_void) -> u64 {
    let opt: *const lzma_options_lzma = options as *const lzma_options_lzma;
    (core::mem::size_of::<lzma_lzma1_decoder>() as u64)
        .wrapping_add(lzma_lz_decoder_memusage((*opt).dict_size as size_t))
}
pub(crate) unsafe fn lzma_lzma_decoder_memusage(options: *const c_void) -> u64 {
    if !is_lclppb_valid(options as *const lzma_options_lzma) {
        return UINT64_MAX;
    }
    lzma_lzma_decoder_memusage_nocheck(options)
}
pub(crate) unsafe fn lzma_lzma_props_decode(
    options: *mut *mut c_void,
    allocator: *const lzma_allocator,
    props: *const u8,
    props_size: size_t,
) -> lzma_ret {
    if props_size != 5 {
        return LZMA_OPTIONS_ERROR;
    }
    let opt: *mut lzma_options_lzma =
        crate::alloc::internal_alloc_object::<lzma_options_lzma>(allocator);
    if opt.is_null() {
        return LZMA_MEM_ERROR;
    }
    if lzma_lzma_lclppb_decode(opt, *props) {
        crate::alloc::internal_free(opt, allocator);
        return LZMA_OPTIONS_ERROR;
    } else {
        (*opt).dict_size = read32le(&*props.add(1).cast::<[u8; 4]>());
        (*opt).preset_dict = core::ptr::null();
        (*opt).preset_dict_size = 0;
        *options = opt as *mut c_void;
        return LZMA_OK;
    };
}
