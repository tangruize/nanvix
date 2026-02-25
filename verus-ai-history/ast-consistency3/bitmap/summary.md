# Exec Diff: bitmap_dev

**Source:** `bitmap_dev.rs`
**Verus:** `verus/split/libs/bitmap/lib.rs`

## Full Diffs (source vs Verus with spec/proof)

Directory: `full/`

| Function | Status | Files |
|----------|--------|-------|
| `alloc_range` | MISMATCH | alloc_range_source.rs, alloc_range_verus.rs, alloc_range.diff |
| `clear` | MISMATCH | clear_source.rs, clear_verus.rs, clear.diff |
| `from_raw_array` | MISMATCH | from_raw_array_source.rs, from_raw_array_verus.rs, from_raw_array.diff |
| `index` | MISMATCH | index_source.rs, index_verus.rs, index.diff |
| `index_unchecked` | MISMATCH | index_unchecked_source.rs, index_unchecked_verus.rs, index_unchecked.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `set` | MISMATCH | set_source.rs, set_verus.rs, set.diff |
| `test` | MISMATCH | test_source.rs, test_verus.rs, test.diff |
| `clear_range` | EXTRA_IN_VERUS | clear_range_verus.rs (EXTRA) |
| `new_managed` | EXTRA_IN_VERUS | new_managed_verus.rs (EXTRA) |
| `test_alloc_and_clear_all_bits_verified` | EXTRA_IN_VERUS | test_alloc_and_clear_all_bits_verified_verus.rs (EXTRA) |
| `test_alloc_in_partial_bitmap_verified` | EXTRA_IN_VERUS | test_alloc_in_partial_bitmap_verified_verus.rs (EXTRA) |
| `test_alloc_range_across_word_boundary_verified` | EXTRA_IN_VERUS | test_alloc_range_across_word_boundary_verified_verus.rs (EXTRA) |
| `test_alloc_range_and_clear_verified` | EXTRA_IN_VERUS | test_alloc_range_and_clear_verified_verus.rs (EXTRA) |
| `test_bitmap_alloc_range_preserves_others_verified` | EXTRA_IN_VERUS | test_bitmap_alloc_range_preserves_others_verified_verus.rs (EXTRA) |
| `test_bitmap_alloc_range_verified` | EXTRA_IN_VERUS | test_bitmap_alloc_range_verified_verus.rs (EXTRA) |
| `test_bitmap_clear_and_realloc_verified` | EXTRA_IN_VERUS | test_bitmap_clear_and_realloc_verified_verus.rs (EXTRA) |
| `test_bitmap_multiple_alloc_verified` | EXTRA_IN_VERUS | test_bitmap_multiple_alloc_verified_verus.rs (EXTRA) |
| `test_bitmap_number_of_bits_constant_verified` | EXTRA_IN_VERUS | test_bitmap_number_of_bits_constant_verified_verus.rs (EXTRA) |
| `test_bitmap_set_clear_verified` | EXTRA_IN_VERUS | test_bitmap_set_clear_verified_verus.rs (EXTRA) |
| `test_bitmap_usage_tracking_verified` | EXTRA_IN_VERUS | test_bitmap_usage_tracking_verified_verus.rs (EXTRA) |
| `test_set_and_clear_all_bits_verified` | EXTRA_IN_VERUS | test_set_and_clear_all_bits_verified_verus.rs (EXTRA) |
| `test_unchecked` | EXTRA_IN_VERUS | test_unchecked_verus.rs (EXTRA) |
| `usage` | EXTRA_IN_VERUS | usage_verus.rs (EXTRA) |

## Exec-Only Diffs (source vs Verus stripped of ghost/proof)

Directory: `exec-only/`

These diffs show only the executable code differences, with all Verus
annotations (requires/ensures, proof blocks, ghost variables, invariants)
removed. This makes it easier to spot real exec logic changes.

| Function | Status | Files |
|----------|--------|-------|
| `alloc_range` | MISMATCH | alloc_range_source.rs, alloc_range_verus_stripped.rs, alloc_range.diff |
| `clear` | MISMATCH | clear_source.rs, clear_verus_stripped.rs, clear.diff |
| `from_raw_array` | MISMATCH | from_raw_array_source.rs, from_raw_array_verus_stripped.rs, from_raw_array.diff |
| `index` | MISMATCH | index_source.rs, index_verus_stripped.rs, index.diff |
| `index_unchecked` | MISMATCH | index_unchecked_source.rs, index_unchecked_verus_stripped.rs, index_unchecked.diff |
| `new` | MISMATCH | new_source.rs, new_verus_stripped.rs, new.diff |
| `set` | MISMATCH | set_source.rs, set_verus_stripped.rs, set.diff |
| `test` | MISMATCH | test_source.rs, test_verus_stripped.rs, test.diff |
| `clear_range` | EXTRA_IN_VERUS | clear_range_verus.rs (EXTRA) |
| `new_managed` | EXTRA_IN_VERUS | new_managed_verus.rs (EXTRA) |
| `test_alloc_and_clear_all_bits_verified` | EXTRA_IN_VERUS | test_alloc_and_clear_all_bits_verified_verus.rs (EXTRA) |
| `test_alloc_in_partial_bitmap_verified` | EXTRA_IN_VERUS | test_alloc_in_partial_bitmap_verified_verus.rs (EXTRA) |
| `test_alloc_range_across_word_boundary_verified` | EXTRA_IN_VERUS | test_alloc_range_across_word_boundary_verified_verus.rs (EXTRA) |
| `test_alloc_range_and_clear_verified` | EXTRA_IN_VERUS | test_alloc_range_and_clear_verified_verus.rs (EXTRA) |
| `test_bitmap_alloc_range_preserves_others_verified` | EXTRA_IN_VERUS | test_bitmap_alloc_range_preserves_others_verified_verus.rs (EXTRA) |
| `test_bitmap_alloc_range_verified` | EXTRA_IN_VERUS | test_bitmap_alloc_range_verified_verus.rs (EXTRA) |
| `test_bitmap_clear_and_realloc_verified` | EXTRA_IN_VERUS | test_bitmap_clear_and_realloc_verified_verus.rs (EXTRA) |
| `test_bitmap_multiple_alloc_verified` | EXTRA_IN_VERUS | test_bitmap_multiple_alloc_verified_verus.rs (EXTRA) |
| `test_bitmap_number_of_bits_constant_verified` | EXTRA_IN_VERUS | test_bitmap_number_of_bits_constant_verified_verus.rs (EXTRA) |
| `test_bitmap_set_clear_verified` | EXTRA_IN_VERUS | test_bitmap_set_clear_verified_verus.rs (EXTRA) |
| `test_bitmap_usage_tracking_verified` | EXTRA_IN_VERUS | test_bitmap_usage_tracking_verified_verus.rs (EXTRA) |
| `test_set_and_clear_all_bits_verified` | EXTRA_IN_VERUS | test_set_and_clear_all_bits_verified_verus.rs (EXTRA) |
| `test_unchecked` | EXTRA_IN_VERUS | test_unchecked_verus.rs (EXTRA) |
| `usage` | EXTRA_IN_VERUS | usage_verus.rs (EXTRA) |
