use crate::delta::delta_decoder::{lzma_delta_decoder_init, lzma_delta_props_decode};
use crate::lzma::lzma_decoder::{lzma_lzma_decoder_memusage, lzma_lzma_props_decode};
use crate::lzma::lzma2_decoder::{
    lzma_lzma2_decoder_init, lzma_lzma2_decoder_memusage, lzma_lzma2_props_decode,
};
use crate::simple::arm::lzma_simple_arm_decoder_init;
use crate::simple::arm64::lzma_simple_arm64_decoder_init;
use crate::simple::armthumb::lzma_simple_armthumb_decoder_init;
use crate::simple::ia64::lzma_simple_ia64_decoder_init;
use crate::simple::powerpc::lzma_simple_powerpc_decoder_init;
use crate::simple::riscv::lzma_simple_riscv_decoder_init;
use crate::simple::simple_decoder::lzma_simple_props_decode;
use crate::simple::sparc::lzma_simple_sparc_decoder_init;
use crate::simple::x86::lzma_simple_x86_decoder_init;
use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_filter_decoder {
    pub id: lzma_vli,
    pub init: lzma_init_function,
    pub memusage: Option<unsafe fn(*const c_void) -> u64>,
    pub props_decode:
        Option<unsafe fn(*mut *mut c_void, *const lzma_allocator, *const u8, size_t) -> lzma_ret>,
}
static decoders: [lzma_filter_decoder; 12] = [
    lzma_filter_decoder {
        id: LZMA_FILTER_LZMA1,
        init: Some(
            lzma_lzma_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: Some(lzma_lzma_decoder_memusage as unsafe fn(*const c_void) -> u64),
        props_decode: Some(
            lzma_lzma_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_LZMA1EXT,
        init: Some(
            lzma_lzma_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: Some(lzma_lzma_decoder_memusage as unsafe fn(*const c_void) -> u64),
        props_decode: Some(
            lzma_lzma_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_LZMA2,
        init: Some(
            lzma_lzma2_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: Some(lzma_lzma2_decoder_memusage as unsafe fn(*const c_void) -> u64),
        props_decode: Some(
            lzma_lzma2_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_X86,
        init: Some(
            lzma_simple_x86_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        props_decode: Some(
            lzma_simple_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_POWERPC,
        init: Some(
            lzma_simple_powerpc_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        props_decode: Some(
            lzma_simple_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_IA64,
        init: Some(
            lzma_simple_ia64_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        props_decode: Some(
            lzma_simple_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_ARM,
        init: Some(
            lzma_simple_arm_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        props_decode: Some(
            lzma_simple_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_ARMTHUMB,
        init: Some(
            lzma_simple_armthumb_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        props_decode: Some(
            lzma_simple_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_ARM64,
        init: Some(
            lzma_simple_arm64_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        props_decode: Some(
            lzma_simple_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_SPARC,
        init: Some(
            lzma_simple_sparc_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        props_decode: Some(
            lzma_simple_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_RISCV,
        init: Some(
            lzma_simple_riscv_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        props_decode: Some(
            lzma_simple_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
    lzma_filter_decoder {
        id: LZMA_FILTER_DELTA,
        init: Some(
            lzma_delta_decoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: Some(lzma_delta_coder_memusage as unsafe fn(*const c_void) -> u64),
        props_decode: Some(
            lzma_delta_props_decode
                as unsafe fn(
                    *mut *mut c_void,
                    *const lzma_allocator,
                    *const u8,
                    size_t,
                ) -> lzma_ret,
        ),
    },
];
fn decoder_find(id: lzma_vli) -> *const lzma_filter_decoder {
    let mut i: size_t = 0;
    while i < core::mem::size_of::<[lzma_filter_decoder; 12]>()
        / core::mem::size_of::<lzma_filter_decoder>()
    {
        if decoders[i as usize].id == id {
            return &decoders[i as usize];
        }
        i += 1;
    }
    core::ptr::null()
}
unsafe fn coder_find(id: lzma_vli) -> *const lzma_filter_coder {
    decoder_find(id) as *const lzma_filter_coder
}
pub fn lzma_filter_decoder_is_supported(id: lzma_vli) -> lzma_bool {
    !decoder_find(id).is_null() as lzma_bool
}
pub unsafe fn lzma_raw_decoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    options: *const lzma_filter,
) -> lzma_ret {
    lzma_raw_coder_init(
        next,
        allocator,
        options,
        coder_find as unsafe fn(lzma_vli) -> *const lzma_filter_coder,
        false,
    )
}
pub unsafe fn lzma_raw_decoder(strm: *mut lzma_stream, options: *const lzma_filter) -> lzma_ret {
    let ret_: lzma_ret = lzma_strm_init(strm);
    if ret_ != LZMA_OK {
        return ret_;
    }
    let ret__0: lzma_ret = lzma_raw_decoder_init(
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
pub unsafe fn lzma_raw_decoder_memusage(filters: *const lzma_filter) -> u64 {
    lzma_raw_coder_memusage(
        coder_find as unsafe fn(lzma_vli) -> *const lzma_filter_coder,
        filters,
    )
}
/// # Safety
/// On success `filter.options` owns an allocation from `allocator` that the
/// caller must free.
pub unsafe fn lzma_properties_decode(
    filter: &mut lzma_filter,
    allocator: *const lzma_allocator,
    props: &[u8],
) -> lzma_ret {
    filter.options = core::ptr::null_mut();
    let fd: *const lzma_filter_decoder = decoder_find(filter.id) as *const lzma_filter_decoder;
    if fd.is_null() {
        return LZMA_OPTIONS_ERROR;
    }
    if let Some(props_decode) = (*fd).props_decode {
        props_decode(
            &raw mut filter.options,
            allocator,
            props.as_ptr(),
            props.len(),
        )
    } else if props.is_empty() {
        LZMA_OK
    } else {
        LZMA_OPTIONS_ERROR
    }
}
