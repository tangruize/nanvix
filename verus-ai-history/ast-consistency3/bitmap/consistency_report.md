# Exec Consistency Report

**Source:** `src/libs/bitmap/src/lib.rs`
**Verus:** `verus/split/libs/bitmap/lib.rs`

## Summary

- Functions matched: 3/11
- Functions mismatched: 8
- Missing in Verus: 0
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `alloc_range` [alloc_range.diff](alloc_range.diff) [alloc_range_source.rs](alloc_range_source.rs) [alloc_range_verus.rs](alloc_range_verus.rs) | MISMATCH | 161-221 | 197-565 |
| `clear` [clear.diff](clear.diff) [clear_source.rs](clear_source.rs) [clear_verus.rs](clear_verus.rs) | MISMATCH | 261-271 | 650-718 |
| `from_raw_array` [from_raw_array.diff](from_raw_array.diff) [from_raw_array_source.rs](from_raw_array_source.rs) [from_raw_array_verus.rs](from_raw_array_verus.rs) | MISMATCH | 105-118 | 115-151 |
| `index` [index.diff](index.diff) [index_source.rs](index_source.rs) [index_verus.rs](index_verus.rs) | MISMATCH | 306-314 | 759-779 |
| `index_unchecked` [index_unchecked.diff](index_unchecked.diff) [index_unchecked_source.rs](index_unchecked_source.rs) [index_unchecked_verus.rs](index_unchecked_verus.rs) | MISMATCH | 329-333 | 744-756 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 63-85 | 57-103 |
| `set` [set.diff](set.diff) [set_source.rs](set_source.rs) [set_verus.rs](set_verus.rs) | MISMATCH | 236-246 | 568-647 |
| `test` [test.diff](test.diff) [test_source.rs](test_source.rs) [test_verus.rs](test_verus.rs) | MISMATCH | 287-290 | 721-737 |

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
