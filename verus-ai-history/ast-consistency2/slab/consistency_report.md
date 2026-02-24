# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/libs/slab/src/lib.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/libs/slab/lib.rs`

## Summary

- Functions matched: 0/3
- Functions mismatched: 3
- Missing in Verus: 0
- Extra in Verus: 17
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `allocate` [allocate.diff](allocate.diff) [allocate_source.rs](allocate_source.rs) [allocate_verus.rs](allocate_verus.rs) | MISMATCH | 171-179 | 547-733 |
| `deallocate` [deallocate.diff](deallocate.diff) [deallocate_source.rs](deallocate_source.rs) [deallocate_verus.rs](deallocate_verus.rs) | MISMATCH | 200-223 | 749-1015 |
| `from_raw_parts` [from_raw_parts.diff](from_raw_parts.diff) [from_raw_parts_source.rs](from_raw_parts_source.rs) [from_raw_parts_verus.rs](from_raw_parts_verus.rs) | MISMATCH | 86-159 | 139-412 |
| `block_size` [block_size_verus.rs](block_size_verus.rs) | EXTRA_IN_VERUS |  | 529-534 |
| `from_raw_parts_at_offset` [from_raw_parts_at_offset_verus.rs](from_raw_parts_at_offset_verus.rs) | EXTRA_IN_VERUS |  | 435-508 |
| `is_power_of_two` [is_power_of_two_verus.rs](is_power_of_two_verus.rs) | EXTRA_IN_VERUS |  | 94-113 |
| `num_data_blocks` [num_data_blocks_verus.rs](num_data_blocks_verus.rs) | EXTRA_IN_VERUS |  | 516-521 |
| `test_address_computation_verified` [test_address_computation_verified_verus.rs](test_address_computation_verified_verus.rs) | EXTRA_IN_VERUS |  | 1293-1323 |
| `test_allocate_deallocate_verified` [test_allocate_deallocate_verified_verus.rs](test_allocate_deallocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1141-1178 |
| `test_allocate_out_of_bounds_verified` [test_allocate_out_of_bounds_verified_verus.rs](test_allocate_out_of_bounds_verified_verus.rs) | EXTRA_IN_VERUS |  | 1223-1250 |
| `test_allocation_reuse_verified` [test_allocation_reuse_verified_verus.rs](test_allocation_reuse_verified_verus.rs) | EXTRA_IN_VERUS |  | 1328-1362 |
| `test_double_deallocate_verified` [test_double_deallocate_verified_verus.rs](test_double_deallocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1184-1217 |
| `test_fresh_slab_all_free_verified` [test_fresh_slab_all_free_verified_verus.rs](test_fresh_slab_all_free_verified_verus.rs) | EXTRA_IN_VERUS |  | 1450-1471 |
| `test_index_blocks_always_used_verified` [test_index_blocks_always_used_verified_verus.rs](test_index_blocks_always_used_verified_verus.rs) | EXTRA_IN_VERUS |  | 1476-1506 |
| `test_memory_block_alignment_verified` [test_memory_block_alignment_verified_verus.rs](test_memory_block_alignment_verified_verus.rs) | EXTRA_IN_VERUS |  | 1366-1399 |
| `test_multiple_allocations_verified` [test_multiple_allocations_verified_verus.rs](test_multiple_allocations_verified_verus.rs) | EXTRA_IN_VERUS |  | 1254-1289 |
| `test_no_data_corruption_verified` [test_no_data_corruption_verified_verus.rs](test_no_data_corruption_verified_verus.rs) | EXTRA_IN_VERUS |  | 1403-1446 |
| `test_slab_creation_verified` [test_slab_creation_verified_verus.rs](test_slab_creation_verified_verus.rs) | EXTRA_IN_VERUS |  | 1113-1136 |
| `test_slab_from_raw_parts_allocate_verified` [test_slab_from_raw_parts_allocate_verified_verus.rs](test_slab_from_raw_parts_allocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1063-1107 |
| `test_slab_from_raw_parts_verified` [test_slab_from_raw_parts_verified_verus.rs](test_slab_from_raw_parts_verified_verus.rs) | EXTRA_IN_VERUS |  | 1021-1059 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `allocate` [allocate.diff](allocate.diff) [allocate_source.rs](allocate_source.rs) [allocate_verus.rs](allocate_verus.rs) | MISMATCH | ❌ |
| `deallocate` [deallocate.diff](deallocate.diff) [deallocate_source.rs](deallocate_source.rs) [deallocate_verus.rs](deallocate_verus.rs) | MISMATCH | ❌ |
| `from_raw_parts` [from_raw_parts.diff](from_raw_parts.diff) [from_raw_parts_source.rs](from_raw_parts_source.rs) [from_raw_parts_verus.rs](from_raw_parts_verus.rs) | MISMATCH | ❌ |
| `block_size` [block_size_verus.rs](block_size_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `from_raw_parts_at_offset` [from_raw_parts_at_offset_verus.rs](from_raw_parts_at_offset_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_power_of_two` [is_power_of_two_verus.rs](is_power_of_two_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `num_data_blocks` [num_data_blocks_verus.rs](num_data_blocks_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_address_computation_verified` [test_address_computation_verified_verus.rs](test_address_computation_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_allocate_deallocate_verified` [test_allocate_deallocate_verified_verus.rs](test_allocate_deallocate_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_allocate_out_of_bounds_verified` [test_allocate_out_of_bounds_verified_verus.rs](test_allocate_out_of_bounds_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_allocation_reuse_verified` [test_allocation_reuse_verified_verus.rs](test_allocation_reuse_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_double_deallocate_verified` [test_double_deallocate_verified_verus.rs](test_double_deallocate_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_fresh_slab_all_free_verified` [test_fresh_slab_all_free_verified_verus.rs](test_fresh_slab_all_free_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_index_blocks_always_used_verified` [test_index_blocks_always_used_verified_verus.rs](test_index_blocks_always_used_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_memory_block_alignment_verified` [test_memory_block_alignment_verified_verus.rs](test_memory_block_alignment_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_multiple_allocations_verified` [test_multiple_allocations_verified_verus.rs](test_multiple_allocations_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_no_data_corruption_verified` [test_no_data_corruption_verified_verus.rs](test_no_data_corruption_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_slab_creation_verified` [test_slab_creation_verified_verus.rs](test_slab_creation_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_slab_from_raw_parts_allocate_verified` [test_slab_from_raw_parts_allocate_verified_verus.rs](test_slab_from_raw_parts_allocate_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_slab_from_raw_parts_verified` [test_slab_from_raw_parts_verified_verus.rs](test_slab_from_raw_parts_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `Slab` [struct_Slab.diff](struct_Slab.diff) [struct_Slab_source.rs](struct_Slab_source.rs) [struct_Slab_verus.rs](struct_Slab_verus.rs): MISMATCH
