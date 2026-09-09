use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct filter_features {
    pub id: lzma_vli,
    pub options_size: size_t,
    pub non_last_ok: bool,
    pub last_ok: bool,
    pub changes_size: bool,
}
static FEATURES: [filter_features; 13] = [
    filter_features {
        id: LZMA_FILTER_LZMA1,
        options_size: core::mem::size_of::<lzma_options_lzma>(),
        non_last_ok: false,
        last_ok: true,
        changes_size: true,
    },
    filter_features {
        id: LZMA_FILTER_LZMA1EXT,
        options_size: core::mem::size_of::<lzma_options_lzma>(),
        non_last_ok: false,
        last_ok: true,
        changes_size: true,
    },
    filter_features {
        id: LZMA_FILTER_LZMA2,
        options_size: core::mem::size_of::<lzma_options_lzma>(),
        non_last_ok: false,
        last_ok: true,
        changes_size: true,
    },
    filter_features {
        id: LZMA_FILTER_X86,
        options_size: core::mem::size_of::<lzma_options_bcj>(),
        non_last_ok: true,
        last_ok: false,
        changes_size: false,
    },
    filter_features {
        id: LZMA_FILTER_POWERPC,
        options_size: core::mem::size_of::<lzma_options_bcj>(),
        non_last_ok: true,
        last_ok: false,
        changes_size: false,
    },
    filter_features {
        id: LZMA_FILTER_IA64,
        options_size: core::mem::size_of::<lzma_options_bcj>(),
        non_last_ok: true,
        last_ok: false,
        changes_size: false,
    },
    filter_features {
        id: LZMA_FILTER_ARM,
        options_size: core::mem::size_of::<lzma_options_bcj>(),
        non_last_ok: true,
        last_ok: false,
        changes_size: false,
    },
    filter_features {
        id: LZMA_FILTER_ARMTHUMB,
        options_size: core::mem::size_of::<lzma_options_bcj>(),
        non_last_ok: true,
        last_ok: false,
        changes_size: false,
    },
    filter_features {
        id: LZMA_FILTER_ARM64,
        options_size: core::mem::size_of::<lzma_options_bcj>(),
        non_last_ok: true,
        last_ok: false,
        changes_size: false,
    },
    filter_features {
        id: LZMA_FILTER_SPARC,
        options_size: core::mem::size_of::<lzma_options_bcj>(),
        non_last_ok: true,
        last_ok: false,
        changes_size: false,
    },
    filter_features {
        id: LZMA_FILTER_RISCV,
        options_size: core::mem::size_of::<lzma_options_bcj>(),
        non_last_ok: true,
        last_ok: false,
        changes_size: false,
    },
    filter_features {
        id: LZMA_FILTER_DELTA,
        options_size: core::mem::size_of::<lzma_options_delta>(),
        non_last_ok: true,
        last_ok: false,
        changes_size: false,
    },
    filter_features {
        id: LZMA_VLI_UNKNOWN,
        options_size: 0,
        non_last_ok: false,
        last_ok: false,
        changes_size: false,
    },
];

fn filter_options_size(id: lzma_vli) -> Option<size_t> {
    let mut i: usize = 0;
    while i < FEATURES.len() {
        if FEATURES[i].id == id {
            return Some(FEATURES[i].options_size);
        }
        if FEATURES[i].id == LZMA_VLI_UNKNOWN {
            break;
        }
        i += 1;
    }
    None
}

pub(crate) unsafe fn lzma_filter_options_free(
    filter: lzma_filter,
    allocator: *const lzma_allocator,
) {
    if filter.options.is_null() {
        return;
    }
    if let Some(size) = filter_options_size(filter.id) {
        crate::alloc::internal_free_bytes(filter.options, size, allocator);
    } else {
        debug_assert!(false, "unknown filter options size");
        #[cfg(feature = "custom_allocator")]
        lzma_free(filter.options, allocator);
    }
}

pub unsafe fn lzma_filters_copy(
    src: *const lzma_filter,
    real_dest: *mut lzma_filter,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    if src.is_null() || real_dest.is_null() {
        return LZMA_PROG_ERROR;
    }
    let mut dest: [lzma_filter; 5] = [lzma_filter {
        id: 0,
        options: core::ptr::null_mut(),
    }; 5];
    let mut ret: lzma_ret = LZMA_OK;
    let mut i: size_t = 0;
    i = 0;
    's_15: loop {
        if (*src.add(i)).id == LZMA_VLI_UNKNOWN {
            break;
        }
        if i == LZMA_FILTERS_MAX as size_t {
            ret = LZMA_OPTIONS_ERROR;
            break 's_15;
        } else {
            dest[i as usize].id = (*src.add(i)).id;
            if (*src.add(i)).options.is_null() {
                dest[i as usize].options = core::ptr::null_mut();
            } else {
                let mut j: size_t = 0;
                j = 0;
                while (*src.add(i)).id != FEATURES[j as usize].id {
                    if FEATURES[j as usize].id == LZMA_VLI_UNKNOWN {
                        ret = LZMA_OPTIONS_ERROR;
                        break 's_15;
                    } else {
                        j += 1;
                    }
                }
                dest[i as usize].options = crate::alloc::internal_alloc_bytes(
                    FEATURES[j as usize].options_size,
                    allocator,
                );
                if dest[i as usize].options.is_null() {
                    ret = LZMA_MEM_ERROR;
                    break 's_15;
                } else {
                    core::ptr::copy_nonoverlapping(
                        (*src.add(i)).options as *const u8,
                        dest[i as usize].options as *mut u8,
                        FEATURES[j as usize].options_size,
                    );
                }
            }
            i += 1;
        }
    }
    if ret != LZMA_OK {
        while i > 0 {
            i -= 1;
            lzma_filter_options_free(dest[i as usize], allocator);
        }
        return ret;
    }
    dest[i as usize].id = LZMA_VLI_UNKNOWN;
    dest[i as usize].options = core::ptr::null_mut();
    core::ptr::copy_nonoverlapping(
        ::core::ptr::addr_of_mut!(dest) as *const u8,
        real_dest as *mut u8,
        (i + 1) * core::mem::size_of::<lzma_filter>(),
    );
    LZMA_OK
}
pub unsafe fn lzma_filters_free(filters: *mut lzma_filter, allocator: *const lzma_allocator) {
    if filters.is_null() {
        return;
    }
    let mut i: size_t = 0;
    while (*filters.add(i)).id != LZMA_VLI_UNKNOWN {
        if i == LZMA_FILTERS_MAX as size_t {
            break;
        }
        lzma_filter_options_free(*filters.add(i), allocator);
        (*filters.add(i)).options = core::ptr::null_mut();
        (*filters.add(i)).id = LZMA_VLI_UNKNOWN;
        i += 1;
    }
}
pub unsafe fn lzma_validate_chain(filters: *const lzma_filter, count: *mut size_t) -> lzma_ret {
    if filters.is_null() || (*filters).id == LZMA_VLI_UNKNOWN {
        return LZMA_PROG_ERROR;
    }
    let mut changes_size_count: size_t = 0;
    let mut non_last_ok: bool = true;
    let mut last_ok: bool = false;
    let mut i: size_t = 0;
    loop {
        let mut j: size_t = 0;
        j = 0;
        while (*filters.add(i)).id != FEATURES[j as usize].id {
            if FEATURES[j as usize].id == LZMA_VLI_UNKNOWN {
                return LZMA_OPTIONS_ERROR;
            }
            j += 1;
        }
        if !non_last_ok {
            return LZMA_OPTIONS_ERROR;
        }
        non_last_ok = FEATURES[j as usize].non_last_ok;
        last_ok = FEATURES[j as usize].last_ok;
        changes_size_count += FEATURES[j as usize].changes_size as size_t;
        i += 1;
        if (*filters.add(i)).id == LZMA_VLI_UNKNOWN {
            break;
        }
    }
    if i > LZMA_FILTERS_MAX as size_t || !last_ok || changes_size_count > 3 {
        return LZMA_OPTIONS_ERROR;
    }
    *count = i;
    LZMA_OK
}
pub unsafe fn lzma_raw_coder_init(
    next: *mut lzma_next_coder,
    allocator: *const lzma_allocator,
    options: *const lzma_filter,
    coder_find: unsafe fn(lzma_vli) -> *const lzma_filter_coder,
    is_encoder: bool,
) -> lzma_ret {
    let mut count: size_t = 0;
    let ret: lzma_ret = lzma_validate_chain(options, ::core::ptr::addr_of_mut!(count));
    if ret != LZMA_OK {
        return ret;
    }
    let mut filters: [lzma_filter_info; 5] = [lzma_filter_info_s {
        id: 0,
        init: None,
        options: core::ptr::null_mut(),
    }; 5];
    if is_encoder {
        let mut i: size_t = 0;
        while i < count {
            let j: size_t = count - i - 1;
            let fc: *const lzma_filter_coder =
                coder_find((*options.add(i)).id) as *const lzma_filter_coder;
            if fc.is_null() || (*fc).init.is_none() {
                return LZMA_OPTIONS_ERROR;
            }
            filters[j as usize].id = (*options.add(i)).id;
            filters[j as usize].init = (*fc).init;
            filters[j as usize].options = (*options.add(i)).options;
            i += 1;
        }
    } else {
        let mut i_0: size_t = 0;
        while i_0 < count {
            let fc_0: *const lzma_filter_coder =
                coder_find((*options.add(i_0)).id) as *const lzma_filter_coder;
            if fc_0.is_null() || (*fc_0).init.is_none() {
                return LZMA_OPTIONS_ERROR;
            }
            filters[i_0 as usize].id = (*options.add(i_0)).id;
            filters[i_0 as usize].init = (*fc_0).init;
            filters[i_0 as usize].options = (*options.add(i_0)).options;
            i_0 += 1;
        }
    }
    filters[count as usize].id = LZMA_VLI_UNKNOWN;
    filters[count as usize].init = None;
    let ret: lzma_ret = lzma_next_filter_init(
        next,
        allocator,
        ::core::ptr::addr_of_mut!(filters) as *mut lzma_filter_info,
    );
    if ret != LZMA_OK {
        lzma_next_end(next, allocator);
    }
    ret
}
pub unsafe fn lzma_raw_coder_memusage(
    coder_find: unsafe fn(lzma_vli) -> *const lzma_filter_coder,
    filters: *const lzma_filter,
) -> u64 {
    let mut tmp: size_t = 0;
    if lzma_validate_chain(filters, ::core::ptr::addr_of_mut!(tmp)) != LZMA_OK {
        return UINT64_MAX;
    }
    let mut total: u64 = 0;
    let mut i: size_t = 0;
    loop {
        let fc: *const lzma_filter_coder =
            coder_find((*filters.add(i)).id) as *const lzma_filter_coder;
        if fc.is_null() {
            return UINT64_MAX;
        }
        if let Some(memusage) = (*fc).memusage {
            let usage: u64 = memusage((*filters.add(i)).options) as u64;
            if usage == UINT64_MAX {
                return UINT64_MAX;
            }
            total += usage;
        } else {
            total += 1024;
        }
        i += 1;
        if (*filters.add(i)).id == LZMA_VLI_UNKNOWN {
            break;
        }
    }
    total + LZMA_MEMUSAGE_BASE
}
