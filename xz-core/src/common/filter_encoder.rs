use crate::delta::delta_encoder::{lzma_delta_encoder_init, lzma_delta_props_encode};
use crate::lzma::lzma_encoder::lzma_lzma_props_encode;
use crate::lzma::lzma2_encoder::{
    lzma_lzma2_block_size, lzma_lzma2_encoder_init, lzma_lzma2_encoder_memusage,
    lzma_lzma2_props_encode,
};
use crate::simple::arm::lzma_simple_arm_encoder_init;
use crate::simple::arm64::lzma_simple_arm64_encoder_init;
use crate::simple::armthumb::lzma_simple_armthumb_encoder_init;
use crate::simple::ia64::lzma_simple_ia64_encoder_init;
use crate::simple::powerpc::lzma_simple_powerpc_encoder_init;
use crate::simple::riscv::lzma_simple_riscv_encoder_init;
use crate::simple::simple_encoder::{lzma_simple_props_encode, lzma_simple_props_size};
use crate::simple::sparc::lzma_simple_sparc_encoder_init;
use crate::simple::x86::lzma_simple_x86_encoder_init;
use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_filter_encoder {
    pub id: lzma_vli,
    pub init: lzma_init_function,
    pub memusage: Option<unsafe fn(*const c_void) -> u64>,
    pub block_size: Option<unsafe fn(*const c_void) -> u64>,
    pub props_size_get: Option<unsafe fn(*mut u32, *const c_void) -> lzma_ret>,
    pub props_size_fixed: u32,
    pub props_encode: Option<unsafe fn(*const c_void, *mut u8) -> lzma_ret>,
}
static encoders: [lzma_filter_encoder; 12] = [
    lzma_filter_encoder {
        id: LZMA_FILTER_LZMA1,
        init: Some(
            lzma_lzma_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: Some(lzma_lzma_encoder_memusage as unsafe fn(*const c_void) -> u64),
        block_size: None,
        props_size_get: None,
        props_size_fixed: 5,
        props_encode: Some(lzma_lzma_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_LZMA1EXT,
        init: Some(
            lzma_lzma_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: Some(lzma_lzma_encoder_memusage as unsafe fn(*const c_void) -> u64),
        block_size: None,
        props_size_get: None,
        props_size_fixed: 5,
        props_encode: Some(lzma_lzma_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_LZMA2,
        init: Some(
            lzma_lzma2_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: Some(lzma_lzma2_encoder_memusage as unsafe fn(*const c_void) -> u64),
        block_size: Some(lzma_lzma2_block_size as unsafe fn(*const c_void) -> u64),
        props_size_get: None,
        props_size_fixed: 1,
        props_encode: Some(
            lzma_lzma2_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret,
        ),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_X86,
        init: Some(
            lzma_simple_x86_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        block_size: None,
        props_size_get: Some(
            lzma_simple_props_size as unsafe fn(*mut u32, *const c_void) -> lzma_ret,
        ),
        props_size_fixed: 0,
        props_encode: Some(
            lzma_simple_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret,
        ),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_POWERPC,
        init: Some(
            lzma_simple_powerpc_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        block_size: None,
        props_size_get: Some(
            lzma_simple_props_size as unsafe fn(*mut u32, *const c_void) -> lzma_ret,
        ),
        props_size_fixed: 0,
        props_encode: Some(
            lzma_simple_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret,
        ),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_IA64,
        init: Some(
            lzma_simple_ia64_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        block_size: None,
        props_size_get: Some(
            lzma_simple_props_size as unsafe fn(*mut u32, *const c_void) -> lzma_ret,
        ),
        props_size_fixed: 0,
        props_encode: Some(
            lzma_simple_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret,
        ),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_ARM,
        init: Some(
            lzma_simple_arm_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        block_size: None,
        props_size_get: Some(
            lzma_simple_props_size as unsafe fn(*mut u32, *const c_void) -> lzma_ret,
        ),
        props_size_fixed: 0,
        props_encode: Some(
            lzma_simple_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret,
        ),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_ARMTHUMB,
        init: Some(
            lzma_simple_armthumb_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        block_size: None,
        props_size_get: Some(
            lzma_simple_props_size as unsafe fn(*mut u32, *const c_void) -> lzma_ret,
        ),
        props_size_fixed: 0,
        props_encode: Some(
            lzma_simple_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret,
        ),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_ARM64,
        init: Some(
            lzma_simple_arm64_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        block_size: None,
        props_size_get: Some(
            lzma_simple_props_size as unsafe fn(*mut u32, *const c_void) -> lzma_ret,
        ),
        props_size_fixed: 0,
        props_encode: Some(
            lzma_simple_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret,
        ),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_SPARC,
        init: Some(
            lzma_simple_sparc_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        block_size: None,
        props_size_get: Some(
            lzma_simple_props_size as unsafe fn(*mut u32, *const c_void) -> lzma_ret,
        ),
        props_size_fixed: 0,
        props_encode: Some(
            lzma_simple_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret,
        ),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_RISCV,
        init: Some(
            lzma_simple_riscv_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: None,
        block_size: None,
        props_size_get: Some(
            lzma_simple_props_size as unsafe fn(*mut u32, *const c_void) -> lzma_ret,
        ),
        props_size_fixed: 0,
        props_encode: Some(
            lzma_simple_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret,
        ),
    },
    lzma_filter_encoder {
        id: LZMA_FILTER_DELTA,
        init: Some(
            lzma_delta_encoder_init
                as unsafe fn(
                    *mut lzma_next_coder,
                    *const lzma_allocator,
                    *const lzma_filter_info,
                ) -> lzma_ret,
        ),
        memusage: Some(lzma_delta_coder_memusage as unsafe fn(*const c_void) -> u64),
        block_size: None,
        props_size_get: None,
        props_size_fixed: 1,
        props_encode: Some(
            lzma_delta_props_encode as unsafe fn(*const c_void, *mut u8) -> lzma_ret,
        ),
    },
];
#[inline(always)]
unsafe fn reversed_filter_slot(filters: *mut [lzma_filter; 5], index: usize) -> *mut lzma_filter {
    debug_assert!(index < 5);
    (filters as *mut lzma_filter).add(index)
}
#[inline(always)]
unsafe fn supported_action_slot(actions: *mut bool, index: lzma_action) -> *mut bool {
    debug_assert!((index as usize) < 5);
    actions.add(index as usize)
}
fn encoder_find(id: lzma_vli) -> *const lzma_filter_encoder {
    let encoders_ptr = encoders.as_ptr();
    let mut i: size_t = 0;
    while i < core::mem::size_of::<[lzma_filter_encoder; 12]>()
        / core::mem::size_of::<lzma_filter_encoder>()
    {
        let encoder = unsafe { encoders_ptr.add(i as usize) };
        if unsafe { (*encoder).id } == id {
            return encoder;
        }
        i += 1;
    }
    core::ptr::null()
}
unsafe fn coder_find(id: lzma_vli) -> *const lzma_filter_coder {
    encoder_find(id) as *const lzma_filter_coder
}
pub fn lzma_filter_encoder_is_supported(id: lzma_vli) -> lzma_bool {
    !encoder_find(id).is_null() as lzma_bool
}
pub unsafe fn lzma_filters_update(strm: *mut lzma_stream, filters: *const lzma_filter) -> lzma_ret {
    if (*(*strm).internal).next.update.is_none() {
        return LZMA_PROG_ERROR;
    }
    if lzma_raw_encoder_memusage(filters) == UINT64_MAX {
        return LZMA_OPTIONS_ERROR;
    }
    let mut count: size_t = 1;
    while (*filters.add(count)).id != LZMA_VLI_UNKNOWN {
        count += 1;
    }
    let mut reversed_filters: [lzma_filter; 5] = [lzma_filter {
        id: 0,
        options: core::ptr::null_mut(),
    }; 5];
    let mut i: size_t = 0;
    while i < count {
        *reversed_filter_slot(
            ::core::ptr::addr_of_mut!(reversed_filters),
            (count - i - 1) as usize,
        ) = *filters.add(i);
        i += 1;
    }
    (*reversed_filter_slot(::core::ptr::addr_of_mut!(reversed_filters), count as usize)).id =
        LZMA_VLI_UNKNOWN;
    debug_assert!((*(*strm).internal).next.update.is_some());
    let update = (*(*strm).internal).next.update.unwrap_unchecked();
    update(
        (*(*strm).internal).next.coder,
        crate::common::common::lzma_stream_allocator(strm),
        filters,
        ::core::ptr::addr_of_mut!(reversed_filters) as *mut lzma_filter,
    )
}
pub unsafe fn lzma_raw_encoder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    filters: *const lzma_filter,
) -> lzma_ret {
    lzma_raw_coder_init(
        next,
        allocator,
        filters,
        coder_find as unsafe fn(lzma_vli) -> *const lzma_filter_coder,
        true,
    )
}
pub unsafe fn lzma_raw_encoder(strm: *mut lzma_stream, filters: *const lzma_filter) -> lzma_ret {
    let ret: lzma_ret = lzma_strm_init(strm);
    if ret != LZMA_OK {
        return ret;
    }
    let ret: lzma_ret = lzma_raw_coder_init(
        ::core::ptr::addr_of_mut!((*(*strm).internal).next),
        crate::common::common::lzma_stream_allocator(strm),
        filters,
        coder_find as unsafe fn(lzma_vli) -> *const lzma_filter_coder,
        true,
    );
    if ret != LZMA_OK {
        lzma_end(strm);
        return ret;
    }
    *supported_action_slot(
        ::core::ptr::addr_of_mut!((*(*strm).internal).supported_actions) as *mut bool,
        LZMA_RUN,
    ) = true;
    *supported_action_slot(
        ::core::ptr::addr_of_mut!((*(*strm).internal).supported_actions) as *mut bool,
        LZMA_SYNC_FLUSH,
    ) = true;
    *supported_action_slot(
        ::core::ptr::addr_of_mut!((*(*strm).internal).supported_actions) as *mut bool,
        LZMA_FINISH,
    ) = true;
    LZMA_OK
}
pub unsafe fn lzma_raw_encoder_memusage(filters: *const lzma_filter) -> u64 {
    lzma_raw_coder_memusage(
        coder_find as unsafe fn(lzma_vli) -> *const lzma_filter_coder,
        filters,
    )
}
pub unsafe fn lzma_mt_block_size(filters: *const lzma_filter) -> u64 {
    if filters.is_null() {
        return UINT64_MAX;
    }
    let mut max: u64 = 0;
    let mut i: size_t = 0;
    while (*filters.add(i)).id != LZMA_VLI_UNKNOWN {
        let fe: *const lzma_filter_encoder =
            encoder_find((*filters.add(i)).id) as *const lzma_filter_encoder;
        if fe.is_null() {
            return UINT64_MAX;
        }
        if let Some(block_size) = (*fe).block_size {
            let size: u64 = block_size((*filters.add(i)).options) as u64;
            if size > max {
                max = size;
            }
        }
        i += 1;
    }
    if max == 0 { UINT64_MAX } else { max }
}
/// # Safety
/// `filter.options` is handed to the filter's own property encoder, which
/// reads it as the options struct `filter.id` names.
pub unsafe fn lzma_properties_size(size: &mut u32, filter: &lzma_filter) -> lzma_ret {
    let fe: *const lzma_filter_encoder = encoder_find(filter.id) as *const lzma_filter_encoder;
    if fe.is_null() {
        return if filter.id <= LZMA_VLI_MAX {
            LZMA_OPTIONS_ERROR
        } else {
            LZMA_PROG_ERROR
        };
    }
    if let Some(props_size_get) = (*fe).props_size_get {
        props_size_get(size, filter.options)
    } else {
        *size = (*fe).props_size_fixed;
        LZMA_OK
    }
}
/// `props` must be exactly the size [`lzma_properties_size`] reports for this
/// filter; the per-filter encoders write that many bytes without a length.
///
/// # Safety
/// Same `filter.options` contract as [`lzma_properties_size`].
pub unsafe fn lzma_properties_encode(filter: &lzma_filter, props: &mut [u8]) -> lzma_ret {
    let fe: *const lzma_filter_encoder = encoder_find(filter.id) as *const lzma_filter_encoder;
    if fe.is_null() {
        return LZMA_PROG_ERROR;
    }
    if let Some(props_encode) = (*fe).props_encode {
        props_encode(filter.options, props.as_mut_ptr())
    } else {
        LZMA_OK
    }
}
