use crate::types::*;
pub const LZMA_LC_DEFAULT: u32 = 3;
pub const LZMA_LP_DEFAULT: u32 = 0;
pub const LZMA_PB_DEFAULT: u32 = 2;
pub const LZMA_PRESET_LEVEL_MASK: c_uint = 0x1f;
pub fn lzma_lzma_preset(options: &mut lzma_options_lzma, preset: u32) -> lzma_bool {
    let level: u32 = preset & LZMA_PRESET_LEVEL_MASK as u32;
    let flags: u32 = preset & !(LZMA_PRESET_LEVEL_MASK as u32);
    let supported_flags: u32 = LZMA_PRESET_EXTREME as u32;
    if level > 9 || flags & !supported_flags != 0 {
        return true as lzma_bool;
    }
    options.preset_dict = core::ptr::null();
    options.preset_dict_size = 0;
    options.lc = LZMA_LC_DEFAULT;
    options.lp = LZMA_LP_DEFAULT;
    options.pb = LZMA_PB_DEFAULT;
    const DICT_POW2: [u8; 10] = [18, 20, 21, 22, 22, 23, 23, 24, 25, 26];
    options.dict_size = 1u32 << DICT_POW2[level as usize];
    if level <= 3 {
        options.mode = LZMA_MODE_FAST;
        options.mf = (if level == 0 { LZMA_MF_HC3 } else { LZMA_MF_HC4 }) as lzma_match_finder;
        options.nice_len = (if level <= 1 { 128 } else { 273 }) as u32;
        const DEPTHS: [u8; 4] = [4, 8, 24, 48];
        options.depth = DEPTHS[level as usize] as u32;
    } else {
        options.mode = LZMA_MODE_NORMAL;
        options.mf = LZMA_MF_BT4;
        options.nice_len = (if level == 4 {
            16
        } else if level == 5 {
            32
        } else {
            64
        }) as u32;
        options.depth = 0;
    }
    if flags & LZMA_PRESET_EXTREME as u32 != 0 {
        options.mode = LZMA_MODE_NORMAL;
        options.mf = LZMA_MF_BT4;
        if level == 3 || level == 5 {
            options.nice_len = 192;
            options.depth = 0;
        } else {
            options.nice_len = 273;
            options.depth = 512;
        }
    }
    false as lzma_bool
}
