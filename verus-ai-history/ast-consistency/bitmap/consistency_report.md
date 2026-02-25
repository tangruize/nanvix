# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/libs/bitmap/src/lib.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/libs/bitmap/lib.rs`

## Summary

- Functions matched: 3/11
- Functions mismatched: 8
- Missing in Verus: 0
- Extra in Verus: 16
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `alloc_range` [alloc_range.diff](alloc_range.diff) [alloc_range_source.rs](alloc_range_source.rs) [alloc_range_verus.rs](alloc_range_verus.rs) | MISMATCH | 161-221 | 210-566 |
| `clear` [clear.diff](clear.diff) [clear_source.rs](clear_source.rs) [clear_verus.rs](clear_verus.rs) | MISMATCH | 261-271 | 651-719 |
| `from_raw_array` [from_raw_array.diff](from_raw_array.diff) [from_raw_array_source.rs](from_raw_array_source.rs) [from_raw_array_verus.rs](from_raw_array_verus.rs) | MISMATCH | 105-118 | 130-152 |
| `index` [index.diff](index.diff) [index_source.rs](index_source.rs) [index_verus.rs](index_verus.rs) | MISMATCH | 306-314 | 867-887 |
| `index_unchecked` [index_unchecked.diff](index_unchecked.diff) [index_unchecked_source.rs](index_unchecked_source.rs) [index_unchecked_verus.rs](index_unchecked_verus.rs) | MISMATCH | 329-333 | 852-864 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 63-85 | 57-103 |
| `set` [set.diff](set.diff) [set_source.rs](set_source.rs) [set_verus.rs](set_verus.rs) | MISMATCH | 236-246 | 569-648 |
| `test` [test.diff](test.diff) [test_source.rs](test_source.rs) [test_verus.rs](test_verus.rs) | MISMATCH | 287-290 | 829-845 |
| `clear_range` [clear_range_verus.rs](clear_range_verus.rs) | EXTRA_IN_VERUS |  | 723-826 |
| `new_managed` [new_managed_verus.rs](new_managed_verus.rs) | EXTRA_IN_VERUS |  | 106-122 |
| `test_alloc_and_clear_all_bits_verified` [test_alloc_and_clear_all_bits_verified_verus.rs](test_alloc_and_clear_all_bits_verified_verus.rs) | EXTRA_IN_VERUS |  | 1063-1119 |
| `test_alloc_in_partial_bitmap_verified` [test_alloc_in_partial_bitmap_verified_verus.rs](test_alloc_in_partial_bitmap_verified_verus.rs) | EXTRA_IN_VERUS |  | 1160-1193 |
| `test_alloc_range_across_word_boundary_verified` [test_alloc_range_across_word_boundary_verified_verus.rs](test_alloc_range_across_word_boundary_verified_verus.rs) | EXTRA_IN_VERUS |  | 1122-1157 |
| `test_alloc_range_and_clear_verified` [test_alloc_range_and_clear_verified_verus.rs](test_alloc_range_and_clear_verified_verus.rs) | EXTRA_IN_VERUS |  | 1196-1238 |
| `test_bitmap_alloc_range_preserves_others_verified` [test_bitmap_alloc_range_preserves_others_verified_verus.rs](test_bitmap_alloc_range_preserves_others_verified_verus.rs) | EXTRA_IN_VERUS |  | 1280-1304 |
| `test_bitmap_alloc_range_verified` [test_bitmap_alloc_range_verified_verus.rs](test_bitmap_alloc_range_verified_verus.rs) | EXTRA_IN_VERUS |  | 933-952 |
| `test_bitmap_clear_and_realloc_verified` [test_bitmap_clear_and_realloc_verified_verus.rs](test_bitmap_clear_and_realloc_verified_verus.rs) | EXTRA_IN_VERUS |  | 978-1001 |
| `test_bitmap_multiple_alloc_verified` [test_bitmap_multiple_alloc_verified_verus.rs](test_bitmap_multiple_alloc_verified_verus.rs) | EXTRA_IN_VERUS |  | 955-975 |
| `test_bitmap_number_of_bits_constant_verified` [test_bitmap_number_of_bits_constant_verified_verus.rs](test_bitmap_number_of_bits_constant_verified_verus.rs) | EXTRA_IN_VERUS |  | 1307-1337 |
| `test_bitmap_set_clear_verified` [test_bitmap_set_clear_verified_verus.rs](test_bitmap_set_clear_verified_verus.rs) | EXTRA_IN_VERUS |  | 907-930 |
| `test_bitmap_usage_tracking_verified` [test_bitmap_usage_tracking_verified_verus.rs](test_bitmap_usage_tracking_verified_verus.rs) | EXTRA_IN_VERUS |  | 1241-1277 |
| `test_set_and_clear_all_bits_verified` [test_set_and_clear_all_bits_verified_verus.rs](test_set_and_clear_all_bits_verified_verus.rs) | EXTRA_IN_VERUS |  | 1004-1060 |
| `test_unchecked` [test_unchecked_verus.rs](test_unchecked_verus.rs) | EXTRA_IN_VERUS |  | 890-899 |
| `usage` [usage_verus.rs](usage_verus.rs) | EXTRA_IN_VERUS |  | 167-175 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `alloc` | MATCH | ✅ |
| `alloc_range` | MISMATCH | ❌ |
| `clear` | MISMATCH | ❌ |
| `deref` | MATCH | ✅ |
| `from_raw_array` | MISMATCH | ❌ |
| `index` | MISMATCH | ❌ |
| `index_unchecked` | MISMATCH | ❌ |
| `new` | MISMATCH | ❌ |
| `number_of_bits` | MATCH | ✅ |
| `set` | MISMATCH | ❌ |
| `test` | MISMATCH | ❌ |
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
