# Exec Diff: kheap

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/kheap.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/kheap.rs`

| Function | Status | Files |
|----------|--------|-------|
| `alloc` | MISSING_IN_VERUS | alloc_source.rs (MISSING in verus) |
| `allocate` | MISMATCH | allocate_source.rs, allocate_verus.rs, allocate.diff |
| `dealloc` | MISSING_IN_VERUS | dealloc_source.rs (MISSING in verus) |
| `deallocate` | MISMATCH | deallocate_source.rs, deallocate_verus.rs, deallocate.diff |
| `from_raw_parts` | MISMATCH | from_raw_parts_source.rs, from_raw_parts_verus.rs, from_raw_parts.diff |
| `init` | MISMATCH | init_source.rs, init_verus.rs, init.diff |
| `layout_to_allocator` | MISSING_IN_VERUS | layout_to_allocator_source.rs (MISSING in verus) |
| `as_usize` | EXTRA_IN_VERUS | as_usize_verus.rs (EXTRA) |
| `layout_to_slab_size` | EXTRA_IN_VERUS | layout_to_slab_size_verus.rs (EXTRA) |
| `test_layout_to_slab_size_verified` | EXTRA_IN_VERUS | test_layout_to_slab_size_verified_verus.rs (EXTRA) |
| `test_slab_size_as_usize_verified` | EXTRA_IN_VERUS | test_slab_size_as_usize_verified_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ArenaAllocator` | MISSING_IN_VERUS | struct_ArenaAllocator_source.rs (MISSING in verus) |
| `HeapStorage` | MISSING_IN_VERUS | struct_HeapStorage_source.rs (MISSING in verus) |
| `Kheap` | MISMATCH | struct_Kheap_source.rs, struct_Kheap_verus.rs, struct_Kheap.diff |
