use crate::types::*;

/// Encode a variable-length integer into `out`, starting at `*out_pos` and
/// advancing it by the number of bytes written.
///
/// `vli_pos` distinguishes the two modes the C API selects with a NULL
/// pointer. `None` is the single-call mode: the whole integer must fit in what
/// is left of `out`, and a short buffer is the caller's error. `Some` carries
/// a cursor across calls, so a full buffer is a normal stop.
pub fn lzma_vli_encode(
    mut vli: lzma_vli,
    vli_pos: Option<&mut size_t>,
    out: &mut [u8],
    out_pos: &mut size_t,
) -> lzma_ret {
    let out_size = out.len();
    let single_call = vli_pos.is_none();
    let mut vli_pos_internal: size_t = 0;
    let vli_pos = vli_pos.unwrap_or(&mut vli_pos_internal);

    if *out_pos >= out_size {
        return if single_call {
            LZMA_PROG_ERROR
        } else {
            LZMA_BUF_ERROR
        };
    }
    if *vli_pos >= LZMA_VLI_BYTES_MAX as size_t || vli > LZMA_VLI_MAX {
        return LZMA_PROG_ERROR;
    }
    vli >>= *vli_pos * 7;
    // The checks above keep `*out_pos` inside `out` on entry, and the loop
    // returns as soon as it reaches the end, so neither `get_mut` can fail.
    // Reporting a short buffer beats leaving a panicking index in the loop.
    while vli >= 0x80 {
        let Some(slot) = out.get_mut(*out_pos) else {
            return LZMA_PROG_ERROR;
        };
        *slot = vli as u8 | 0x80;
        *vli_pos += 1;
        vli >>= 7;
        *out_pos += 1;
        if *out_pos == out_size {
            return if single_call {
                LZMA_PROG_ERROR
            } else {
                LZMA_OK
            };
        }
    }
    let Some(slot) = out.get_mut(*out_pos) else {
        return LZMA_PROG_ERROR;
    };
    *slot = vli as u8;
    *out_pos += 1;
    *vli_pos += 1;
    if single_call {
        LZMA_OK
    } else {
        LZMA_STREAM_END
    }
}
