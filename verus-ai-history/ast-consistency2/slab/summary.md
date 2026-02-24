# Exec Diff: lib

**Source:** `src/libs/slab/src/lib.rs`
**Verus:** `verus/split/libs/slab/lib.rs`

| Function | Status | Files |
|----------|--------|-------|
| `allocate` | MISMATCH | allocate_source.rs, allocate_verus.rs, allocate.diff |
| `deallocate` | MISMATCH | deallocate_source.rs, deallocate_verus.rs, deallocate.diff |
| `from_raw_parts` | MISMATCH | from_raw_parts_source.rs, from_raw_parts_verus.rs, from_raw_parts.diff |
| `block_size` | EXTRA_IN_VERUS | block_size_verus.rs (EXTRA) |
| `from_raw_parts_at_offset` | EXTRA_IN_VERUS | from_raw_parts_at_offset_verus.rs (EXTRA) |
| `is_power_of_two` | EXTRA_IN_VERUS | is_power_of_two_verus.rs (EXTRA) |
| `num_data_blocks` | EXTRA_IN_VERUS | num_data_blocks_verus.rs (EXTRA) |
| `test_address_computation_verified` | EXTRA_IN_VERUS | test_address_computation_verified_verus.rs (EXTRA) |
| `test_allocate_deallocate_verified` | EXTRA_IN_VERUS | test_allocate_deallocate_verified_verus.rs (EXTRA) |
| `test_allocate_out_of_bounds_verified` | EXTRA_IN_VERUS | test_allocate_out_of_bounds_verified_verus.rs (EXTRA) |
| `test_allocation_reuse_verified` | EXTRA_IN_VERUS | test_allocation_reuse_verified_verus.rs (EXTRA) |
| `test_double_deallocate_verified` | EXTRA_IN_VERUS | test_double_deallocate_verified_verus.rs (EXTRA) |
| `test_fresh_slab_all_free_verified` | EXTRA_IN_VERUS | test_fresh_slab_all_free_verified_verus.rs (EXTRA) |
| `test_index_blocks_always_used_verified` | EXTRA_IN_VERUS | test_index_blocks_always_used_verified_verus.rs (EXTRA) |
| `test_memory_block_alignment_verified` | EXTRA_IN_VERUS | test_memory_block_alignment_verified_verus.rs (EXTRA) |
| `test_multiple_allocations_verified` | EXTRA_IN_VERUS | test_multiple_allocations_verified_verus.rs (EXTRA) |
| `test_no_data_corruption_verified` | EXTRA_IN_VERUS | test_no_data_corruption_verified_verus.rs (EXTRA) |
| `test_slab_creation_verified` | EXTRA_IN_VERUS | test_slab_creation_verified_verus.rs (EXTRA) |
| `test_slab_from_raw_parts_allocate_verified` | EXTRA_IN_VERUS | test_slab_from_raw_parts_allocate_verified_verus.rs (EXTRA) |
| `test_slab_from_raw_parts_verified` | EXTRA_IN_VERUS | test_slab_from_raw_parts_verified_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `Slab` | MISMATCH | struct_Slab_source.rs, struct_Slab_verus.rs, struct_Slab.diff |
