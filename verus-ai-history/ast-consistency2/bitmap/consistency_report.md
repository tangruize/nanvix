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

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `alloc` [alloc.diff](alloc.diff) | [alloc_source.rs](alloc_source.rs) | [alloc_verus.rs](alloc_verus.rs) | MISMATCH | 141-143 | 169-197 |
| `alloc_range` [alloc_range.diff](alloc_range.diff) | [alloc_range_source.rs](alloc_range_source.rs) | [alloc_range_verus.rs](alloc_range_verus.rs) | MISMATCH | 159-219 | 201-663 |
| `clear` [clear.diff](clear.diff) | [clear_source.rs](clear_source.rs) | [clear_verus.rs](clear_verus.rs) | MISMATCH | 259-269 | 748-816 |
| `from_raw_array` [from_raw_array.diff](from_raw_array.diff) | [from_raw_array_source.rs](from_raw_array_source.rs) | [from_raw_array_verus.rs](from_raw_array_verus.rs) | MISMATCH | 103-116 | 121-143 |
| `index` [index.diff](index.diff) | [index_source.rs](index_source.rs) | [index_verus.rs](index_verus.rs) | MISMATCH | 304-312 | 965-985 |
| `index_unchecked` [index_unchecked.diff](index_unchecked.diff) | [index_unchecked_source.rs](index_unchecked_source.rs) | [index_unchecked_verus.rs](index_unchecked_verus.rs) | MISMATCH | 327-331 | 949-962 |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 61-87 | 56-99 |
| `number_of_bits` [number_of_bits.diff](number_of_bits.diff) | [number_of_bits_source.rs](number_of_bits_source.rs) | [number_of_bits_verus.rs](number_of_bits_verus.rs) | MISMATCH | 127-129 | 146-155 |
| `set` [set.diff](set.diff) | [set_source.rs](set_source.rs) | [set_verus.rs](set_verus.rs) | MISMATCH | 234-244 | 666-745 |
| `test` [test.diff](test.diff) | [test_source.rs](test_source.rs) | [test_verus.rs](test_verus.rs) | MISMATCH | 285-288 | 926-942 |
| `clear_range` [clear_range_verus.rs](clear_range_verus.rs) | EXTRA_IN_VERUS |  | 820-923 |
| `new_managed` [new_managed_verus.rs](new_managed_verus.rs) | EXTRA_IN_VERUS |  | 102-118 |
| `test_alloc_and_clear_all_bits_verified` [test_alloc_and_clear_all_bits_verified_verus.rs](test_alloc_and_clear_all_bits_verified_verus.rs) | EXTRA_IN_VERUS |  | 1161-1217 |
| `test_alloc_in_partial_bitmap_verified` [test_alloc_in_partial_bitmap_verified_verus.rs](test_alloc_in_partial_bitmap_verified_verus.rs) | EXTRA_IN_VERUS |  | 1258-1291 |
| `test_alloc_range_across_word_boundary_verified` [test_alloc_range_across_word_boundary_verified_verus.rs](test_alloc_range_across_word_boundary_verified_verus.rs) | EXTRA_IN_VERUS |  | 1220-1255 |
| `test_alloc_range_and_clear_verified` [test_alloc_range_and_clear_verified_verus.rs](test_alloc_range_and_clear_verified_verus.rs) | EXTRA_IN_VERUS |  | 1294-1336 |
| `test_bitmap_alloc_range_preserves_others_verified` [test_bitmap_alloc_range_preserves_others_verified_verus.rs](test_bitmap_alloc_range_preserves_others_verified_verus.rs) | EXTRA_IN_VERUS |  | 1378-1402 |
| `test_bitmap_alloc_range_verified` [test_bitmap_alloc_range_verified_verus.rs](test_bitmap_alloc_range_verified_verus.rs) | EXTRA_IN_VERUS |  | 1031-1050 |
| `test_bitmap_clear_and_realloc_verified` [test_bitmap_clear_and_realloc_verified_verus.rs](test_bitmap_clear_and_realloc_verified_verus.rs) | EXTRA_IN_VERUS |  | 1076-1099 |
| `test_bitmap_multiple_alloc_verified` [test_bitmap_multiple_alloc_verified_verus.rs](test_bitmap_multiple_alloc_verified_verus.rs) | EXTRA_IN_VERUS |  | 1053-1073 |
| `test_bitmap_number_of_bits_constant_verified` [test_bitmap_number_of_bits_constant_verified_verus.rs](test_bitmap_number_of_bits_constant_verified_verus.rs) | EXTRA_IN_VERUS |  | 1405-1435 |
| `test_bitmap_set_clear_verified` [test_bitmap_set_clear_verified_verus.rs](test_bitmap_set_clear_verified_verus.rs) | EXTRA_IN_VERUS |  | 1005-1028 |
| `test_bitmap_usage_tracking_verified` [test_bitmap_usage_tracking_verified_verus.rs](test_bitmap_usage_tracking_verified_verus.rs) | EXTRA_IN_VERUS |  | 1339-1375 |
| `test_set_and_clear_all_bits_verified` [test_set_and_clear_all_bits_verified_verus.rs](test_set_and_clear_all_bits_verified_verus.rs) | EXTRA_IN_VERUS |  | 1102-1158 |
| `test_unchecked` [test_unchecked_verus.rs](test_unchecked_verus.rs) | EXTRA_IN_VERUS |  | 988-997 |
| `usage` [usage_verus.rs](usage_verus.rs) | EXTRA_IN_VERUS |  | 158-166 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `alloc` [alloc.diff](alloc.diff) | [alloc_source.rs](alloc_source.rs) | [alloc_verus.rs](alloc_verus.rs) | MISMATCH | ❌ |
| `alloc_range` [alloc_range.diff](alloc_range.diff) | [alloc_range_source.rs](alloc_range_source.rs) | [alloc_range_verus.rs](alloc_range_verus.rs) | MISMATCH | ❌ |
| `clear` [clear.diff](clear.diff) | [clear_source.rs](clear_source.rs) | [clear_verus.rs](clear_verus.rs) | MISMATCH | ❌ |
| `deref` | MATCH | ✅ |
| `from_raw_array` [from_raw_array.diff](from_raw_array.diff) | [from_raw_array_source.rs](from_raw_array_source.rs) | [from_raw_array_verus.rs](from_raw_array_verus.rs) | MISMATCH | ❌ |
| `index` [index.diff](index.diff) | [index_source.rs](index_source.rs) | [index_verus.rs](index_verus.rs) | MISMATCH | ❌ |
| `index_unchecked` [index_unchecked.diff](index_unchecked.diff) | [index_unchecked_source.rs](index_unchecked_source.rs) | [index_unchecked_verus.rs](index_unchecked_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `number_of_bits` [number_of_bits.diff](number_of_bits.diff) | [number_of_bits_source.rs](number_of_bits_source.rs) | [number_of_bits_verus.rs](number_of_bits_verus.rs) | MISMATCH | ❌ |
| `set` [set.diff](set.diff) | [set_source.rs](set_source.rs) | [set_verus.rs](set_verus.rs) | MISMATCH | ❌ |
| `test` [test.diff](test.diff) | [test_source.rs](test_source.rs) | [test_verus.rs](test_verus.rs) | MISMATCH | ❌ |
| `clear_range` [clear_range_verus.rs](clear_range_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `new_managed` [new_managed_verus.rs](new_managed_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_alloc_and_clear_all_bits_verified` [test_alloc_and_clear_all_bits_verified_verus.rs](test_alloc_and_clear_all_bits_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_alloc_in_partial_bitmap_verified` [test_alloc_in_partial_bitmap_verified_verus.rs](test_alloc_in_partial_bitmap_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_alloc_range_across_word_boundary_verified` [test_alloc_range_across_word_boundary_verified_verus.rs](test_alloc_range_across_word_boundary_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_alloc_range_and_clear_verified` [test_alloc_range_and_clear_verified_verus.rs](test_alloc_range_and_clear_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_bitmap_alloc_range_preserves_others_verified` [test_bitmap_alloc_range_preserves_others_verified_verus.rs](test_bitmap_alloc_range_preserves_others_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_bitmap_alloc_range_verified` [test_bitmap_alloc_range_verified_verus.rs](test_bitmap_alloc_range_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_bitmap_clear_and_realloc_verified` [test_bitmap_clear_and_realloc_verified_verus.rs](test_bitmap_clear_and_realloc_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_bitmap_multiple_alloc_verified` [test_bitmap_multiple_alloc_verified_verus.rs](test_bitmap_multiple_alloc_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_bitmap_number_of_bits_constant_verified` [test_bitmap_number_of_bits_constant_verified_verus.rs](test_bitmap_number_of_bits_constant_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_bitmap_set_clear_verified` [test_bitmap_set_clear_verified_verus.rs](test_bitmap_set_clear_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_bitmap_usage_tracking_verified` [test_bitmap_usage_tracking_verified_verus.rs](test_bitmap_usage_tracking_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_set_and_clear_all_bits_verified` [test_set_and_clear_all_bits_verified_verus.rs](test_set_and_clear_all_bits_verified_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `test_unchecked` [test_unchecked_verus.rs](test_unchecked_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `usage` [usage_verus.rs](usage_verus.rs) | EXTRA_IN_VERUS | ❌ |
