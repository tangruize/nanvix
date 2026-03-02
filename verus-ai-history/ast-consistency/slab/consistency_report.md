# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/libs/slab/src/lib.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/libs/slab/lib.rs`

## Summary

- Functions matched: 0/3
- Functions mismatched: 3
- Missing in Verus: 0
- Extra in Verus: 18
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `Slab::allocate` [Slab__allocate.diff](Slab__allocate.diff) [Slab__allocate_source.rs](Slab__allocate_source.rs) [Slab__allocate_verus.rs](Slab__allocate_verus.rs) | MISMATCH | 174-182 | 594-780 |
| `Slab::deallocate` [Slab__deallocate.diff](Slab__deallocate.diff) [Slab__deallocate_source.rs](Slab__deallocate_source.rs) [Slab__deallocate_verus.rs](Slab__deallocate_verus.rs) | MISMATCH | 203-226 | 803-1069 |
| `Slab::from_raw_parts` [Slab__from_raw_parts.diff](Slab__from_raw_parts.diff) [Slab__from_raw_parts_source.rs](Slab__from_raw_parts_source.rs) [Slab__from_raw_parts_verus.rs](Slab__from_raw_parts_verus.rs) | MISMATCH | 86-162 | 179-452 |
| `Slab::block_size` [Slab__block_size_verus.rs](Slab__block_size_verus.rs) | EXTRA_IN_VERUS |  | 569-574 |
| `Slab::from_raw_parts_at_offset` [Slab__from_raw_parts_at_offset_verus.rs](Slab__from_raw_parts_at_offset_verus.rs) | EXTRA_IN_VERUS |  | 475-548 |
| `Slab::is_power_of_two` [Slab__is_power_of_two_verus.rs](Slab__is_power_of_two_verus.rs) | EXTRA_IN_VERUS |  | 121-140 |
| `Slab::num_data_blocks` [Slab__num_data_blocks_verus.rs](Slab__num_data_blocks_verus.rs) | EXTRA_IN_VERUS |  | 556-561 |
| `raw_array_from_addr` [raw_array_from_addr_verus.rs](raw_array_from_addr_verus.rs) | EXTRA_IN_VERUS |  | 46-60 |
| `test_address_computation_verified` [test_address_computation_verified_verus.rs](test_address_computation_verified_verus.rs) | EXTRA_IN_VERUS |  | 1347-1377 |
| `test_allocate_deallocate_verified` [test_allocate_deallocate_verified_verus.rs](test_allocate_deallocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1195-1232 |
| `test_allocate_out_of_bounds_verified` [test_allocate_out_of_bounds_verified_verus.rs](test_allocate_out_of_bounds_verified_verus.rs) | EXTRA_IN_VERUS |  | 1277-1304 |
| `test_allocation_reuse_verified` [test_allocation_reuse_verified_verus.rs](test_allocation_reuse_verified_verus.rs) | EXTRA_IN_VERUS |  | 1382-1416 |
| `test_double_deallocate_verified` [test_double_deallocate_verified_verus.rs](test_double_deallocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1238-1271 |
| `test_fresh_slab_all_free_verified` [test_fresh_slab_all_free_verified_verus.rs](test_fresh_slab_all_free_verified_verus.rs) | EXTRA_IN_VERUS |  | 1504-1525 |
| `test_index_blocks_always_used_verified` [test_index_blocks_always_used_verified_verus.rs](test_index_blocks_always_used_verified_verus.rs) | EXTRA_IN_VERUS |  | 1530-1560 |
| `test_memory_block_alignment_verified` [test_memory_block_alignment_verified_verus.rs](test_memory_block_alignment_verified_verus.rs) | EXTRA_IN_VERUS |  | 1420-1453 |
| `test_multiple_allocations_verified` [test_multiple_allocations_verified_verus.rs](test_multiple_allocations_verified_verus.rs) | EXTRA_IN_VERUS |  | 1308-1343 |
| `test_no_data_corruption_verified` [test_no_data_corruption_verified_verus.rs](test_no_data_corruption_verified_verus.rs) | EXTRA_IN_VERUS |  | 1457-1500 |
| `test_slab_creation_verified` [test_slab_creation_verified_verus.rs](test_slab_creation_verified_verus.rs) | EXTRA_IN_VERUS |  | 1167-1190 |
| `test_slab_from_raw_parts_allocate_verified` [test_slab_from_raw_parts_allocate_verified_verus.rs](test_slab_from_raw_parts_allocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1117-1161 |
| `test_slab_from_raw_parts_verified` [test_slab_from_raw_parts_verified_verus.rs](test_slab_from_raw_parts_verified_verus.rs) | EXTRA_IN_VERUS |  | 1075-1113 |

## All Functions

| Function | Status | Hash Match | Verification |
|----------|--------|------------|--------------|
| `Slab::allocate` | MISMATCH | ❌ | ✅ verified |
| `Slab::deallocate` | MISMATCH | ❌ | ✅ verified |
| `Slab::from_raw_parts` | MISMATCH | ❌ | ✅ verified |
| `Slab::block_size` [Slab__block_size_verus.rs](Slab__block_size_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `Slab::from_raw_parts_at_offset` [Slab__from_raw_parts_at_offset_verus.rs](Slab__from_raw_parts_at_offset_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `Slab::is_power_of_two` [Slab__is_power_of_two_verus.rs](Slab__is_power_of_two_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `Slab::num_data_blocks` [Slab__num_data_blocks_verus.rs](Slab__num_data_blocks_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
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
