# Exec Consistency Report

**Source:** `src/libs/slab/src/lib.rs`
**Verus:** `verus/split/libs/slab/lib.rs`

## Summary

- Functions matched: 0/3
- Functions mismatched: 3
- Missing in Verus: 0
- Extra in Verus: 16
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `Slab::allocate` [Slab__allocate.diff](Slab__allocate.diff) [Slab__allocate_source.rs](Slab__allocate_source.rs) [Slab__allocate_verus.rs](Slab__allocate_verus.rs) | MISMATCH | 174-182 | 584-770 |
| `Slab::deallocate` [Slab__deallocate.diff](Slab__deallocate.diff) [Slab__deallocate_source.rs](Slab__deallocate_source.rs) [Slab__deallocate_verus.rs](Slab__deallocate_verus.rs) | MISMATCH | 203-226 | 793-1045 |
| `Slab::from_raw_parts` [Slab__from_raw_parts.diff](Slab__from_raw_parts.diff) [Slab__from_raw_parts_source.rs](Slab__from_raw_parts_source.rs) [Slab__from_raw_parts_verus.rs](Slab__from_raw_parts_verus.rs) | MISMATCH | 86-162 | 179-468 |
| `Slab::from_raw_parts_at_offset` [Slab__from_raw_parts_at_offset_verus.rs](Slab__from_raw_parts_at_offset_verus.rs) | EXTRA_IN_VERUS |  | 491-564 |
| `Slab::is_power_of_two` [Slab__is_power_of_two_verus.rs](Slab__is_power_of_two_verus.rs) | EXTRA_IN_VERUS |  | 121-140 |
| `raw_array_from_addr` [raw_array_from_addr_verus.rs](raw_array_from_addr_verus.rs) | EXTRA_IN_VERUS |  | 46-60 |
| `test_address_computation_verified` [test_address_computation_verified_verus.rs](test_address_computation_verified_verus.rs) | EXTRA_IN_VERUS |  | 1323-1353 |
| `test_allocate_deallocate_verified` [test_allocate_deallocate_verified_verus.rs](test_allocate_deallocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1171-1208 |
| `test_allocate_out_of_bounds_verified` [test_allocate_out_of_bounds_verified_verus.rs](test_allocate_out_of_bounds_verified_verus.rs) | EXTRA_IN_VERUS |  | 1253-1280 |
| `test_allocation_reuse_verified` [test_allocation_reuse_verified_verus.rs](test_allocation_reuse_verified_verus.rs) | EXTRA_IN_VERUS |  | 1358-1392 |
| `test_double_deallocate_verified` [test_double_deallocate_verified_verus.rs](test_double_deallocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1214-1247 |
| `test_fresh_slab_all_free_verified` [test_fresh_slab_all_free_verified_verus.rs](test_fresh_slab_all_free_verified_verus.rs) | EXTRA_IN_VERUS |  | 1480-1501 |
| `test_index_blocks_always_used_verified` [test_index_blocks_always_used_verified_verus.rs](test_index_blocks_always_used_verified_verus.rs) | EXTRA_IN_VERUS |  | 1506-1536 |
| `test_memory_block_alignment_verified` [test_memory_block_alignment_verified_verus.rs](test_memory_block_alignment_verified_verus.rs) | EXTRA_IN_VERUS |  | 1396-1429 |
| `test_multiple_allocations_verified` [test_multiple_allocations_verified_verus.rs](test_multiple_allocations_verified_verus.rs) | EXTRA_IN_VERUS |  | 1284-1319 |
| `test_no_data_corruption_verified` [test_no_data_corruption_verified_verus.rs](test_no_data_corruption_verified_verus.rs) | EXTRA_IN_VERUS |  | 1433-1476 |
| `test_slab_creation_verified` [test_slab_creation_verified_verus.rs](test_slab_creation_verified_verus.rs) | EXTRA_IN_VERUS |  | 1143-1166 |
| `test_slab_from_raw_parts_allocate_verified` [test_slab_from_raw_parts_allocate_verified_verus.rs](test_slab_from_raw_parts_allocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1093-1137 |
| `test_slab_from_raw_parts_verified` [test_slab_from_raw_parts_verified_verus.rs](test_slab_from_raw_parts_verified_verus.rs) | EXTRA_IN_VERUS |  | 1051-1089 |

## All Functions

| Function | Status | Hash Match | Verification |
|----------|--------|------------|--------------|
| `Slab::allocate` | MISMATCH | ❌ | ✅ verified |
| `Slab::deallocate` | MISMATCH | ❌ | ✅ verified |
| `Slab::from_raw_parts` | MISMATCH | ❌ | ✅ verified |
| `Slab::from_raw_parts_at_offset` [Slab__from_raw_parts_at_offset_verus.rs](Slab__from_raw_parts_at_offset_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `Slab::is_power_of_two` [Slab__is_power_of_two_verus.rs](Slab__is_power_of_two_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `raw_array_from_addr` [raw_array_from_addr_verus.rs](raw_array_from_addr_verus.rs) | EXTRA_IN_VERUS | ❌ | 🔒 external_body |
| `test_address_computation_verified` [test_address_computation_verified_verus.rs](test_address_computation_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_allocate_deallocate_verified` [test_allocate_deallocate_verified_verus.rs](test_allocate_deallocate_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_allocate_out_of_bounds_verified` [test_allocate_out_of_bounds_verified_verus.rs](test_allocate_out_of_bounds_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_allocation_reuse_verified` [test_allocation_reuse_verified_verus.rs](test_allocation_reuse_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_double_deallocate_verified` [test_double_deallocate_verified_verus.rs](test_double_deallocate_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_fresh_slab_all_free_verified` [test_fresh_slab_all_free_verified_verus.rs](test_fresh_slab_all_free_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_index_blocks_always_used_verified` [test_index_blocks_always_used_verified_verus.rs](test_index_blocks_always_used_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_memory_block_alignment_verified` [test_memory_block_alignment_verified_verus.rs](test_memory_block_alignment_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_multiple_allocations_verified` [test_multiple_allocations_verified_verus.rs](test_multiple_allocations_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_no_data_corruption_verified` [test_no_data_corruption_verified_verus.rs](test_no_data_corruption_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_slab_creation_verified` [test_slab_creation_verified_verus.rs](test_slab_creation_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_slab_from_raw_parts_allocate_verified` [test_slab_from_raw_parts_allocate_verified_verus.rs](test_slab_from_raw_parts_allocate_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `test_slab_from_raw_parts_verified` [test_slab_from_raw_parts_verified_verus.rs](test_slab_from_raw_parts_verified_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |

## Verification Coverage

**🔒 1 function(s) use `external_body`** (body not verified):

- `raw_array_from_addr` (lines 46-60)

These functions have requires/ensures contracts but the body is trusted.
Justify why `external_body` is necessary for each.

## Inconsistent Structs

| Struct | Status | Source Lines | Verus Lines |
|--------|--------|-------------|-------------|
| `Slab` | MISMATCH | 46-57 | 77-96 |
