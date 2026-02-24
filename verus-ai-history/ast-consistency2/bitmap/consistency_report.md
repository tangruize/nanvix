# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/libs/bitmap/src/lib.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/libs/bitmap/lib.rs`

## Summary

- Functions matched: 1/11
- Functions mismatched: 10
- Missing in Verus: 0
- Extra in Verus: 16
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines | Diff |
|----------|--------|-------------|-------------|------|
| `alloc` | MISMATCH | 141-143 | 169-197  | [diff](alloc.diff) [src](alloc_source.rs) [verus](alloc_verus.rs) |
| `alloc_range` | MISMATCH | 159-219 | 201-663  | [diff](alloc_range.diff) [src](alloc_range_source.rs) [verus](alloc_range_verus.rs) |
| `clear` | MISMATCH | 259-269 | 748-816  | [diff](clear.diff) [src](clear_source.rs) [verus](clear_verus.rs) |
| `from_raw_array` | MISMATCH | 103-116 | 121-143  | [diff](from_raw_array.diff) [src](from_raw_array_source.rs) [verus](from_raw_array_verus.rs) |
| `index` | MISMATCH | 304-312 | 965-985  | [diff](index.diff) [src](index_source.rs) [verus](index_verus.rs) |
| `index_unchecked` | MISMATCH | 327-331 | 949-962  | [diff](index_unchecked.diff) [src](index_unchecked_source.rs) [verus](index_unchecked_verus.rs) |
| `new` | MISMATCH | 61-87 | 56-99  | [diff](new.diff) [src](new_source.rs) [verus](new_verus.rs) |
| `number_of_bits` | MISMATCH | 127-129 | 146-155  | [diff](number_of_bits.diff) [src](number_of_bits_source.rs) [verus](number_of_bits_verus.rs) |
| `set` | MISMATCH | 234-244 | 666-745  | [diff](set.diff) [src](set_source.rs) [verus](set_verus.rs) |
| `test` | MISMATCH | 285-288 | 926-942  | [diff](test.diff) [src](test_source.rs) [verus](test_verus.rs) |
| `clear_range` | EXTRA_IN_VERUS |  | 820-923  | [verus](clear_range_verus.rs) |
| `new_managed` | EXTRA_IN_VERUS |  | 102-118  | [verus](new_managed_verus.rs) |
| `test_alloc_and_clear_all_bits_verified` | EXTRA_IN_VERUS |  | 1161-1217  | [verus](test_alloc_and_clear_all_bits_verified_verus.rs) |
| `test_alloc_in_partial_bitmap_verified` | EXTRA_IN_VERUS |  | 1258-1291  | [verus](test_alloc_in_partial_bitmap_verified_verus.rs) |
| `test_alloc_range_across_word_boundary_verified` | EXTRA_IN_VERUS |  | 1220-1255  | [verus](test_alloc_range_across_word_boundary_verified_verus.rs) |
| `test_alloc_range_and_clear_verified` | EXTRA_IN_VERUS |  | 1294-1336  | [verus](test_alloc_range_and_clear_verified_verus.rs) |
| `test_bitmap_alloc_range_preserves_others_verified` | EXTRA_IN_VERUS |  | 1378-1402  | [verus](test_bitmap_alloc_range_preserves_others_verified_verus.rs) |
| `test_bitmap_alloc_range_verified` | EXTRA_IN_VERUS |  | 1031-1050  | [verus](test_bitmap_alloc_range_verified_verus.rs) |
| `test_bitmap_clear_and_realloc_verified` | EXTRA_IN_VERUS |  | 1076-1099  | [verus](test_bitmap_clear_and_realloc_verified_verus.rs) |
| `test_bitmap_multiple_alloc_verified` | EXTRA_IN_VERUS |  | 1053-1073  | [verus](test_bitmap_multiple_alloc_verified_verus.rs) |
| `test_bitmap_number_of_bits_constant_verified` | EXTRA_IN_VERUS |  | 1405-1435  | [verus](test_bitmap_number_of_bits_constant_verified_verus.rs) |
| `test_bitmap_set_clear_verified` | EXTRA_IN_VERUS |  | 1005-1028  | [verus](test_bitmap_set_clear_verified_verus.rs) |
| `test_bitmap_usage_tracking_verified` | EXTRA_IN_VERUS |  | 1339-1375  | [verus](test_bitmap_usage_tracking_verified_verus.rs) |
| `test_set_and_clear_all_bits_verified` | EXTRA_IN_VERUS |  | 1102-1158  | [verus](test_set_and_clear_all_bits_verified_verus.rs) |
| `test_unchecked` | EXTRA_IN_VERUS |  | 988-997  | [verus](test_unchecked_verus.rs) |
| `usage` | EXTRA_IN_VERUS |  | 158-166  | [verus](usage_verus.rs) |

## All Functions

| Function | Status | Hash Match | Diff |
|----------|--------|------------|------|
| `alloc` | MISMATCH | ❌  | [diff](alloc.diff) |
 | [diff](alloc.diff) [src](alloc_source.rs) [verus](alloc_verus.rs) | `alloc_range` | MISMATCH | ❌  | [diff](alloc_range.diff) |
| `clear` | MISMATCH | ❌  | [diff](clear.diff) |
 | [diff](clear.diff) [src](clear_source.rs) [verus](clear_verus.rs) | `deref` | MATCH | ✅  | |
| `from_raw_array` | MISMATCH | ❌  | [diff](from_raw_array.diff) |
 | [diff](from_raw_array.diff) [src](from_raw_array_source.rs) [verus](from_raw_array_verus.rs) | `index` | MISMATCH | ❌  | [diff](index.diff) |
| `index_unchecked` | MISMATCH | ❌  | [diff](index_unchecked.diff) |
 | [diff](index_unchecked.diff) [src](index_unchecked_source.rs) [verus](index_unchecked_verus.rs) | `new` | MISMATCH | ❌  | [diff](new.diff) |
| `number_of_bits` | MISMATCH | ❌  | [diff](number_of_bits.diff) |
 | [diff](number_of_bits.diff) [src](number_of_bits_source.rs) [verus](number_of_bits_verus.rs) | `set` | MISMATCH | ❌  | [diff](set.diff) |
| `test` | MISMATCH | ❌  | [diff](test.diff) |
 | [diff](test.diff) [src](test_source.rs) [verus](test_verus.rs) | `clear_range` | EXTRA_IN_VERUS | ❌  | [verus](clear_range_verus.rs) |
| `new_managed` | EXTRA_IN_VERUS | ❌  | [verus](new_managed_verus.rs) |
 | [verus](new_managed_verus.rs) | `test_alloc_and_clear_all_bits_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_alloc_and_clear_all_bits_verified_verus.rs) |
| `test_alloc_in_partial_bitmap_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_alloc_in_partial_bitmap_verified_verus.rs) |
 | [verus](test_alloc_in_partial_bitmap_verified_verus.rs) | `test_alloc_range_across_word_boundary_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_alloc_range_across_word_boundary_verified_verus.rs) |
| `test_alloc_range_and_clear_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_alloc_range_and_clear_verified_verus.rs) |
 | [verus](test_alloc_range_and_clear_verified_verus.rs) | `test_bitmap_alloc_range_preserves_others_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_bitmap_alloc_range_preserves_others_verified_verus.rs) |
| `test_bitmap_alloc_range_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_bitmap_alloc_range_verified_verus.rs) |
 | [verus](test_bitmap_alloc_range_verified_verus.rs) | `test_bitmap_clear_and_realloc_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_bitmap_clear_and_realloc_verified_verus.rs) |
| `test_bitmap_multiple_alloc_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_bitmap_multiple_alloc_verified_verus.rs) |
 | [verus](test_bitmap_multiple_alloc_verified_verus.rs) | `test_bitmap_number_of_bits_constant_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_bitmap_number_of_bits_constant_verified_verus.rs) |
| `test_bitmap_set_clear_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_bitmap_set_clear_verified_verus.rs) |
 | [verus](test_bitmap_set_clear_verified_verus.rs) | `test_bitmap_usage_tracking_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_bitmap_usage_tracking_verified_verus.rs) |
| `test_set_and_clear_all_bits_verified` | EXTRA_IN_VERUS | ❌  | [verus](test_set_and_clear_all_bits_verified_verus.rs) |
 | [verus](test_set_and_clear_all_bits_verified_verus.rs) | `test_unchecked` | EXTRA_IN_VERUS | ❌  | [verus](test_unchecked_verus.rs) |
| `usage` | EXTRA_IN_VERUS | ❌  | [verus](usage_verus.rs) |
