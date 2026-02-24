# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/mm/kheap.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/mm/kheap.rs`

## Summary

- Functions matched: 0/7
- Functions mismatched: 4
- Missing in Verus: 3
- Extra in Verus: 4
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `alloc` [alloc_source.rs](alloc_source.rs) | MISSING_IN_VERUS | 198-212 |  |
| `allocate` [allocate.diff](allocate.diff) [allocate_source.rs](allocate_source.rs) [allocate_verus.rs](allocate_verus.rs) | MISMATCH | 156-167 | 555-646 |
| `dealloc` [dealloc_source.rs](dealloc_source.rs) | MISSING_IN_VERUS | 214-221 |  |
| `deallocate` [deallocate.diff](deallocate.diff) [deallocate_source.rs](deallocate_source.rs) [deallocate_verus.rs](deallocate_verus.rs) | MISMATCH | 169-180 | 671-734 |
| `from_raw_parts` [from_raw_parts.diff](from_raw_parts.diff) [from_raw_parts_source.rs](from_raw_parts_source.rs) [from_raw_parts_verus.rs](from_raw_parts_verus.rs) | MISMATCH | 85-154 | 212-532 |
| `init` [init.diff](init.diff) [init_source.rs](init_source.rs) [init_verus.rs](init_verus.rs) | MISMATCH | 228-237 | 757-786 |
| `layout_to_allocator` [layout_to_allocator_source.rs](layout_to_allocator_source.rs) | MISSING_IN_VERUS | 182-194 |  |
| `as_usize` [as_usize_verus.rs](as_usize_verus.rs) | EXTRA_IN_VERUS |  | 100-113 |
| `layout_to_slab_size` [layout_to_slab_size_verus.rs](layout_to_slab_size_verus.rs) | EXTRA_IN_VERUS |  | 133-154 |
| `test_layout_to_slab_size_verified` [test_layout_to_slab_size_verified_verus.rs](test_layout_to_slab_size_verified_verus.rs) | EXTRA_IN_VERUS |  | 791-843 |
| `test_slab_size_as_usize_verified` [test_slab_size_as_usize_verified_verus.rs](test_slab_size_as_usize_verified_verus.rs) | EXTRA_IN_VERUS |  | 847-880 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `alloc` [alloc_source.rs](alloc_source.rs) | MISSING_IN_VERUS | ❌ |
| `allocate` [allocate.diff](allocate.diff) [allocate_source.rs](allocate_source.rs) [allocate_verus.rs](allocate_verus.rs) | MISMATCH | ❌ |
| `dealloc` [dealloc_source.rs](dealloc_source.rs) | MISSING_IN_VERUS | ❌ |
| `deallocate` [deallocate.diff](deallocate.diff) [deallocate_source.rs](deallocate_source.rs) [deallocate_verus.rs](deallocate_verus.rs) | MISMATCH | ❌ |
| `from_raw_parts` [from_raw_parts.diff](from_raw_parts.diff) [from_raw_parts_source.rs](from_raw_parts_source.rs) [from_raw_parts_verus.rs](from_raw_parts_verus.rs) | MISMATCH | ❌ |
| `init` [init.diff](init.diff) [init_source.rs](init_source.rs) [init_verus.rs](init_verus.rs) | MISMATCH | ❌ |
| `layout_to_allocator` [layout_to_allocator_source.rs](layout_to_allocator_source.rs) | MISSING_IN_VERUS | ❌ |
| `as_usize` [as_usize_verus.rs](as_usize_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `layout_to_slab_size` [layout_to_slab_size_verus.rs](layout_to_slab_size_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_layout_to_slab_size_verified` [test_layout_to_slab_size_verified_verus.rs](test_layout_to_slab_size_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_slab_size_as_usize_verified` [test_slab_size_as_usize_verified_verus.rs](test_slab_size_as_usize_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `ArenaAllocator` [struct_ArenaAllocator_source.rs](struct_ArenaAllocator_source.rs): MISSING_IN_VERUS
- `HeapStorage` [struct_HeapStorage_source.rs](struct_HeapStorage_source.rs): MISSING_IN_VERUS
- `Kheap` [struct_Kheap.diff](struct_Kheap.diff) [struct_Kheap_source.rs](struct_Kheap_source.rs) [struct_Kheap_verus.rs](struct_Kheap_verus.rs): MISMATCH
