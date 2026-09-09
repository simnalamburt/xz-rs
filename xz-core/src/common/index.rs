use crate::types::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lzma_index_s {
    pub streams: index_tree,
    pub uncompressed_size: lzma_vli,
    pub total_size: lzma_vli,
    pub record_count: lzma_vli,
    pub index_list_size: lzma_vli,
    pub prealloc: size_t,
    pub checks: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct index_tree {
    pub root: *mut index_tree_node,
    pub leftmost: *mut index_tree_node,
    pub rightmost: *mut index_tree_node,
    pub count: u32,
}
pub type index_tree_node = index_tree_node_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct index_tree_node_s {
    pub uncompressed_base: lzma_vli,
    pub compressed_base: lzma_vli,
    pub parent: *mut index_tree_node,
    pub left: *mut index_tree_node,
    pub right: *mut index_tree_node,
}
pub type lzma_index = lzma_index_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct index_record {
    pub uncompressed_sum: lzma_vli,
    pub unpadded_sum: lzma_vli,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct index_group {
    pub node: index_tree_node,
    pub number_base: lzma_vli,
    pub allocated: size_t,
    pub last: size_t,
    pub records: [index_record; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct index_stream {
    pub node: index_tree_node,
    pub number: u32,
    pub block_number_base: lzma_vli,
    pub groups: index_tree,
    pub record_count: lzma_vli,
    pub index_list_size: lzma_vli,
    pub stream_flags: lzma_stream_flags,
    pub stream_padding: lzma_vli,
}
pub const ITER_METHOD_NORMAL: iter_method = 0;
pub const ITER_METHOD: iter_mode = 4;
pub const ITER_RECORD: iter_mode = 3;
pub const ITER_GROUP: iter_mode = 2;
pub const ITER_STREAM: iter_mode = 1;
pub const ITER_INDEX: iter_mode = 0;
pub const ITER_METHOD_LEFTMOST: iter_method = 2;
pub const ITER_METHOD_NEXT: iter_method = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct index_cat_info {
    pub uncompressed_size: lzma_vli,
    pub file_size: lzma_vli,
    pub block_number_add: lzma_vli,
    pub stream_number_add: u32,
    pub streams: *mut index_tree,
}
pub type iter_mode = c_uint;
pub type iter_method = c_uint;
#[inline]
fn bsr32(n: u32) -> u32 {
    n.leading_zeros() as i32 as u32 ^ 31
}
#[inline]
fn ctz32(n: u32) -> u32 {
    n.trailing_zeros() as i32 as u32
}
pub const INDEX_GROUP_SIZE: u32 = 512;
pub const PREALLOC_MAX: usize = (SIZE_MAX as usize)
    .wrapping_sub(core::mem::size_of::<index_group>())
    .wrapping_div(core::mem::size_of::<index_record>());
#[inline]
fn index_group_size(records: size_t) -> size_t {
    core::mem::size_of::<index_group>() + records * core::mem::size_of::<index_record>()
}
unsafe fn index_group_alloc(records: size_t, allocator: *const lzma_allocator) -> *mut index_group {
    crate::alloc::internal_alloc_array::<u8>(index_group_size(records), allocator)
        as *mut index_group
}
unsafe fn index_group_free(g: *mut index_group, allocator: *const lzma_allocator) {
    if !g.is_null() {
        crate::alloc::internal_free_array(
            g as *mut u8,
            index_group_size((*g).allocated),
            allocator,
        );
    }
}
unsafe fn index_tree_init(tree: *mut index_tree) {
    (*tree).root = core::ptr::null_mut();
    (*tree).leftmost = core::ptr::null_mut();
    (*tree).rightmost = core::ptr::null_mut();
    (*tree).count = 0;
}
unsafe fn index_tree_node_end(
    node: *mut index_tree_node,
    allocator: *const lzma_allocator,
    free_func: unsafe fn(*mut c_void, *const lzma_allocator) -> (),
) {
    if !(*node).left.is_null() {
        index_tree_node_end((*node).left, allocator, free_func);
    }
    if !(*node).right.is_null() {
        index_tree_node_end((*node).right, allocator, free_func);
    }
    free_func(node as *mut c_void, allocator);
}
unsafe fn index_tree_end(
    tree: *mut index_tree,
    allocator: *const lzma_allocator,
    free_func: unsafe fn(*mut c_void, *const lzma_allocator) -> (),
) {
    if !(*tree).root.is_null() {
        index_tree_node_end((*tree).root, allocator, free_func);
    }
}
unsafe fn index_group_node_free(node: *mut c_void, allocator: *const lzma_allocator) {
    index_group_free(node as *mut index_group, allocator);
}
unsafe fn index_tree_append(tree: *mut index_tree, mut node: *mut index_tree_node) {
    (*node).parent = (*tree).rightmost;
    (*node).left = core::ptr::null_mut();
    (*node).right = core::ptr::null_mut();
    (*tree).count += 1;
    if (*tree).root.is_null() {
        (*tree).root = node;
        (*tree).leftmost = node;
        (*tree).rightmost = node;
        return;
    }
    (*(*tree).rightmost).right = node;
    (*tree).rightmost = node;
    let mut up: u32 = (*tree).count ^ 1 << bsr32((*tree).count);
    if up != 0 {
        up = ctz32((*tree).count) + 2;
        loop {
            node = (*node).parent;
            up -= 1;
            if up == 0 {
                break;
            }
        }
        let pivot: *mut index_tree_node = (*node).right;
        if (*node).parent.is_null() {
            (*tree).root = pivot;
        } else {
            (*(*node).parent).right = pivot;
        }
        (*pivot).parent = (*node).parent;
        (*node).right = (*pivot).left;
        if !(*node).right.is_null() {
            (*(*node).right).parent = node;
        }
        (*pivot).left = node;
        (*node).parent = pivot;
    }
}
unsafe fn index_tree_next(mut node: *const index_tree_node) -> *mut c_void {
    if !(*node).right.is_null() {
        node = (*node).right;
        while !(*node).left.is_null() {
            node = (*node).left;
        }
        return node as *mut c_void;
    }
    while !(*node).parent.is_null() && (*(*node).parent).right == node as *mut index_tree_node {
        node = (*node).parent;
    }
    (*node).parent as *mut c_void
}
unsafe fn index_tree_locate(tree: *const index_tree, target: lzma_vli) -> *mut c_void {
    let mut result: *const index_tree_node = core::ptr::null();
    let mut node: *const index_tree_node = (*tree).root;
    while !node.is_null() {
        if (*node).uncompressed_base > target {
            node = (*node).left;
        } else {
            result = node;
            node = (*node).right;
        }
    }
    result as *mut c_void
}
unsafe fn index_stream_init(
    compressed_base: lzma_vli,
    uncompressed_base: lzma_vli,
    stream_number: u32,
    block_number_base: lzma_vli,
    allocator: *const lzma_allocator,
) -> *mut index_stream {
    let s: *mut index_stream = crate::alloc::internal_alloc_object::<index_stream>(allocator);
    if s.is_null() {
        return core::ptr::null_mut();
    }
    (*s).node.uncompressed_base = uncompressed_base;
    (*s).node.compressed_base = compressed_base;
    (*s).node.parent = core::ptr::null_mut();
    (*s).node.left = core::ptr::null_mut();
    (*s).node.right = core::ptr::null_mut();
    (*s).number = stream_number;
    (*s).block_number_base = block_number_base;
    index_tree_init(::core::ptr::addr_of_mut!((*s).groups));
    (*s).record_count = 0;
    (*s).index_list_size = 0;
    (*s).stream_flags.version = UINT32_MAX;
    (*s).stream_padding = 0;
    s
}
unsafe fn index_stream_end(node: *mut c_void, allocator: *const lzma_allocator) {
    let s: *mut index_stream = node as *mut index_stream;
    index_tree_end(
        ::core::ptr::addr_of_mut!((*s).groups),
        allocator,
        index_group_node_free as unsafe fn(*mut c_void, *const lzma_allocator) -> (),
    );
    crate::alloc::internal_free(s, allocator);
}
unsafe fn index_init_plain(allocator: *const lzma_allocator) -> *mut lzma_index {
    let i: *mut lzma_index = crate::alloc::internal_alloc_object::<lzma_index>(allocator);
    if !i.is_null() {
        index_tree_init(::core::ptr::addr_of_mut!((*i).streams));
        (*i).uncompressed_size = 0;
        (*i).total_size = 0;
        (*i).record_count = 0;
        (*i).index_list_size = 0;
        (*i).prealloc = INDEX_GROUP_SIZE as size_t;
        (*i).checks = 0;
    }
    i
}
pub unsafe fn lzma_index_init(allocator: *const lzma_allocator) -> *mut lzma_index {
    let i: *mut lzma_index = index_init_plain(allocator);
    if i.is_null() {
        return core::ptr::null_mut();
    }
    let s: *mut index_stream = index_stream_init(0, 0, 1, 0, allocator);
    if s.is_null() {
        crate::alloc::internal_free(i, allocator);
        return core::ptr::null_mut();
    }
    index_tree_append(
        ::core::ptr::addr_of_mut!((*i).streams),
        ::core::ptr::addr_of_mut!((*s).node),
    );
    i
}
pub unsafe fn lzma_index_end(i: *mut lzma_index, allocator: *const lzma_allocator) {
    if !i.is_null() {
        index_tree_end(
            ::core::ptr::addr_of_mut!((*i).streams),
            allocator,
            index_stream_end as unsafe fn(*mut c_void, *const lzma_allocator) -> (),
        );
        crate::alloc::internal_free(i, allocator);
    }
}
pub fn lzma_index_prealloc(i: &mut lzma_index, mut records: lzma_vli) {
    if records > PREALLOC_MAX as lzma_vli {
        records = PREALLOC_MAX as lzma_vli;
    }

    // If index_decoder.c calls us with records == 0, it's decoding
    // an Index that has no Records. In that case the decoder won't call
    // lzma_index_append() at all, and i->prealloc isn't used during
    // the Index decoding either.
    //
    // Normally the first lzma_index_append() call from the Index decoder
    // would reset i->prealloc to INDEX_GROUP_SIZE. With no Records,
    // lzma_index_append() isn't called and the resetting of prealloc
    // won't occur either. Thus, if records == 0, use the default value
    // INDEX_GROUP_SIZE instead.
    //
    // NOTE: lzma_index_append() assumes i->prealloc > 0. liblzma <= 5.8.2
    // didn't have this check and could set i->prealloc = 0, which would
    // result in a buffer overflow if the application called
    // lzma_index_append() after decoding an empty Index. Appending
    // Records after decoding an Index is a rare thing to do, but
    // it is supposed to work.
    if records == 0 {
        records = INDEX_GROUP_SIZE as lzma_vli;
    }

    i.prealloc = records as size_t;
}
pub fn lzma_index_memusage(streams: lzma_vli, blocks: lzma_vli) -> u64 {
    let alloc_overhead: size_t = (4_usize).wrapping_mul(core::mem::size_of::<*mut c_void>());
    let stream_base: size_t = (core::mem::size_of::<index_stream>())
        .wrapping_add(core::mem::size_of::<index_group>())
        .wrapping_add((2_usize).wrapping_mul(alloc_overhead));
    let group_base: size_t = (core::mem::size_of::<index_group>())
        .wrapping_add(
            (INDEX_GROUP_SIZE as size_t).wrapping_mul(core::mem::size_of::<index_record>()),
        )
        .wrapping_add(alloc_overhead);
    let groups: lzma_vli = blocks
        .wrapping_add(INDEX_GROUP_SIZE as lzma_vli)
        .wrapping_sub(1)
        .wrapping_div(INDEX_GROUP_SIZE as lzma_vli);
    let streams_mem: u64 = (streams as u64).wrapping_mul(stream_base as u64);
    let groups_mem: u64 = (groups as u64).wrapping_mul(group_base as u64);
    let index_base: u64 =
        (core::mem::size_of::<lzma_index>()).wrapping_add(alloc_overhead as usize) as u64;
    let limit: u64 = (UINT64_MAX).wrapping_sub(index_base);
    if streams == 0
        || streams > UINT32_MAX as lzma_vli
        || blocks > LZMA_VLI_MAX
        || streams > limit.wrapping_div(stream_base as u64)
        || groups > limit.wrapping_div(group_base as u64)
        || limit.wrapping_sub(streams_mem) < groups_mem
    {
        return UINT64_MAX;
    }
    index_base
        .wrapping_add(streams_mem)
        .wrapping_add(groups_mem)
}
pub fn lzma_index_memused(i: &lzma_index) -> u64 {
    lzma_index_memusage(i.streams.count as lzma_vli, i.record_count)
}
pub fn lzma_index_block_count(i: &lzma_index) -> lzma_vli {
    i.record_count
}
pub fn lzma_index_stream_count(i: &lzma_index) -> lzma_vli {
    i.streams.count as lzma_vli
}
pub fn lzma_index_size(i: &lzma_index) -> lzma_vli {
    index_size(i.record_count, i.index_list_size)
}
pub fn lzma_index_total_size(i: &lzma_index) -> lzma_vli {
    i.total_size
}
pub fn lzma_index_stream_size(i: &lzma_index) -> lzma_vli {
    (LZMA_STREAM_HEADER_SIZE as lzma_vli)
        + i.total_size
        + index_size(i.record_count, i.index_list_size)
        + LZMA_STREAM_HEADER_SIZE as lzma_vli
}
unsafe fn index_file_size(
    compressed_base: lzma_vli,
    unpadded_sum: lzma_vli,
    record_count: lzma_vli,
    index_list_size: lzma_vli,
    stream_padding: lzma_vli,
) -> lzma_vli {
    let mut file_size: lzma_vli = compressed_base
        .wrapping_add((2 * LZMA_STREAM_HEADER_SIZE) as lzma_vli)
        .wrapping_add(stream_padding)
        .wrapping_add(vli_ceil4(unpadded_sum));
    if file_size > LZMA_VLI_MAX {
        return LZMA_VLI_UNKNOWN;
    }
    file_size = file_size.wrapping_add(index_size(record_count, index_list_size));
    if file_size > LZMA_VLI_MAX {
        return LZMA_VLI_UNKNOWN;
    }
    file_size
}
/// # Safety
/// `i`'s stream tree must be intact: `streams.rightmost` and the groups it
/// reaches are followed as raw pointers. Holding a `&lzma_index` proves the
/// handle itself is live, not that its `pub` tree fields still point at the
/// nodes the index allocated.
pub unsafe fn lzma_index_file_size(i: &lzma_index) -> lzma_vli {
    let s: *const index_stream = i.streams.rightmost as *const index_stream;
    let g: *const index_group = (*s).groups.rightmost as *const index_group;
    index_file_size(
        (*s).node.compressed_base,
        if g.is_null() {
            0
        } else {
            (*(::core::ptr::addr_of!((*g).records) as *const index_record).add((*g).last))
                .unpadded_sum
        },
        (*s).record_count,
        (*s).index_list_size,
        (*s).stream_padding,
    )
}
pub fn lzma_index_uncompressed_size(i: &lzma_index) -> lzma_vli {
    i.uncompressed_size
}
/// # Safety
/// Same tree invariant as [`lzma_index_file_size`].
pub unsafe fn lzma_index_checks(i: &lzma_index) -> u32 {
    let mut checks: u32 = i.checks;
    let s: *const index_stream = i.streams.rightmost as *const index_stream;
    if (*s).stream_flags.version != UINT32_MAX {
        checks = (checks | 1u32 << (*s).stream_flags.check) as u32;
    }
    checks
}
pub fn lzma_index_padding_size(i: &lzma_index) -> u32 {
    ((4_u64).wrapping_sub(index_size_unpadded(i.record_count, i.index_list_size)) & 3) as u32
}
pub unsafe fn lzma_index_stream_flags(
    i: *mut lzma_index,
    stream_flags: *const lzma_stream_flags,
) -> lzma_ret {
    if i.is_null() || stream_flags.is_null() {
        return LZMA_PROG_ERROR;
    }
    let ret: lzma_ret = lzma_stream_flags_compare(&*stream_flags, &*stream_flags);
    if ret != LZMA_OK {
        return ret;
    }
    let s: *mut index_stream = (*i).streams.rightmost as *mut index_stream;
    (*s).stream_flags = *stream_flags;
    LZMA_OK
}
pub unsafe fn lzma_index_stream_padding(i: *mut lzma_index, stream_padding: lzma_vli) -> lzma_ret {
    if i.is_null() || stream_padding > LZMA_VLI_MAX || stream_padding & 3 != 0 {
        return LZMA_PROG_ERROR;
    }
    let s: *mut index_stream = (*i).streams.rightmost as *mut index_stream;
    let old_stream_padding: lzma_vli = (*s).stream_padding;
    (*s).stream_padding = 0;
    if lzma_index_file_size(&*i).wrapping_add(stream_padding) > LZMA_VLI_MAX {
        (*s).stream_padding = old_stream_padding;
        return LZMA_DATA_ERROR;
    }
    (*s).stream_padding = stream_padding;
    LZMA_OK
}
pub unsafe fn lzma_index_append(
    i: *mut lzma_index,
    allocator: *const lzma_allocator,
    unpadded_size: lzma_vli,
    uncompressed_size: lzma_vli,
) -> lzma_ret {
    if i.is_null()
        || unpadded_size < UNPADDED_SIZE_MIN
        || unpadded_size > UNPADDED_SIZE_MAX
        || uncompressed_size > LZMA_VLI_MAX
    {
        return LZMA_PROG_ERROR;
    }
    let s: *mut index_stream = (*i).streams.rightmost as *mut index_stream;
    let mut g: *mut index_group = (*s).groups.rightmost as *mut index_group;
    let compressed_base: lzma_vli = if g.is_null() {
        0
    } else {
        vli_ceil4(
            (*(::core::ptr::addr_of_mut!((*g).records) as *mut index_record).add((*g).last))
                .unpadded_sum,
        ) as lzma_vli
    };
    let uncompressed_base: lzma_vli = if g.is_null() {
        0
    } else {
        (*(::core::ptr::addr_of_mut!((*g).records) as *mut index_record).add((*g).last))
            .uncompressed_sum
    };
    let index_list_size_add: u32 =
        lzma_vli_size(unpadded_size) as u32 + lzma_vli_size(uncompressed_size) as u32;
    if uncompressed_base.wrapping_add(uncompressed_size) > LZMA_VLI_MAX {
        return LZMA_DATA_ERROR;
    }
    if compressed_base.wrapping_add(unpadded_size) > UNPADDED_SIZE_MAX {
        return LZMA_DATA_ERROR;
    }
    if index_file_size(
        (*s).node.compressed_base,
        compressed_base.wrapping_add(unpadded_size),
        (*s).record_count.wrapping_add(1),
        (*s).index_list_size
            .wrapping_add(index_list_size_add as lzma_vli),
        (*s).stream_padding,
    ) == LZMA_VLI_UNKNOWN
    {
        return LZMA_DATA_ERROR;
    }
    if index_size(
        (*i).record_count.wrapping_add(1),
        (*i).index_list_size
            .wrapping_add(index_list_size_add as lzma_vli),
    ) > LZMA_BACKWARD_SIZE_MAX
    {
        return LZMA_DATA_ERROR;
    }
    if !g.is_null() && (*g).last + 1 < (*g).allocated {
        (*g).last += 1;
    } else {
        debug_assert!((*i).prealloc > 0);
        g = index_group_alloc((*i).prealloc, allocator);
        if g.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*g).last = 0;
        (*g).allocated = (*i).prealloc;
        (*i).prealloc = INDEX_GROUP_SIZE as size_t;
        (*g).node.uncompressed_base = uncompressed_base;
        (*g).node.compressed_base = compressed_base;
        (*g).number_base = (*s).record_count + 1;
        index_tree_append(
            ::core::ptr::addr_of_mut!((*s).groups),
            ::core::ptr::addr_of_mut!((*g).node),
        );
    }
    (*(::core::ptr::addr_of_mut!((*g).records) as *mut index_record).add((*g).last))
        .uncompressed_sum = uncompressed_base + uncompressed_size;
    (*(::core::ptr::addr_of_mut!((*g).records) as *mut index_record).add((*g).last)).unpadded_sum =
        compressed_base + unpadded_size;
    (*s).record_count += 1;
    (*s).index_list_size += index_list_size_add as lzma_vli;
    (*i).total_size += vli_ceil4(unpadded_size);
    (*i).uncompressed_size += uncompressed_size;
    (*i).record_count += 1;
    (*i).index_list_size += index_list_size_add as lzma_vli;
    LZMA_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_prealloc_zero_keeps_append_usable() {
        unsafe {
            let allocator = core::ptr::null();
            let index = lzma_index_init(allocator);
            assert!(!index.is_null());

            lzma_index_prealloc(&mut *index, 0);
            assert_eq!((*index).prealloc, INDEX_GROUP_SIZE as size_t);

            let ret = lzma_index_append(index, allocator, UNPADDED_SIZE_MIN, 0);
            assert_eq!(ret, LZMA_OK);

            lzma_index_end(index, allocator);
        }
    }
}
unsafe fn index_cat_helper(info: *const index_cat_info, this: *mut index_stream) {
    let left: *mut index_stream = (*this).node.left as *mut index_stream;
    let right: *mut index_stream = (*this).node.right as *mut index_stream;
    if !left.is_null() {
        index_cat_helper(info, left);
    }
    (*this).node.uncompressed_base += (*info).uncompressed_size;
    (*this).node.compressed_base += (*info).file_size;
    (*this).number += (*info).stream_number_add;
    (*this).block_number_base += (*info).block_number_add;
    index_tree_append((*info).streams, ::core::ptr::addr_of_mut!((*this).node));
    if !right.is_null() {
        index_cat_helper(info, right);
    }
}
pub unsafe fn lzma_index_cat(
    dest: *mut lzma_index,
    src: *mut lzma_index,
    allocator: *const lzma_allocator,
) -> lzma_ret {
    if dest.is_null() || src.is_null() {
        return LZMA_PROG_ERROR;
    }
    let dest_file_size: lzma_vli = lzma_index_file_size(&*dest);
    if dest_file_size.wrapping_add(lzma_index_file_size(&*src)) > LZMA_VLI_MAX
        || (*dest)
            .uncompressed_size
            .wrapping_add((*src).uncompressed_size)
            > LZMA_VLI_MAX
    {
        return LZMA_DATA_ERROR;
    }
    let dest_size: lzma_vli =
        index_size_unpadded((*dest).record_count, (*dest).index_list_size) as lzma_vli;
    let src_size: lzma_vli =
        index_size_unpadded((*src).record_count, (*src).index_list_size) as lzma_vli;
    if vli_ceil4(dest_size.wrapping_add(src_size)) > LZMA_BACKWARD_SIZE_MAX {
        return LZMA_DATA_ERROR;
    }
    let s: *mut index_stream = (*dest).streams.rightmost as *mut index_stream;
    let g: *mut index_group = (*s).groups.rightmost as *mut index_group;
    if !g.is_null() && (*g).last + 1 < (*g).allocated {
        let newg: *mut index_group = index_group_alloc((*g).last + 1, allocator);
        if newg.is_null() {
            return LZMA_MEM_ERROR;
        }
        (*newg).node = (*g).node;
        (*newg).allocated = (*g).last + 1;
        (*newg).last = (*g).last;
        (*newg).number_base = (*g).number_base;
        core::ptr::copy_nonoverlapping(
            ::core::ptr::addr_of_mut!((*g).records) as *const u8,
            ::core::ptr::addr_of_mut!((*newg).records) as *mut u8,
            (*newg).allocated * core::mem::size_of::<index_record>(),
        );
        if !(*g).node.parent.is_null() {
            (*(*g).node.parent).right = ::core::ptr::addr_of_mut!((*newg).node);
        }
        if (*s).groups.leftmost == ::core::ptr::addr_of_mut!((*g).node) {
            (*s).groups.leftmost = ::core::ptr::addr_of_mut!((*newg).node);
            (*s).groups.root = ::core::ptr::addr_of_mut!((*newg).node);
        }
        (*s).groups.rightmost = ::core::ptr::addr_of_mut!((*newg).node);
        index_group_free(g, allocator);
    }
    (*dest).checks = lzma_index_checks(&*dest);
    let info: index_cat_info = index_cat_info {
        uncompressed_size: (*dest).uncompressed_size,
        file_size: dest_file_size,
        block_number_add: (*dest).record_count,
        stream_number_add: (*dest).streams.count,
        streams: ::core::ptr::addr_of_mut!((*dest).streams),
    };
    index_cat_helper(
        ::core::ptr::addr_of!(info),
        (*src).streams.root as *mut index_stream,
    );
    (*dest).uncompressed_size += (*src).uncompressed_size;
    (*dest).total_size += (*src).total_size;
    (*dest).record_count += (*src).record_count;
    (*dest).index_list_size += (*src).index_list_size;
    (*dest).checks |= (*src).checks;
    crate::alloc::internal_free(src, allocator);
    LZMA_OK
}
unsafe fn index_dup_stream(
    src: *const index_stream,
    allocator: *const lzma_allocator,
) -> *mut index_stream {
    if (*src).record_count > PREALLOC_MAX as lzma_vli {
        return core::ptr::null_mut();
    }
    let dest: *mut index_stream = index_stream_init(
        (*src).node.compressed_base,
        (*src).node.uncompressed_base,
        (*src).number,
        (*src).block_number_base,
        allocator,
    );
    if dest.is_null() {
        return core::ptr::null_mut();
    }
    (*dest).record_count = (*src).record_count;
    (*dest).index_list_size = (*src).index_list_size;
    (*dest).stream_flags = (*src).stream_flags;
    (*dest).stream_padding = (*src).stream_padding;
    if (*src).groups.leftmost.is_null() {
        return dest;
    }
    let destg: *mut index_group = index_group_alloc((*src).record_count as size_t, allocator);
    if destg.is_null() {
        index_stream_end(dest as *mut c_void, allocator);
        return core::ptr::null_mut();
    }
    (*destg).node.uncompressed_base = 0;
    (*destg).node.compressed_base = 0;
    (*destg).number_base = 1;
    (*destg).allocated = (*src).record_count as size_t;
    (*destg).last = ((*src).record_count - 1) as size_t;
    let mut srcg: *const index_group = (*src).groups.leftmost as *const index_group;
    let mut i: size_t = 0;
    loop {
        core::ptr::copy_nonoverlapping(
            ::core::ptr::addr_of!((*srcg).records) as *const u8,
            (::core::ptr::addr_of_mut!((*destg).records) as *mut index_record).add(i) as *mut u8,
            ((*srcg).last + 1) * core::mem::size_of::<index_record>(),
        );
        i += (*srcg).last + 1;
        srcg = index_tree_next(::core::ptr::addr_of!((*srcg).node)) as *const index_group;
        if srcg.is_null() {
            break;
        }
    }
    index_tree_append(
        ::core::ptr::addr_of_mut!((*dest).groups),
        ::core::ptr::addr_of_mut!((*destg).node),
    );
    dest
}
pub unsafe fn lzma_index_dup(
    src: *const lzma_index,
    allocator: *const lzma_allocator,
) -> *mut lzma_index {
    let dest: *mut lzma_index = index_init_plain(allocator);
    if dest.is_null() {
        return core::ptr::null_mut();
    }
    (*dest).uncompressed_size = (*src).uncompressed_size;
    (*dest).total_size = (*src).total_size;
    (*dest).record_count = (*src).record_count;
    (*dest).index_list_size = (*src).index_list_size;
    let mut srcstream: *const index_stream = (*src).streams.leftmost as *const index_stream;
    loop {
        let deststream: *mut index_stream = index_dup_stream(srcstream, allocator);
        if deststream.is_null() {
            lzma_index_end(dest, allocator);
            return core::ptr::null_mut();
        }
        index_tree_append(
            ::core::ptr::addr_of_mut!((*dest).streams),
            ::core::ptr::addr_of_mut!((*deststream).node),
        );
        srcstream =
            index_tree_next(::core::ptr::addr_of!((*srcstream).node)) as *const index_stream;
        if srcstream.is_null() {
            break;
        }
    }
    dest
}
unsafe fn iter_set_info(iter: &mut lzma_index_iter) {
    let i: *const lzma_index = iter.internal[ITER_INDEX as usize].p as *const lzma_index;
    let stream: *const index_stream = iter.internal[ITER_STREAM as usize].p as *const index_stream;
    let group: *const index_group = iter.internal[ITER_GROUP as usize].p as *const index_group;
    let record: size_t = iter.internal[ITER_RECORD as usize].s;
    if group.is_null() {
        iter.internal[ITER_METHOD as usize].s = ITER_METHOD_LEFTMOST as size_t;
    } else if (*i).streams.rightmost
        != ::core::ptr::addr_of!((*stream).node) as *mut index_tree_node
        || (*stream).groups.rightmost
            != ::core::ptr::addr_of!((*group).node) as *mut index_tree_node
    {
        iter.internal[ITER_METHOD as usize].s = ITER_METHOD_NORMAL as size_t;
    } else if (*stream).groups.leftmost
        != ::core::ptr::addr_of!((*group).node) as *mut index_tree_node
    {
        iter.internal[ITER_METHOD as usize].s = ITER_METHOD_NEXT as size_t;
        iter.internal[ITER_GROUP as usize].p = (*group).node.parent as *const c_void;
    } else {
        iter.internal[ITER_METHOD as usize].s = ITER_METHOD_LEFTMOST as size_t;
        iter.internal[ITER_GROUP as usize].p = core::ptr::null();
    }
    iter.stream.number = (*stream).number as lzma_vli;
    iter.stream.block_count = (*stream).record_count;
    iter.stream.compressed_offset = (*stream).node.compressed_base;
    iter.stream.uncompressed_offset = (*stream).node.uncompressed_base;
    iter.stream.flags = if (*stream).stream_flags.version == UINT32_MAX {
        core::ptr::null()
    } else {
        ::core::ptr::addr_of!((*stream).stream_flags)
    };
    iter.stream.padding = (*stream).stream_padding;
    if (*stream).groups.rightmost.is_null() {
        iter.stream.compressed_size = index_size(0, 0) + (2 * LZMA_STREAM_HEADER_SIZE) as lzma_vli;
        iter.stream.uncompressed_size = 0;
    } else {
        let g: *const index_group = (*stream).groups.rightmost as *const index_group;
        iter.stream.compressed_size = (2 * LZMA_STREAM_HEADER_SIZE) as lzma_vli
            + index_size((*stream).record_count, (*stream).index_list_size)
            + vli_ceil4(
                (*(::core::ptr::addr_of!((*g).records) as *const index_record).add((*g).last))
                    .unpadded_sum,
            );
        iter.stream.uncompressed_size =
            (*(::core::ptr::addr_of!((*g).records) as *const index_record).add((*g).last))
                .uncompressed_sum;
    }
    if !group.is_null() {
        iter.block.number_in_stream = (*group).number_base + record as lzma_vli;
        iter.block.number_in_file = iter.block.number_in_stream + (*stream).block_number_base;
        iter.block.compressed_stream_offset = if record == 0 {
            (*group).node.compressed_base
        } else {
            vli_ceil4(
                (*(::core::ptr::addr_of!((*group).records) as *const index_record).add(record - 1))
                    .unpadded_sum,
            )
        };
        iter.block.uncompressed_stream_offset = if record == 0 {
            (*group).node.uncompressed_base
        } else {
            (*(::core::ptr::addr_of!((*group).records) as *const index_record).add(record - 1))
                .uncompressed_sum
        };
        iter.block.uncompressed_size =
            (*(::core::ptr::addr_of!((*group).records) as *const index_record).add(record))
                .uncompressed_sum
                - iter.block.uncompressed_stream_offset;
        iter.block.unpadded_size =
            (*(::core::ptr::addr_of!((*group).records) as *const index_record).add(record))
                .unpadded_sum
                - iter.block.compressed_stream_offset;
        iter.block.total_size = vli_ceil4(iter.block.unpadded_size);
        iter.block.compressed_stream_offset += LZMA_STREAM_HEADER_SIZE as lzma_vli;
        iter.block.compressed_file_offset =
            iter.block.compressed_stream_offset + iter.stream.compressed_offset;
        iter.block.uncompressed_file_offset =
            iter.block.uncompressed_stream_offset + iter.stream.uncompressed_offset;
    }
}
/// # Safety
/// The iterator keeps `i` as a bare pointer, so `i` must stay live and
/// unmodified for as long as `iter` is used. The borrow ends when this call
/// returns; nothing ties the two lifetimes together.
pub unsafe fn lzma_index_iter_init(iter: &mut lzma_index_iter, i: &lzma_index) {
    iter.internal[ITER_INDEX as usize].p = (i as *const lzma_index).cast::<c_void>();
    lzma_index_iter_rewind(iter);
}
pub fn lzma_index_iter_rewind(iter: &mut lzma_index_iter) {
    iter.internal[ITER_STREAM as usize].p = core::ptr::null();
    iter.internal[ITER_GROUP as usize].p = core::ptr::null();
    iter.internal[ITER_RECORD as usize].s = 0;
    iter.internal[ITER_METHOD as usize].s = ITER_METHOD_NORMAL as size_t;
}
/// # Safety
/// `iter` must have been initialised by [`lzma_index_iter_init`] and the
/// `lzma_index` it was given must still be live: the index and the tree nodes
/// below it are reached through the bare pointers stored in `iter.internal`.
pub unsafe fn lzma_index_iter_next(
    iter: &mut lzma_index_iter,
    mode: lzma_index_iter_mode,
) -> lzma_bool {
    if mode > LZMA_INDEX_ITER_NONEMPTY_BLOCK {
        return true as lzma_bool;
    }
    let i: *const lzma_index = iter.internal[ITER_INDEX as usize].p as *const lzma_index;
    let mut stream: *const index_stream =
        iter.internal[ITER_STREAM as usize].p as *const index_stream;
    let mut group: *const index_group = core::ptr::null();
    let mut record: size_t = iter.internal[ITER_RECORD as usize].s;
    if mode != LZMA_INDEX_ITER_STREAM {
        match iter.internal[ITER_METHOD as usize].s {
            0 => {
                group = iter.internal[ITER_GROUP as usize].p as *const index_group;
            }
            1 => {
                group =
                    index_tree_next(iter.internal[ITER_GROUP as usize].p as *const index_tree_node)
                        as *const index_group;
            }
            2 => {
                group = (*stream).groups.leftmost as *const index_group;
            }
            _ => {}
        }
    }
    loop {
        if stream.is_null() {
            stream = (*i).streams.leftmost as *const index_stream;
            if mode >= LZMA_INDEX_ITER_BLOCK {
                while (*stream).groups.leftmost.is_null() {
                    stream = index_tree_next(::core::ptr::addr_of!((*stream).node))
                        as *const index_stream;
                    if stream.is_null() {
                        return true as lzma_bool;
                    }
                }
            }
            group = (*stream).groups.leftmost as *const index_group;
            record = 0;
        } else if !group.is_null() && record < (*group).last {
            record += 1;
        } else {
            record = 0;
            if !group.is_null() {
                group = index_tree_next(::core::ptr::addr_of!((*group).node)) as *const index_group;
            }
            if group.is_null() {
                loop {
                    stream = index_tree_next(::core::ptr::addr_of!((*stream).node))
                        as *const index_stream;
                    if stream.is_null() {
                        return true as lzma_bool;
                    }
                    if mode < LZMA_INDEX_ITER_BLOCK || !(*stream).groups.leftmost.is_null() {
                        break;
                    }
                }
                group = (*stream).groups.leftmost as *const index_group;
            }
        }
        if mode != LZMA_INDEX_ITER_NONEMPTY_BLOCK {
            break;
        }
        if record == 0 {
            if (*group).node.uncompressed_base
                != (*(::core::ptr::addr_of!((*group).records) as *const index_record))
                    .uncompressed_sum
            {
                break;
            }
        } else if (*(::core::ptr::addr_of!((*group).records) as *const index_record)
            .add(record - 1))
        .uncompressed_sum
            != (*(::core::ptr::addr_of!((*group).records) as *const index_record).add(record))
                .uncompressed_sum
        {
            break;
        }
    }
    iter.internal[ITER_STREAM as usize].p = stream as *const c_void;
    iter.internal[ITER_GROUP as usize].p = group as *const c_void;
    iter.internal[ITER_RECORD as usize].s = record;
    iter_set_info(iter);
    false as lzma_bool
}
/// # Safety
/// Same as [`lzma_index_iter_next`].
pub unsafe fn lzma_index_iter_locate(
    iter: &mut lzma_index_iter,
    mut target: lzma_vli,
) -> lzma_bool {
    let i: *const lzma_index = iter.internal[ITER_INDEX as usize].p as *const lzma_index;
    if (*i).uncompressed_size <= target {
        return true as lzma_bool;
    }
    let stream: *const index_stream =
        index_tree_locate(::core::ptr::addr_of!((*i).streams), target) as *const index_stream;
    target -= (*stream).node.uncompressed_base;
    let group: *const index_group =
        index_tree_locate(::core::ptr::addr_of!((*stream).groups), target) as *const index_group;
    let mut left: size_t = 0;
    let mut right: size_t = (*group).last;
    while left < right {
        let pos: size_t = left + (right - left) / 2;
        if (*(::core::ptr::addr_of!((*group).records) as *const index_record).add(pos))
            .uncompressed_sum
            <= target
        {
            left = pos + 1;
        } else {
            right = pos;
        }
    }
    iter.internal[ITER_STREAM as usize].p = stream as *const c_void;
    iter.internal[ITER_GROUP as usize].p = group as *const c_void;
    iter.internal[ITER_RECORD as usize].s = left;
    iter_set_info(iter);
    false as lzma_bool
}
