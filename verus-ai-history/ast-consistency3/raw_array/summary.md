# Exec Diff: lib

**Source:** `src/libs/raw-array/src/lib.rs`
**Verus:** `verus/split/libs/raw_array/lib.rs`

## Full Diffs (source vs Verus with spec/proof)

Directory: `full/`

| Function | Status | Files |
|----------|--------|-------|
| `RawArray::drop` | MISSING_IN_VERUS | RawArray__drop_source.rs (MISSING in verus) |
| `RawArray::get` | EXTRA_IN_VERUS | RawArray__get_verus.rs (EXTRA) |
| `RawArray::len` | EXTRA_IN_VERUS | RawArray__len_verus.rs (EXTRA) |
| `RawArray::set` | EXTRA_IN_VERUS | RawArray__set_verus.rs (EXTRA) |
| `RawArrayStorage::drop` | EXTRA_IN_VERUS | RawArrayStorage__drop_verus.rs (EXTRA) |

## Exec-Only Diffs (source vs Verus stripped of ghost/proof)

Directory: `exec-only/`

These diffs show only the executable code differences, with all Verus
annotations (requires/ensures, proof blocks, ghost variables, invariants)
removed. This makes it easier to spot real exec logic changes.

| Function | Status | Files |
|----------|--------|-------|
| `RawArray::drop` | MISSING_IN_VERUS | RawArray__drop_source.rs (MISSING in verus) |
| `RawArray::get` | EXTRA_IN_VERUS | RawArray__get_verus.rs (EXTRA) |
| `RawArray::len` | EXTRA_IN_VERUS | RawArray__len_verus.rs (EXTRA) |
| `RawArray::set` | EXTRA_IN_VERUS | RawArray__set_verus.rs (EXTRA) |
| `RawArrayStorage::drop` | EXTRA_IN_VERUS | RawArrayStorage__drop_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ExRawArrayStorage` | EXTRA_IN_VERUS | struct_ExRawArrayStorage_verus.rs (EXTRA) |
| `RawArray` | MISMATCH | struct_RawArray_source.rs, struct_RawArray_verus.rs, struct_RawArray.diff |
