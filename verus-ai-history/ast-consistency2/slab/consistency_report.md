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
| `allocate` [diff](allocate.diff) [source](allocate_source.rs) [verus](allocate_verus.rs) | MISMATCH | 171-179 | 547-733 |
| `deallocate` [diff](deallocate.diff) [source](deallocate_source.rs) [verus](deallocate_verus.rs) | MISMATCH | 200-223 | 749-1015 |
| `from_raw_parts` [diff](from_raw_parts.diff) [source](from_raw_parts_source.rs) [verus](from_raw_parts_verus.rs) | MISMATCH | 86-159 | 139-412 |
| `block_size` [verus](block_size_verus.rs) | EXTRA_IN_VERUS |  | 529-534 |
| `from_raw_parts_at_offset` [verus](from_raw_parts_at_offset_verus.rs) | EXTRA_IN_VERUS |  | 435-508 |
| `is_power_of_two` [verus](is_power_of_two_verus.rs) | EXTRA_IN_VERUS |  | 94-113 |
| `num_data_blocks` [verus](num_data_blocks_verus.rs) | EXTRA_IN_VERUS |  | 516-521 |
| `test_address_computation_verified` [verus](test_address_computation_verified_verus.rs) | EXTRA_IN_VERUS |  | 1293-1323 |
| `test_allocate_deallocate_verified` [verus](test_allocate_deallocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1141-1178 |
| `test_allocate_out_of_bounds_verified` [verus](test_allocate_out_of_bounds_verified_verus.rs) | EXTRA_IN_VERUS |  | 1223-1250 |
| `test_allocation_reuse_verified` [verus](test_allocation_reuse_verified_verus.rs) | EXTRA_IN_VERUS |  | 1328-1362 |
| `test_double_deallocate_verified` [verus](test_double_deallocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1184-1217 |
| `test_fresh_slab_all_free_verified` [verus](test_fresh_slab_all_free_verified_verus.rs) | EXTRA_IN_VERUS |  | 1450-1471 |
| `test_index_blocks_always_used_verified` [verus](test_index_blocks_always_used_verified_verus.rs) | EXTRA_IN_VERUS |  | 1476-1506 |
| `test_memory_block_alignment_verified` [verus](test_memory_block_alignment_verified_verus.rs) | EXTRA_IN_VERUS |  | 1366-1399 |
| `test_multiple_allocations_verified` [verus](test_multiple_allocations_verified_verus.rs) | EXTRA_IN_VERUS |  | 1254-1289 |
| `test_no_data_corruption_verified` [verus](test_no_data_corruption_verified_verus.rs) | EXTRA_IN_VERUS |  | 1403-1446 |
| `test_slab_creation_verified` [verus](test_slab_creation_verified_verus.rs) | EXTRA_IN_VERUS |  | 1113-1136 |
| `test_slab_from_raw_parts_allocate_verified` [verus](test_slab_from_raw_parts_allocate_verified_verus.rs) | EXTRA_IN_VERUS |  | 1063-1107 |
| `test_slab_from_raw_parts_verified` [verus](test_slab_from_raw_parts_verified_verus.rs) | EXTRA_IN_VERUS |  | 1021-1059 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `allocate` [diff](allocate.diff) [source](allocate_source.rs) [verus](allocate_verus.rs) | MISMATCH | ❌ |
| `deallocate` [diff](deallocate.diff) [source](deallocate_source.rs) [verus](deallocate_verus.rs) | MISMATCH | ❌ |
| `from_raw_parts` [diff](from_raw_parts.diff) [source](from_raw_parts_source.rs) [verus](from_raw_parts_verus.rs) | MISMATCH | ❌ |
| `block_size` [verus](block_size_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `from_raw_parts_at_offset` [verus](from_raw_parts_at_offset_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `is_power_of_two` [verus](is_power_of_two_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `num_data_blocks` [verus](num_data_blocks_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_address_computation_verified` [verus](test_address_computation_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_allocate_deallocate_verified` [verus](test_allocate_deallocate_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_allocate_out_of_bounds_verified` [verus](test_allocate_out_of_bounds_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_allocation_reuse_verified` [verus](test_allocation_reuse_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_double_deallocate_verified` [verus](test_double_deallocate_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_fresh_slab_all_free_verified` [verus](test_fresh_slab_all_free_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_index_blocks_always_used_verified` [verus](test_index_blocks_always_used_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_memory_block_alignment_verified` [verus](test_memory_block_alignment_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_multiple_allocations_verified` [verus](test_multiple_allocations_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_no_data_corruption_verified` [verus](test_no_data_corruption_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_slab_creation_verified` [verus](test_slab_creation_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_slab_from_raw_parts_allocate_verified` [verus](test_slab_from_raw_parts_allocate_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_slab_from_raw_parts_verified` [verus](test_slab_from_raw_parts_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `Slab`: MISMATCH
