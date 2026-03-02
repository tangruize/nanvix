# Exec Diff: lib

**Source:** `/home/ubuntu/nanvix/src/libs/slab/src/lib.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/libs/slab/lib.rs`

## Full Diffs (source vs Verus with spec/proof)

Directory: `full/`

| Function | Status | Files |
|----------|--------|-------|
| `Slab::allocate` | MISMATCH | Slab__allocate_source.rs, Slab__allocate_verus.rs, Slab__allocate.diff |
| `Slab::deallocate` | MISMATCH | Slab__deallocate_source.rs, Slab__deallocate_verus.rs, Slab__deallocate.diff |
| `Slab::from_raw_parts` | MISMATCH | Slab__from_raw_parts_source.rs, Slab__from_raw_parts_verus.rs, Slab__from_raw_parts.diff |
| `Slab::block_size` | EXTRA_IN_VERUS | Slab__block_size_verus.rs (EXTRA) |
| `Slab::from_raw_parts_at_offset` | EXTRA_IN_VERUS | Slab__from_raw_parts_at_offset_verus.rs (EXTRA) |
| `Slab::is_power_of_two` | EXTRA_IN_VERUS | Slab__is_power_of_two_verus.rs (EXTRA) |
| `Slab::num_data_blocks` | EXTRA_IN_VERUS | Slab__num_data_blocks_verus.rs (EXTRA) |
| `raw_array_from_addr` | EXTRA_IN_VERUS | raw_array_from_addr_verus.rs (EXTRA) |
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

## Exec-Only Diffs (source vs Verus stripped of ghost/proof)

Directory: `exec-only/`

These diffs show only the executable code differences, with all Verus
annotations (requires/ensures, proof blocks, ghost variables, invariants)
removed. This makes it easier to spot real exec logic changes.

| Function | Status | Files |
|----------|--------|-------|
| `Slab::allocate` | MISMATCH | Slab__allocate_source.rs, Slab__allocate_verus_stripped.rs, Slab__allocate.diff |
| `Slab::deallocate` | MISMATCH | Slab__deallocate_source.rs, Slab__deallocate_verus_stripped.rs, Slab__deallocate.diff |
| `Slab::from_raw_parts` | MISMATCH | Slab__from_raw_parts_source.rs, Slab__from_raw_parts_verus_stripped.rs, Slab__from_raw_parts.diff |
| `Slab::block_size` | EXTRA_IN_VERUS | Slab__block_size_verus.rs (EXTRA) |
| `Slab::from_raw_parts_at_offset` | EXTRA_IN_VERUS | Slab__from_raw_parts_at_offset_verus.rs (EXTRA) |
| `Slab::is_power_of_two` | EXTRA_IN_VERUS | Slab__is_power_of_two_verus.rs (EXTRA) |
| `Slab::num_data_blocks` | EXTRA_IN_VERUS | Slab__num_data_blocks_verus.rs (EXTRA) |
| `raw_array_from_addr` | EXTRA_IN_VERUS | raw_array_from_addr_verus.rs (EXTRA) |
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
