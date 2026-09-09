use crate::lz::lz_encoder::{lzma_lz_encoder_init, lzma_lz_encoder_memusage, lzma_lz_options};
use crate::lzma::lzma_encoder_optimum_fast::lzma_lzma_optimum_fast;
use crate::lzma::lzma_encoder_optimum_normal::lzma_lzma_optimum_normal;
use crate::types::*;
#[inline]
unsafe fn rc_reset(rc: *mut lzma_range_encoder) {
    (*rc).low = 0;
    (*rc).cache_size = 1;
    (*rc).range = UINT32_MAX;
    (*rc).cache = 0;
    (*rc).out_total = 0;
    (*rc).count = 0;
    (*rc).pos = 0;
}
#[inline]
unsafe fn rc_forget(rc: *mut lzma_range_encoder) {
    (*rc).count = 0;
}
#[inline]
unsafe fn rc_bit(rc: *mut lzma_range_encoder, prob: *mut probability, bit: u32) {
    *rc_symbol_slot_mut(rc, (*rc).count) = bit as rc_symbol;
    *rc_prob_slot_mut(rc, (*rc).count) = prob;
    (*rc).count += 1;
}

#[inline(always)]
unsafe fn rc_symbol_slot_mut(rc: *mut lzma_range_encoder, index: size_t) -> *mut rc_symbol {
    debug_assert!(index < 53);
    (::core::ptr::addr_of_mut!((*rc).symbols) as *mut rc_symbol).add(index)
}

#[inline(always)]
unsafe fn rc_prob_slot_mut(rc: *mut lzma_range_encoder, index: size_t) -> *mut *mut probability {
    debug_assert!(index < 53);
    (::core::ptr::addr_of_mut!((*rc).probs) as *mut *mut probability).add(index)
}

#[inline(always)]
unsafe fn coder_rep_slot_mut(coder: *mut lzma_lzma1_encoder, index: usize) -> *mut u32 {
    debug_assert!(index < REPS as usize);
    (::core::ptr::addr_of_mut!((*coder).reps) as *mut u32).add(index)
}
#[inline(always)]
unsafe fn encoder_is_match_row(coder: *mut lzma_lzma1_encoder, state: u32) -> *mut probability {
    debug_assert!((state as usize) < STATES as usize);
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*coder).is_match) as *mut [probability; 16])
            .add(state as usize)
    ) as *mut probability
}
#[inline(always)]
unsafe fn encoder_is_rep0_long_row(coder: *mut lzma_lzma1_encoder, state: u32) -> *mut probability {
    debug_assert!((state as usize) < STATES as usize);
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*coder).is_rep0_long) as *mut [probability; 16])
            .add(state as usize)
    ) as *mut probability
}
#[inline(always)]
unsafe fn encoder_is_rep_slot(
    probs: *mut [probability; STATES as usize],
    state: u32,
) -> *mut probability {
    debug_assert!((state as usize) < STATES as usize);
    (probs as *mut probability).add(state as usize)
}
#[inline(always)]
unsafe fn encoder_dist_special_prob(
    coder: *mut lzma_lzma1_encoder,
    index: usize,
) -> *mut probability {
    debug_assert!(index < (FULL_DISTANCES - DIST_MODEL_END) as usize);
    (::core::ptr::addr_of_mut!((*coder).dist_special) as *mut probability).add(index)
}
#[inline(always)]
unsafe fn encoder_dist_slot_row(coder: *mut lzma_lzma1_encoder, state: u32) -> *mut probability {
    debug_assert!((state as usize) < DIST_STATES as usize);
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*coder).dist_slot) as *mut [probability; 64])
            .add(state as usize)
    ) as *mut probability
}
#[inline(always)]
unsafe fn encoder_dist_align_prob(coder: *mut lzma_lzma1_encoder, index: u32) -> *mut probability {
    debug_assert!((index as usize) < ALIGN_SIZE as usize);
    (::core::ptr::addr_of_mut!((*coder).dist_align) as *mut probability).add(index as usize)
}
#[inline(always)]
unsafe fn rc_bittree(
    rc: *mut lzma_range_encoder,
    probs: *mut probability,
    mut bit_count: u32,
    symbol: u32,
) {
    let mut model_index: u32 = 1;
    loop {
        bit_count -= 1;
        let bit: u32 = symbol >> bit_count & 1;
        rc_bit(
            rc,
            probs.offset(model_index as isize) as *mut probability,
            bit,
        );
        model_index = (model_index << 1) + bit;
        if bit_count == 0 {
            break;
        }
    }
}
#[inline(always)]
unsafe fn rc_bittree_reverse(
    rc: *mut lzma_range_encoder,
    probs: *mut probability,
    mut bit_count: u32,
    mut symbol: u32,
) {
    let mut model_index: u32 = 1;
    loop {
        let bit: u32 = symbol & 1;
        symbol >>= 1;
        rc_bit(
            rc,
            probs.offset(model_index as isize) as *mut probability,
            bit,
        );
        model_index = (model_index << 1) + bit;
        bit_count -= 1;
        if bit_count == 0 {
            break;
        }
    }
}
#[inline(always)]
unsafe fn rc_direct(rc: *mut lzma_range_encoder, value: u32, mut bit_count: u32) {
    loop {
        bit_count -= 1;
        *rc_symbol_slot_mut(rc, (*rc).count) =
            ((RC_DIRECT_0 as u32) + (value >> bit_count & 1)) as rc_symbol;
        (*rc).count += 1;
        if bit_count == 0 {
            break;
        }
    }
}
#[inline(always)]
unsafe fn rc_flush(rc: *mut lzma_range_encoder) {
    let mut i: size_t = 0;
    while i < 5 {
        *rc_symbol_slot_mut(rc, (*rc).count) = RC_FLUSH;
        (*rc).count += 1;
        i += 1;
    }
}
#[inline(always)]
unsafe fn rc_shift_low_raw(
    low: &mut u64,
    cache_size: &mut u64,
    cache: &mut u8,
    out_total: &mut u64,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> bool {
    if (*low as u32) < 0xff000000 || (*low >> 32) as u32 != 0 {
        loop {
            if *out_pos == out_size {
                return true;
            }
            *out.add(*out_pos) = cache.wrapping_add((*low >> 32) as u8);
            *out_pos += 1;
            *out_total += 1;
            *cache = 0xff;
            *cache_size -= 1;
            if *cache_size == 0 {
                break;
            }
        }
        *cache = ((*low >> 24) & 0xff) as u8;
    }
    *cache_size += 1;
    *low = (*low & 0x00ff_ffff) << RC_SHIFT_BITS;
    false
}
#[inline]
unsafe fn rc_shift_low_dummy(
    low: *mut u64,
    cache_size: *mut u64,
    cache: *mut u8,
    out_pos: *mut u64,
    out_size: u64,
) -> bool {
    if (*low as u32) < 0xff000000 || (*low >> 32) as u32 != 0 {
        loop {
            if *out_pos == out_size {
                return true;
            }
            *out_pos += 1;
            *cache = 0xff;
            *cache_size -= 1;
            if *cache_size == 0 {
                break;
            }
        }
        *cache = (*low >> 24 & 0xff as u64) as u8;
    }
    *cache_size += 1;
    *low = (*low & 0xffffff as u64) << RC_SHIFT_BITS;
    false
}
#[inline(always)]
unsafe fn rc_encode(
    rc: *mut lzma_range_encoder,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> bool {
    let symbols = ::core::ptr::addr_of_mut!((*rc).symbols) as *mut rc_symbol;
    let probs = ::core::ptr::addr_of_mut!((*rc).probs) as *mut *mut probability;
    let mut low = (*rc).low;
    let mut cache_size = (*rc).cache_size;
    let mut range = (*rc).range;
    let mut cache = (*rc).cache;
    let mut out_total = (*rc).out_total;
    let count = (*rc).count;
    let mut pos = (*rc).pos;

    while pos < count {
        if range < RC_TOP_VALUE as u32 {
            if rc_shift_low_raw(
                &mut low,
                &mut cache_size,
                &mut cache,
                &mut out_total,
                out,
                out_pos,
                out_size,
            ) {
                (*rc).low = low;
                (*rc).cache_size = cache_size;
                (*rc).range = range;
                (*rc).cache = cache;
                (*rc).out_total = out_total;
                (*rc).pos = pos;
                return true;
            }
            range <<= RC_SHIFT_BITS;
        }
        match *symbols.add(pos) {
            0 => {
                let prob_ptr = *probs.add(pos);
                let mut prob: probability = *prob_ptr;
                range = (range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(prob as u32);
                prob = (prob as u32)
                    .wrapping_add(RC_BIT_MODEL_TOTAL.wrapping_sub(prob as u32) >> RC_MOVE_BITS)
                    as probability;
                *prob_ptr = prob;
            }
            1 => {
                let prob_ptr = *probs.add(pos);
                let mut prob_0: probability = *prob_ptr;
                let bound: u32 = (prob_0 as u32).wrapping_mul(range >> RC_BIT_MODEL_TOTAL_BITS);
                low = low.wrapping_add(bound as u64);
                range = range.wrapping_sub(bound);
                prob_0 -= prob_0 >> RC_MOVE_BITS;
                *prob_ptr = prob_0;
            }
            2 => {
                range >>= 1;
            }
            3 => {
                range >>= 1;
                low = low.wrapping_add(range as u64);
            }
            4 => {
                range = UINT32_MAX;
                loop {
                    if rc_shift_low_raw(
                        &mut low,
                        &mut cache_size,
                        &mut cache,
                        &mut out_total,
                        out,
                        out_pos,
                        out_size,
                    ) {
                        (*rc).low = low;
                        (*rc).cache_size = cache_size;
                        (*rc).range = range;
                        (*rc).cache = cache;
                        (*rc).out_total = out_total;
                        (*rc).pos = pos;
                        return true;
                    }
                    pos += 1;
                    if pos >= count {
                        break;
                    }
                }
                rc_reset(rc);
                return false;
            }
            _ => {}
        }
        pos += 1;
    }
    (*rc).low = low;
    (*rc).cache_size = cache_size;
    (*rc).range = range;
    (*rc).cache = cache;
    (*rc).out_total = out_total;
    (*rc).count = 0;
    (*rc).pos = 0;
    false
}
#[inline]
unsafe fn rc_encode_dummy(rc: *const lzma_range_encoder, out_limit: u64) -> bool {
    let symbols = ::core::ptr::addr_of!((*rc).symbols) as *const rc_symbol;
    let probs = ::core::ptr::addr_of!((*rc).probs) as *const *mut probability;
    let mut low: u64 = (*rc).low;
    let mut cache_size: u64 = (*rc).cache_size;
    let mut range: u32 = (*rc).range;
    let mut cache: u8 = (*rc).cache;
    let mut out_pos: u64 = (*rc).out_total;
    let mut pos: size_t = (*rc).pos;
    loop {
        if range < RC_TOP_VALUE as u32 {
            if rc_shift_low_dummy(
                ::core::ptr::addr_of_mut!(low),
                ::core::ptr::addr_of_mut!(cache_size),
                ::core::ptr::addr_of_mut!(cache),
                ::core::ptr::addr_of_mut!(out_pos),
                out_limit,
            ) {
                return true;
            }
            range <<= RC_SHIFT_BITS;
        }
        if pos == (*rc).count {
            break;
        }
        match *symbols.add(pos) {
            0 => {
                let prob: probability = **probs.add(pos);
                range = (range >> RC_BIT_MODEL_TOTAL_BITS).wrapping_mul(prob as u32);
            }
            1 => {
                let prob_0: probability = **probs.add(pos);
                let bound: u32 = (prob_0 as u32).wrapping_mul(range >> RC_BIT_MODEL_TOTAL_BITS);
                low = low.wrapping_add(bound as u64);
                range = range.wrapping_sub(bound);
            }
            2 => {
                range >>= 1;
            }
            3 => {
                range >>= 1;
                low = low.wrapping_add(range as u64);
            }
            4 | _ => {}
        }
        pos += 1;
    }
    pos = 0;
    while pos < 5 {
        if rc_shift_low_dummy(
            ::core::ptr::addr_of_mut!(low),
            ::core::ptr::addr_of_mut!(cache_size),
            ::core::ptr::addr_of_mut!(cache),
            ::core::ptr::addr_of_mut!(out_pos),
            out_limit,
        ) {
            return true;
        }
        pos += 1;
    }
    false
}
#[inline]
unsafe fn rc_pending(rc: *const lzma_range_encoder) -> u64 {
    (*rc).cache_size + 5 - 1
}
pub const LEN_SYMBOLS: u32 = LEN_LOW_SYMBOLS + LEN_MID_SYMBOLS + LEN_HIGH_SYMBOLS;
pub const MATCH_LEN_MAX: u32 = MATCH_LEN_MIN + LEN_SYMBOLS - 1;
#[inline(always)]
unsafe fn literal_matched(
    rc: *mut lzma_range_encoder,
    subcoder: *mut probability,
    mut match_byte: u32,
    mut symbol: u32,
) {
    let mut offset: u32 = 0x100;
    symbol = (symbol as u32 + (1u32 << 8)) as u32;
    loop {
        match_byte <<= 1;
        let match_bit: u32 = match_byte & offset;
        let subcoder_index: u32 = offset + match_bit + (symbol >> 8);
        let bit: u32 = symbol >> 7 & 1;
        rc_bit(
            rc,
            subcoder.offset(subcoder_index as isize) as *mut probability,
            bit,
        );
        symbol <<= 1;
        offset &= !(match_byte ^ symbol);
        if symbol >= 1 << 16 {
            break;
        }
    }
}
#[inline(always)]
unsafe fn literal(coder: *mut lzma_lzma1_encoder, mf: *mut lzma_mf, position: u32) {
    let cur_byte: u8 = *(*mf)
        .buffer
        .offset(((*mf).read_pos - (*mf).read_ahead) as isize);
    let subcoder: *mut probability =
        (::core::ptr::addr_of_mut!((*coder).literal) as *mut probability).offset(
            (3u32
                * (((position << 8)
                    + *(*mf)
                        .buffer
                        .offset(((*mf).read_pos - (*mf).read_ahead - 1) as isize)
                        as u32)
                    & (*coder).literal_mask)
                << (*coder).literal_context_bits) as isize,
        );
    if ((*coder).state as u32) < LIT_STATES {
        (*coder).state = (if (*coder).state <= STATE_SHORTREP_LIT_LIT {
            STATE_LIT_LIT
        } else {
            (*coder).state as u32 - 3
        }) as lzma_lzma_state;
        rc_bittree(
            ::core::ptr::addr_of_mut!((*coder).rc),
            subcoder,
            8,
            cur_byte as u32,
        );
    } else {
        (*coder).state = (if (*coder).state <= STATE_LIT_SHORTREP {
            (*coder).state as u32 - 3
        } else {
            (*coder).state as u32 - 6
        }) as lzma_lzma_state;
        let match_byte: u8 = *(*mf).buffer.offset(
            ((*mf).read_pos - *coder_rep_slot_mut(coder, 0) - 1 - (*mf).read_ahead) as isize,
        );
        literal_matched(
            ::core::ptr::addr_of_mut!((*coder).rc),
            subcoder,
            match_byte as u32,
            cur_byte as u32,
        );
    };
}
#[inline(always)]
unsafe fn length_prices_row(lc: *mut lzma_length_encoder, pos_state: u32) -> *mut u32 {
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*lc).prices) as *mut [u32; 272]).add(pos_state as usize)
    ) as *mut u32
}
#[inline(always)]
unsafe fn length_low_probs(lc: *mut lzma_length_encoder, pos_state: u32) -> *mut probability {
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*lc).low) as *mut [probability; 8]).add(pos_state as usize)
    ) as *mut probability
}
#[inline(always)]
unsafe fn length_mid_probs(lc: *mut lzma_length_encoder, pos_state: u32) -> *mut probability {
    ::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*lc).mid) as *mut [probability; 8]).add(pos_state as usize)
    ) as *mut probability
}
#[inline(always)]
unsafe fn length_high_probs(lc: *mut lzma_length_encoder) -> *mut probability {
    ::core::ptr::addr_of_mut!((*lc).high) as *mut probability
}
#[inline(always)]
unsafe fn length_counter(lc: *mut lzma_length_encoder, pos_state: u32) -> *mut u32 {
    debug_assert!((pos_state as usize) < (*lc).counters.len());
    (::core::ptr::addr_of_mut!((*lc).counters) as *mut u32).add(pos_state as usize)
}
#[inline(always)]
unsafe fn is_match_prob(coder: *mut lzma_lzma1_encoder, pos_state: u32) -> *mut probability {
    (::core::ptr::addr_of_mut!(
        *(::core::ptr::addr_of_mut!((*coder).is_match) as *mut [probability; 16])
            .add((*coder).state as usize)
    ) as *mut probability)
        .add(pos_state as usize)
}
#[inline(always)]
unsafe fn is_rep_prob(coder: *mut lzma_lzma1_encoder) -> *mut probability {
    (::core::ptr::addr_of_mut!((*coder).is_rep) as *mut probability).add((*coder).state as usize)
}
unsafe fn length_update_prices(lc: *mut lzma_length_encoder, pos_state: u32) {
    let table_size: u32 = (*lc).table_size;
    *length_counter(lc, pos_state) = table_size;
    let a0: u32 = rc_bit_0_price((*lc).choice) as u32;
    let a1: u32 = rc_bit_1_price((*lc).choice) as u32;
    let b0: u32 = a1 + rc_bit_0_price((*lc).choice2) as u32;
    let b1: u32 = a1 + rc_bit_1_price((*lc).choice2) as u32;
    let prices: *mut u32 = length_prices_row(lc, pos_state);
    let low: *mut probability = length_low_probs(lc, pos_state);
    let mid: *mut probability = length_mid_probs(lc, pos_state);
    let high: *mut probability = ::core::ptr::addr_of_mut!((*lc).high) as *mut probability;
    let mut i: u32 = 0;
    while i < table_size && i < LEN_LOW_SYMBOLS {
        *prices.add(i as usize) = a0 + rc_bittree_price(low, LEN_LOW_BITS, i);
        i += 1;
    }
    while i < table_size && i < LEN_LOW_SYMBOLS + LEN_MID_SYMBOLS {
        *prices.add(i as usize) = b0 + rc_bittree_price(mid, LEN_MID_BITS, i - LEN_LOW_SYMBOLS);
        i += 1;
    }
    while i < table_size {
        *prices.add(i as usize) =
            b1 + rc_bittree_price(high, LEN_HIGH_BITS, i - LEN_LOW_SYMBOLS - LEN_MID_SYMBOLS);
        i += 1;
    }
}
#[inline(always)]
unsafe fn length(
    rc: *mut lzma_range_encoder,
    lc: *mut lzma_length_encoder,
    pos_state: u32,
    mut len: u32,
    fast_mode: bool,
) {
    len -= MATCH_LEN_MIN;
    if len < LEN_LOW_SYMBOLS {
        rc_bit(rc, ::core::ptr::addr_of_mut!((*lc).choice), 0);
        rc_bittree(rc, length_low_probs(lc, pos_state), LEN_LOW_BITS, len);
    } else {
        rc_bit(rc, ::core::ptr::addr_of_mut!((*lc).choice), 1);
        len -= LEN_LOW_SYMBOLS;
        if len < LEN_MID_SYMBOLS {
            rc_bit(rc, ::core::ptr::addr_of_mut!((*lc).choice2), 0);
            rc_bittree(rc, length_mid_probs(lc, pos_state), LEN_MID_BITS, len);
        } else {
            rc_bit(rc, ::core::ptr::addr_of_mut!((*lc).choice2), 1);
            len -= LEN_MID_SYMBOLS;
            rc_bittree(
                rc,
                ::core::ptr::addr_of_mut!((*lc).high) as *mut probability,
                LEN_HIGH_BITS,
                len,
            );
        }
    }
    if !fast_mode {
        let counter = length_counter(lc, pos_state);
        *counter -= 1;
        if *counter == 0 {
            length_update_prices(lc, pos_state);
        }
    }
}
#[inline(always)]
unsafe fn match_0(coder: *mut lzma_lzma1_encoder, pos_state: u32, distance: u32, len: u32) {
    (*coder).state = (if ((*coder).state as u32) < LIT_STATES {
        STATE_LIT_MATCH
    } else {
        STATE_NONLIT_MATCH
    }) as lzma_lzma_state;
    length(
        ::core::ptr::addr_of_mut!((*coder).rc),
        ::core::ptr::addr_of_mut!((*coder).match_len_encoder),
        pos_state,
        len,
        (*coder).fast_mode,
    );
    let dist_slot: u32 = get_dist_slot(distance) as u32;
    let dist_state: u32 = if len < (DIST_STATES + MATCH_LEN_MIN) as u32 {
        len - MATCH_LEN_MIN
    } else {
        (DIST_STATES - 1) as u32
    };
    rc_bittree(
        ::core::ptr::addr_of_mut!((*coder).rc),
        ::core::ptr::addr_of_mut!(
            *(::core::ptr::addr_of_mut!((*coder).dist_slot) as *mut [probability; 64])
                .offset(dist_state as isize)
        ) as *mut probability,
        DIST_SLOT_BITS,
        dist_slot,
    );
    if dist_slot >= DIST_MODEL_START {
        let footer_bits: u32 = (dist_slot >> 1) - 1;
        let base: u32 = (2 | dist_slot & 1) << footer_bits;
        let dist_reduced: u32 = distance - base;
        if dist_slot < DIST_MODEL_END {
            rc_bittree_reverse(
                ::core::ptr::addr_of_mut!((*coder).rc),
                (::core::ptr::addr_of_mut!((*coder).dist_special) as *mut probability)
                    .offset(base as isize)
                    .offset(-(dist_slot as isize))
                    .offset(-1),
                footer_bits,
                dist_reduced,
            );
        } else {
            rc_direct(
                ::core::ptr::addr_of_mut!((*coder).rc),
                dist_reduced >> ALIGN_BITS,
                footer_bits - ALIGN_BITS,
            );
            rc_bittree_reverse(
                ::core::ptr::addr_of_mut!((*coder).rc),
                ::core::ptr::addr_of_mut!((*coder).dist_align) as *mut probability,
                ALIGN_BITS,
                dist_reduced & ALIGN_MASK,
            );
            (*coder).align_price_count += 1;
        }
    }
    *coder_rep_slot_mut(coder, 3) = *coder_rep_slot_mut(coder, 2);
    *coder_rep_slot_mut(coder, 2) = *coder_rep_slot_mut(coder, 1);
    *coder_rep_slot_mut(coder, 1) = *coder_rep_slot_mut(coder, 0);
    *coder_rep_slot_mut(coder, 0) = distance;
    (*coder).match_price_count += 1;
}
#[inline(always)]
unsafe fn rep_match(coder: *mut lzma_lzma1_encoder, pos_state: u32, rep: u32, len: u32) {
    if rep == 0 {
        rc_bit(
            ::core::ptr::addr_of_mut!((*coder).rc),
            (::core::ptr::addr_of_mut!((*coder).is_rep0) as *mut probability)
                .offset((*coder).state as isize) as *mut probability,
            0,
        );
        rc_bit(
            ::core::ptr::addr_of_mut!((*coder).rc),
            (::core::ptr::addr_of_mut!(
                *(::core::ptr::addr_of_mut!((*coder).is_rep0_long) as *mut [probability; 16])
                    .offset((*coder).state as isize)
            ) as *mut probability)
                .offset(pos_state as isize) as *mut probability,
            (len != 1) as u32,
        );
    } else {
        let distance: u32 = *coder_rep_slot_mut(coder, rep as usize);
        rc_bit(
            ::core::ptr::addr_of_mut!((*coder).rc),
            (::core::ptr::addr_of_mut!((*coder).is_rep0) as *mut probability)
                .offset((*coder).state as isize) as *mut probability,
            1,
        );
        if rep == 1 {
            rc_bit(
                ::core::ptr::addr_of_mut!((*coder).rc),
                (::core::ptr::addr_of_mut!((*coder).is_rep1) as *mut probability)
                    .offset((*coder).state as isize) as *mut probability,
                0,
            );
        } else {
            rc_bit(
                ::core::ptr::addr_of_mut!((*coder).rc),
                (::core::ptr::addr_of_mut!((*coder).is_rep1) as *mut probability)
                    .offset((*coder).state as isize) as *mut probability,
                1,
            );
            rc_bit(
                ::core::ptr::addr_of_mut!((*coder).rc),
                (::core::ptr::addr_of_mut!((*coder).is_rep2) as *mut probability)
                    .offset((*coder).state as isize) as *mut probability,
                rep - 2,
            );
            if rep == 3 {
                *coder_rep_slot_mut(coder, 3) = *coder_rep_slot_mut(coder, 2);
            }
            *coder_rep_slot_mut(coder, 2) = *coder_rep_slot_mut(coder, 1);
        }
        *coder_rep_slot_mut(coder, 1) = *coder_rep_slot_mut(coder, 0);
        *coder_rep_slot_mut(coder, 0) = distance;
    }
    if len == 1 {
        (*coder).state = (if ((*coder).state as u32) < LIT_STATES {
            STATE_LIT_SHORTREP
        } else {
            STATE_NONLIT_REP
        }) as lzma_lzma_state;
    } else {
        length(
            ::core::ptr::addr_of_mut!((*coder).rc),
            ::core::ptr::addr_of_mut!((*coder).rep_len_encoder),
            pos_state,
            len,
            (*coder).fast_mode,
        );
        (*coder).state = (if ((*coder).state as u32) < LIT_STATES {
            STATE_LIT_LONGREP
        } else {
            STATE_NONLIT_REP
        }) as lzma_lzma_state;
    };
}
#[inline(always)]
unsafe fn encode_symbol(
    coder: *mut lzma_lzma1_encoder,
    mf: *mut lzma_mf,
    back: u32,
    len: u32,
    position: u32,
) {
    let pos_state: u32 = position & (*coder).pos_mask;
    if back == UINT32_MAX {
        rc_bit(
            ::core::ptr::addr_of_mut!((*coder).rc),
            is_match_prob(coder, pos_state),
            0,
        );
        literal(coder, mf, position);
    } else {
        rc_bit(
            ::core::ptr::addr_of_mut!((*coder).rc),
            is_match_prob(coder, pos_state),
            1,
        );
        if back < REPS {
            rc_bit(
                ::core::ptr::addr_of_mut!((*coder).rc),
                is_rep_prob(coder),
                1,
            );
            rep_match(coder, pos_state, back, len);
        } else {
            rc_bit(
                ::core::ptr::addr_of_mut!((*coder).rc),
                is_rep_prob(coder),
                0,
            );
            match_0(coder, pos_state, back - REPS, len);
        }
    }
    (*mf).read_ahead -= len;
}
unsafe fn encode_init(coder: *mut lzma_lzma1_encoder, mf: *mut lzma_mf) -> bool {
    if (*mf).read_pos == (*mf).read_limit {
        if (*mf).action == LZMA_RUN {
            return false;
        }
    } else {
        mf_skip(mf, 1);
        (*mf).read_ahead = 0;
        rc_bit(
            ::core::ptr::addr_of_mut!((*coder).rc),
            is_match_prob(coder, 0),
            0,
        );
        rc_bittree(
            ::core::ptr::addr_of_mut!((*coder).rc),
            ::core::ptr::addr_of_mut!((*coder).literal) as *mut probability,
            8,
            *(*mf).buffer as u32,
        );
        (*coder).uncomp_size += 1;
    }
    (*coder).is_initialized = true;
    true
}
unsafe fn encode_eopm(coder: *mut lzma_lzma1_encoder, position: u32) {
    let pos_state: u32 = position & (*coder).pos_mask;
    rc_bit(
        ::core::ptr::addr_of_mut!((*coder).rc),
        is_match_prob(coder, pos_state),
        1,
    );
    rc_bit(
        ::core::ptr::addr_of_mut!((*coder).rc),
        is_rep_prob(coder),
        0,
    );
    match_0(coder, pos_state, UINT32_MAX, MATCH_LEN_MIN);
}
pub const LOOP_INPUT_MAX: u32 = OPTS + 1;
#[cold]
#[inline(never)]
unsafe fn finish_lzma_stream(
    coder: *mut lzma_lzma1_encoder,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    let rc = ::core::ptr::addr_of_mut!((*coder).rc);
    if !(*coder).uncomp_size_ptr.is_null() {
        *(*coder).uncomp_size_ptr = (*coder).uncomp_size;
    }
    if (*coder).use_eopm {
        encode_eopm(coder, (*coder).uncomp_size as u32);
    }
    rc_flush(rc);
    if rc_encode(rc, out, out_pos, out_size) {
        (*coder).is_flushed = true;
        return LZMA_OK;
    }
    LZMA_STREAM_END
}
pub unsafe fn lzma_lzma_encode(
    coder: *mut lzma_lzma1_encoder,
    mf: *mut lzma_mf,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
    limit: u32,
) -> lzma_ret {
    let rc = ::core::ptr::addr_of_mut!((*coder).rc);
    if !(*coder).is_initialized && !encode_init(coder, mf) {
        return LZMA_OK;
    }
    if rc_encode(rc, out, out_pos, out_size) {
        return LZMA_OK;
    }
    if (*coder).is_flushed {
        return LZMA_STREAM_END;
    }
    loop {
        if limit != UINT32_MAX
            && ((*mf).read_pos - (*mf).read_ahead >= limit
                || (*out_pos as u64) + rc_pending(rc) >= (LZMA2_CHUNK_MAX - LOOP_INPUT_MAX) as u64)
        {
            break;
        }
        if (*mf).read_pos >= (*mf).read_limit {
            if (*mf).action == LZMA_RUN {
                return LZMA_OK;
            }
            if (*mf).read_ahead == 0 {
                break;
            }
        }
        let mut len: u32 = 0;
        let mut back: u32 = 0;
        if (*coder).fast_mode {
            lzma_lzma_optimum_fast(
                coder,
                mf,
                ::core::ptr::addr_of_mut!(back),
                ::core::ptr::addr_of_mut!(len),
            );
        } else {
            lzma_lzma_optimum_normal(
                coder,
                mf,
                ::core::ptr::addr_of_mut!(back),
                ::core::ptr::addr_of_mut!(len),
                (*coder).uncomp_size as u32,
            );
        }
        encode_symbol(coder, mf, back, len, (*coder).uncomp_size as u32);
        if (*coder).out_limit != 0 && rc_encode_dummy(rc, (*coder).out_limit) {
            rc_forget(rc);
            break;
        }
        (*coder).uncomp_size += len as u64;
        if rc_encode(rc, out, out_pos, out_size) {
            return LZMA_OK;
        }
    }
    finish_lzma_stream(coder, out, out_pos, out_size)
}
unsafe fn lzma_encode(
    coder: *mut c_void,
    mf: *mut lzma_mf,
    out: *mut u8,
    out_pos: *mut size_t,
    out_size: size_t,
) -> lzma_ret {
    if (*mf).action == LZMA_SYNC_FLUSH {
        return LZMA_OPTIONS_ERROR;
    }
    lzma_lzma_encode(
        coder as *mut lzma_lzma1_encoder,
        mf,
        out,
        out_pos,
        out_size,
        UINT32_MAX,
    )
}
unsafe fn lzma_lzma_set_out_limit(
    coder_ptr: *mut c_void,
    uncomp_size: *mut u64,
    out_limit: u64,
) -> lzma_ret {
    if out_limit < 6 {
        return LZMA_BUF_ERROR;
    }
    let coder: *mut lzma_lzma1_encoder = coder_ptr as *mut lzma_lzma1_encoder;
    (*coder).out_limit = out_limit;
    (*coder).uncomp_size_ptr = uncomp_size;
    (*coder).use_eopm = false;
    LZMA_OK
}
fn is_options_valid(options: *const lzma_options_lzma) -> bool {
    unsafe {
        is_lclppb_valid(options)
            && (*options).nice_len >= MATCH_LEN_MIN
            && (*options).nice_len <= MATCH_LEN_MAX
            && ((*options).mode == LZMA_MODE_FAST || (*options).mode == LZMA_MODE_NORMAL)
    }
}
fn set_lz_options(lz_options: *mut lzma_lz_options, options: *const lzma_options_lzma) {
    unsafe {
        (*lz_options).before_size = OPTS as size_t;
        (*lz_options).dict_size = (*options).dict_size as size_t;
        (*lz_options).after_size = LOOP_INPUT_MAX as size_t;
        (*lz_options).match_len_max = MATCH_LEN_MAX as size_t;
        (*lz_options).nice_len = (if mf_get_hash_bytes((*options).mf) > (*options).nice_len {
            mf_get_hash_bytes((*options).mf)
        } else {
            (*options).nice_len
        }) as size_t;
        (*lz_options).match_finder = (*options).mf;
        (*lz_options).depth = (*options).depth;
        (*lz_options).preset_dict = (*options).preset_dict;
        (*lz_options).preset_dict_size = (*options).preset_dict_size;
    }
}
unsafe fn length_encoder_reset(
    lencoder: *mut lzma_length_encoder,
    num_pos_states: u32,
    fast_mode: bool,
) {
    (*lencoder).choice = (RC_BIT_MODEL_TOTAL >> 1) as probability;
    (*lencoder).choice2 = (RC_BIT_MODEL_TOTAL >> 1) as probability;
    let mut pos_state: size_t = 0;
    while pos_state < num_pos_states as size_t {
        let low = length_low_probs(lencoder, pos_state as u32);
        let mid = length_mid_probs(lencoder, pos_state as u32);
        let mut bt_i: u32 = 0;
        while bt_i < (1 << 3) as u32 {
            *low.add(bt_i as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            bt_i += 1;
        }
        let mut bt_i_0: u32 = 0;
        while bt_i_0 < (1 << 3) as u32 {
            *mid.add(bt_i_0 as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            bt_i_0 += 1;
        }
        pos_state += 1;
    }
    let high = length_high_probs(lencoder);
    let mut bt_i_1: u32 = 0;
    while bt_i_1 < (1 << 8) as u32 {
        *high.add(bt_i_1 as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        bt_i_1 += 1;
    }
    if !fast_mode {
        let mut pos_state_0: u32 = 0;
        while pos_state_0 < num_pos_states {
            length_update_prices(lencoder, pos_state_0);
            pos_state_0 += 1;
        }
    }
}
pub unsafe fn lzma_lzma_encoder_reset(
    coder: *mut lzma_lzma1_encoder,
    options: *const lzma_options_lzma,
) -> lzma_ret {
    if !is_options_valid(options) {
        return LZMA_OPTIONS_ERROR;
    }
    (*coder).pos_mask = ((1u32 << (*options).pb) - 1) as u32;
    (*coder).literal_context_bits = (*options).lc;
    (*coder).literal_mask = (0x100u32 << (*options).lp) - (0x100 >> (*options).lc);
    rc_reset(::core::ptr::addr_of_mut!((*coder).rc));
    (*coder).state = STATE_LIT_LIT;
    let mut i: size_t = 0;
    while i < REPS as size_t {
        *coder_rep_slot_mut(coder, i as usize) = 0;
        i += 1;
    }
    literal_init(
        ::core::ptr::addr_of_mut!((*coder).literal) as *mut probability,
        (*options).lc,
        (*options).lp,
    );
    let mut i_0: size_t = 0;
    while i_0 < STATES as size_t {
        let is_match = encoder_is_match_row(coder, i_0 as u32);
        let is_rep0_long = encoder_is_rep0_long_row(coder, i_0 as u32);
        let mut j: size_t = 0;
        while j <= (*coder).pos_mask as size_t {
            *is_match.add(j as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            *is_rep0_long.add(j as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            j += 1;
        }
        *encoder_is_rep_slot(::core::ptr::addr_of_mut!((*coder).is_rep), i_0 as u32) =
            (RC_BIT_MODEL_TOTAL >> 1) as probability;
        *encoder_is_rep_slot(::core::ptr::addr_of_mut!((*coder).is_rep0), i_0 as u32) =
            (RC_BIT_MODEL_TOTAL >> 1) as probability;
        *encoder_is_rep_slot(::core::ptr::addr_of_mut!((*coder).is_rep1), i_0 as u32) =
            (RC_BIT_MODEL_TOTAL >> 1) as probability;
        *encoder_is_rep_slot(::core::ptr::addr_of_mut!((*coder).is_rep2), i_0 as u32) =
            (RC_BIT_MODEL_TOTAL >> 1) as probability;
        i_0 += 1;
    }
    let mut i_1: size_t = 0;
    while i_1 < (FULL_DISTANCES - DIST_MODEL_END) as size_t {
        *encoder_dist_special_prob(coder, i_1 as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        i_1 += 1;
    }
    let mut i_2: size_t = 0;
    while i_2 < DIST_STATES as size_t {
        let dist_slot = encoder_dist_slot_row(coder, i_2 as u32);
        let mut bt_i: u32 = 0;
        while bt_i < (1 << 6) as u32 {
            *dist_slot.add(bt_i as usize) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
            bt_i += 1;
        }
        i_2 += 1;
    }
    let mut bt_i_0: u32 = 0;
    while bt_i_0 < (1 << 4) as u32 {
        *encoder_dist_align_prob(coder, bt_i_0) = (RC_BIT_MODEL_TOTAL >> 1) as probability;
        bt_i_0 += 1;
    }
    length_encoder_reset(
        ::core::ptr::addr_of_mut!((*coder).match_len_encoder),
        1 << (*options).pb,
        (*coder).fast_mode,
    );
    length_encoder_reset(
        ::core::ptr::addr_of_mut!((*coder).rep_len_encoder),
        1 << (*options).pb,
        (*coder).fast_mode,
    );
    (*coder).match_price_count = UINT32_MAX / 2;
    (*coder).align_price_count = UINT32_MAX / 2;
    (*coder).opts_end_index = 0;
    (*coder).opts_current_index = 0;
    LZMA_OK
}
pub unsafe fn lzma_lzma_encoder_create(
    coder_ptr: *mut *mut c_void,
    allocator: *const lzma_allocator,
    id: lzma_vli,
    options: *const lzma_options_lzma,
    lz_options: *mut lzma_lz_options,
) -> lzma_ret {
    if (*coder_ptr).is_null() {
        *coder_ptr = crate::alloc::internal_alloc_object::<lzma_lzma1_encoder>(allocator).cast();
        if (*coder_ptr).is_null() {
            return LZMA_MEM_ERROR;
        }
    }
    let coder: *mut lzma_lzma1_encoder = *coder_ptr as *mut lzma_lzma1_encoder;
    match (*options).mode {
        1 => {
            (*coder).fast_mode = true;
        }
        2 => {
            (*coder).fast_mode = false;
            if (*options).dict_size > (1u32 << 30) + (1 << 29) {
                return LZMA_OPTIONS_ERROR;
            }
            let mut log_size: u32 = 0;
            while 1 << log_size < (*options).dict_size {
                log_size += 1;
            }
            (*coder).dist_table_size = log_size * 2;
            let nice_len: u32 = if mf_get_hash_bytes((*options).mf) > (*options).nice_len {
                mf_get_hash_bytes((*options).mf) as u32
            } else {
                (*options).nice_len
            };
            (*coder).match_len_encoder.table_size = nice_len + 1 - MATCH_LEN_MIN;
            (*coder).rep_len_encoder.table_size = nice_len + 1 - MATCH_LEN_MIN;
        }
        _ => return LZMA_OPTIONS_ERROR,
    }
    (*coder).is_initialized = !(*options).preset_dict.is_null() && (*options).preset_dict_size > 0;
    (*coder).is_flushed = false;
    (*coder).uncomp_size = 0;
    (*coder).uncomp_size_ptr = core::ptr::null_mut();
    (*coder).out_limit = 0;
    (*coder).use_eopm = id == LZMA_FILTER_LZMA1;
    if id == LZMA_FILTER_LZMA1EXT {
        if (*options).ext_flags & !(LZMA_LZMA1EXT_ALLOW_EOPM as u32) != 0 {
            return LZMA_OPTIONS_ERROR;
        }
        (*coder).use_eopm = (*options).ext_flags & LZMA_LZMA1EXT_ALLOW_EOPM as u32 != 0;
    }
    set_lz_options(lz_options, options);
    lzma_lzma_encoder_reset(coder, options)
}
unsafe fn lzma_encoder_end(coder_ptr: *mut c_void, allocator: *const lzma_allocator) {
    crate::alloc::internal_free(coder_ptr as *mut lzma_lzma1_encoder, allocator);
}
unsafe fn lzma_encoder_init(
    lz: *mut lzma_lz_encoder,
    allocator: *const lzma_allocator,
    id: lzma_vli,
    options: *const c_void,
    lz_options: *mut lzma_lz_options,
) -> lzma_ret {
    if options.is_null() {
        return LZMA_PROG_ERROR;
    }
    (*lz).code = lzma_encode as lzma_lz_encoder_code_function;
    (*lz).end = Some(lzma_encoder_end as unsafe fn(*mut c_void, *const lzma_allocator) -> ());
    (*lz).set_out_limit =
        Some(lzma_lzma_set_out_limit as unsafe fn(*mut c_void, *mut u64, u64) -> lzma_ret);
    lzma_lzma_encoder_create(
        ::core::ptr::addr_of_mut!((*lz).coder),
        allocator,
        id,
        options as *const lzma_options_lzma,
        lz_options,
    )
}
pub(crate) unsafe fn lzma_lzma_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter_info,
) -> lzma_ret {
    lzma_lz_encoder_init(
        next,
        allocator,
        filters,
        lzma_encoder_init
            as unsafe fn(
                *mut lzma_lz_encoder,
                *const lzma_allocator,
                lzma_vli,
                *const c_void,
                *mut lzma_lz_options,
            ) -> lzma_ret,
    )
}
pub(crate) unsafe fn lzma_lzma_encoder_memusage(options: *const c_void) -> u64 {
    if !is_options_valid(options as *const lzma_options_lzma) {
        return UINT64_MAX;
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
    set_lz_options(
        ::core::ptr::addr_of_mut!(lz_options),
        options as *const lzma_options_lzma,
    );
    let lz_memusage: u64 = lzma_lz_encoder_memusage(&lz_options) as u64;
    if lz_memusage == UINT64_MAX {
        return UINT64_MAX;
    }
    (core::mem::size_of::<lzma_lzma1_encoder>() as u64) + lz_memusage
}
pub unsafe fn lzma_lzma_lclppb_encode(options: *const lzma_options_lzma, byte: *mut u8) -> bool {
    if !is_lclppb_valid(options) {
        return true;
    }
    *byte = (((*options).pb * 5 + (*options).lp) * 9 + (*options).lc) as u8;
    false
}
pub(crate) unsafe fn lzma_lzma_props_encode(options: *const c_void, out: *mut u8) -> lzma_ret {
    if options.is_null() {
        return LZMA_PROG_ERROR;
    }
    let opt: *const lzma_options_lzma = options as *const lzma_options_lzma;
    if lzma_lzma_lclppb_encode(opt, out) {
        return LZMA_PROG_ERROR;
    }
    write32le(&mut *out.add(1).cast::<[u8; 4]>(), (*opt).dict_size);
    LZMA_OK
}
pub fn lzma_mode_is_supported(mode: lzma_mode) -> lzma_bool {
    (mode == LZMA_MODE_FAST || mode == LZMA_MODE_NORMAL) as lzma_bool
}
