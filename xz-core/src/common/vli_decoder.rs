use crate::types::*;

/// Decode a variable-length integer from `input`, starting at `*in_pos` and
/// advancing it by the number of bytes consumed.
///
/// `vli_pos` distinguishes the two modes the C API selects with a NULL
/// pointer. `None` is the single-call mode: the whole integer must be present,
/// and running out of input is a data error. `Some` carries a cursor across
/// calls, so running out of input is a normal stop.
pub fn lzma_vli_decode(
    vli: &mut lzma_vli,
    vli_pos: Option<&mut size_t>,
    input: &[u8],
    in_pos: &mut size_t,
) -> lzma_ret {
    let in_size = input.len();
    let single_call = vli_pos.is_none();
    let mut vli_pos_internal: size_t = 0;
    let vli_pos = vli_pos.unwrap_or(&mut vli_pos_internal);

    if single_call {
        *vli = 0;
        if *in_pos >= in_size {
            return LZMA_DATA_ERROR;
        }
    } else {
        if *vli_pos == 0 {
            *vli = 0;
        }
        if *vli_pos >= LZMA_VLI_BYTES_MAX as size_t || *vli >> (*vli_pos * 7) != 0 {
            return LZMA_PROG_ERROR;
        }
        if *in_pos >= in_size {
            return LZMA_BUF_ERROR;
        }
    }
    // The checks above guarantee the first read is in range, so `get` decides
    // the same thing the trailing `*in_pos >= in_size` test did, without
    // leaving a panicking index in the loop.
    while let Some(&byte) = input.get(*in_pos) {
        *in_pos += 1;
        *vli += ((byte & 0x7f) as lzma_vli) << (*vli_pos * 7);
        *vli_pos += 1;
        if byte & 0x80 == 0 {
            if byte == 0 && *vli_pos > 1 {
                return LZMA_DATA_ERROR;
            }
            return if single_call {
                LZMA_OK
            } else {
                LZMA_STREAM_END
            };
        }
        if *vli_pos == LZMA_VLI_BYTES_MAX as size_t {
            return LZMA_DATA_ERROR;
        }
    }
    if single_call {
        LZMA_DATA_ERROR
    } else {
        LZMA_OK
    }
}
