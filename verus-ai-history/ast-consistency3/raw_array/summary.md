# Exec Diff: lib

**Source:** `/home/ubuntu/nanvix-dev/src/libs/raw-array/src/lib.rs`
**Verus:** `verus/split/libs/raw_array/lib.rs`

## Full Diffs (source vs Verus with spec/proof)

Directory: `full/`

| Function | Status | Files |
|----------|--------|-------|
| `deref_mut` | MISSING_IN_VERUS | deref_mut_source.rs (MISSING in verus) |
| `drop` | MISMATCH | drop_source.rs, drop_verus.rs, drop.diff |
| `get` | MISMATCH | get_source.rs, get_verus.rs, get.diff |
| `new_managed` | MISMATCH | new_managed_source.rs, new_managed_verus.rs, new_managed.diff |
| `new_unmanaged` | MISMATCH | new_unmanaged_source.rs, new_unmanaged_verus.rs, new_unmanaged.diff |
| `from_raw_addr` | EXTRA_IN_VERUS | from_raw_addr_verus.rs (EXTRA) |
| `len` | EXTRA_IN_VERUS | len_verus.rs (EXTRA) |
| `set` | EXTRA_IN_VERUS | set_verus.rs (EXTRA) |
| `storage_len` | EXTRA_IN_VERUS | storage_len_verus.rs (EXTRA) |

## Exec-Only Diffs (source vs Verus stripped of ghost/proof)

Directory: `exec-only/`

These diffs show only the executable code differences, with all Verus
annotations (requires/ensures, proof blocks, ghost variables, invariants)
removed. This makes it easier to spot real exec logic changes.

| Function | Status | Files |
|----------|--------|-------|
| `deref_mut` | MISSING_IN_VERUS | deref_mut_source.rs (MISSING in verus) |
| `drop` | MISMATCH | drop_source.rs, drop_verus_stripped.rs, drop.diff |
| `get` | MISMATCH | get_source.rs, get_verus_stripped.rs, get.diff |
| `new_managed` | MISMATCH | new_managed_source.rs, new_managed_verus_stripped.rs, new_managed.diff |
| `new_unmanaged` | MISMATCH | new_unmanaged_source.rs, new_unmanaged_verus_stripped.rs, new_unmanaged.diff |
| `from_raw_addr` | EXTRA_IN_VERUS | from_raw_addr_verus.rs (EXTRA) |
| `len` | EXTRA_IN_VERUS | len_verus.rs (EXTRA) |
| `set` | EXTRA_IN_VERUS | set_verus.rs (EXTRA) |
| `storage_len` | EXTRA_IN_VERUS | storage_len_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ExRawArrayStorage` | EXTRA_IN_VERUS | struct_ExRawArrayStorage_verus.rs (EXTRA) |
| `RawArray` | MISMATCH | struct_RawArray_source.rs, struct_RawArray_verus.rs, struct_RawArray.diff |
