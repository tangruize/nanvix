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
| `Bitmap::alloc_range` [Bitmap__alloc_range.diff](Bitmap__alloc_range.diff) [Bitmap__alloc_range_source.rs](Bitmap__alloc_range_source.rs) [Bitmap__alloc_range_verus.rs](Bitmap__alloc_range_verus.rs) | MISMATCH | 161-221 | 245-471 |
| `Bitmap::clear` [Bitmap__clear.diff](Bitmap__clear.diff) [Bitmap__clear_source.rs](Bitmap__clear_source.rs) [Bitmap__clear_verus.rs](Bitmap__clear_verus.rs) | MISMATCH | 261-271 | 551-601 |
| `Bitmap::from_raw_array` [Bitmap__from_raw_array.diff](Bitmap__from_raw_array.diff) [Bitmap__from_raw_array_source.rs](Bitmap__from_raw_array_source.rs) [Bitmap__from_raw_array_verus.rs](Bitmap__from_raw_array_verus.rs) | MISMATCH | 105-118 | 133-169 |
| `Bitmap::index` [Bitmap__index.diff](Bitmap__index.diff) [Bitmap__index_source.rs](Bitmap__index_source.rs) [Bitmap__index_verus.rs](Bitmap__index_verus.rs) | MISMATCH | 306-314 | 680-700 |
| `Bitmap::index_unchecked` [Bitmap__index_unchecked.diff](Bitmap__index_unchecked.diff) [Bitmap__index_unchecked_source.rs](Bitmap__index_unchecked_source.rs) [Bitmap__index_unchecked_verus.rs](Bitmap__index_unchecked_verus.rs) | MISMATCH | 329-333 | 652-664 |
| `Bitmap::new` [Bitmap__new.diff](Bitmap__new.diff) [Bitmap__new_source.rs](Bitmap__new_source.rs) [Bitmap__new_verus.rs](Bitmap__new_verus.rs) | MISMATCH | 63-85 | 73-113 |
| `Bitmap::set` [Bitmap__set.diff](Bitmap__set.diff) [Bitmap__set_source.rs](Bitmap__set_source.rs) [Bitmap__set_verus.rs](Bitmap__set_verus.rs) | MISMATCH | 236-246 | 486-536 |
| `Bitmap::test` [Bitmap__test.diff](Bitmap__test.diff) [Bitmap__test_source.rs](Bitmap__test_source.rs) [Bitmap__test_verus.rs](Bitmap__test_verus.rs) | MISMATCH | 287-290 | 617-633 |

## All Functions

| Function | Status | Hash Match | Verification |
|----------|--------|------------|--------------|
| `Bitmap::alloc` | MATCH | ✅ | ✅ verified |
| `Bitmap::alloc_range` | MISMATCH | ❌ | ✅ verified |
| `Bitmap::clear` | MISMATCH | ❌ | ✅ verified |
| `Bitmap::deref` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `Bitmap::from_raw_array` | MISMATCH | ❌ | ✅ verified |
| `Bitmap::index` | MISMATCH | ❌ | ✅ verified |
| `Bitmap::index_unchecked` | MISMATCH | ❌ | ✅ verified |
| `Bitmap::new` | MISMATCH | ❌ | ✅ verified |
| `Bitmap::number_of_bits` | MATCH | ✅ | ✅ verified |
| `Bitmap::set` | MISMATCH | ❌ | ✅ verified |
| `Bitmap::test` | MISMATCH | ❌ | ✅ verified |

## Verification Coverage

**⚠️ 1 function(s) are UNVERIFIED** (outside `verus!` block):

- `Bitmap::deref` (lines 714-716)

These functions are not checked by Verus at all. Justify why each
cannot be verified, or move them inside `verus!` with proper contracts.
