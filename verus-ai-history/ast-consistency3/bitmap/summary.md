# Exec Diff: lib

**Source:** `src/libs/bitmap/src/lib.rs`
**Verus:** `verus/split/libs/bitmap/lib.rs`

## Full Diffs (source vs Verus with spec/proof)

Directory: `full/`

| Function | Status | Files |
|----------|--------|-------|
| `Bitmap::alloc_range` | MISMATCH | Bitmap__alloc_range_source.rs, Bitmap__alloc_range_verus.rs, Bitmap__alloc_range.diff |
| `Bitmap::clear` | MISMATCH | Bitmap__clear_source.rs, Bitmap__clear_verus.rs, Bitmap__clear.diff |
| `Bitmap::from_raw_array` | MISMATCH | Bitmap__from_raw_array_source.rs, Bitmap__from_raw_array_verus.rs, Bitmap__from_raw_array.diff |
| `Bitmap::index` | MISMATCH | Bitmap__index_source.rs, Bitmap__index_verus.rs, Bitmap__index.diff |
| `Bitmap::index_unchecked` | MISMATCH | Bitmap__index_unchecked_source.rs, Bitmap__index_unchecked_verus.rs, Bitmap__index_unchecked.diff |
| `Bitmap::new` | MISMATCH | Bitmap__new_source.rs, Bitmap__new_verus.rs, Bitmap__new.diff |
| `Bitmap::set` | MISMATCH | Bitmap__set_source.rs, Bitmap__set_verus.rs, Bitmap__set.diff |
| `Bitmap::test` | MISMATCH | Bitmap__test_source.rs, Bitmap__test_verus.rs, Bitmap__test.diff |

## Exec-Only Diffs (source vs Verus stripped of ghost/proof)

Directory: `exec-only/`

These diffs show only the executable code differences, with all Verus
annotations (requires/ensures, proof blocks, ghost variables, invariants)
removed. This makes it easier to spot real exec logic changes.

| Function | Status | Files |
|----------|--------|-------|
| `Bitmap::alloc_range` | MISMATCH | Bitmap__alloc_range_source.rs, Bitmap__alloc_range_verus_stripped.rs, Bitmap__alloc_range.diff |
| `Bitmap::clear` | MISMATCH | Bitmap__clear_source.rs, Bitmap__clear_verus_stripped.rs, Bitmap__clear.diff |
| `Bitmap::from_raw_array` | MISMATCH | Bitmap__from_raw_array_source.rs, Bitmap__from_raw_array_verus_stripped.rs, Bitmap__from_raw_array.diff |
| `Bitmap::index` | MISMATCH | Bitmap__index_source.rs, Bitmap__index_verus_stripped.rs, Bitmap__index.diff |
| `Bitmap::index_unchecked` | MISMATCH | Bitmap__index_unchecked_source.rs, Bitmap__index_unchecked_verus_stripped.rs, Bitmap__index_unchecked.diff |
| `Bitmap::new` | MISMATCH | Bitmap__new_source.rs, Bitmap__new_verus_stripped.rs, Bitmap__new.diff |
| `Bitmap::set` | MISMATCH | Bitmap__set_source.rs, Bitmap__set_verus_stripped.rs, Bitmap__set.diff |
| `Bitmap::test` | MISMATCH | Bitmap__test_source.rs, Bitmap__test_verus_stripped.rs, Bitmap__test.diff |
