# Exec Diff: lib

**Source:** `src/libs/raw-array/src/lib.rs`
**Verus:** `verus/split/libs/raw_array/lib.rs`

| Function | Status | Files |
|----------|--------|-------|
| `deref` | MISMATCH | deref_source.rs, deref_verus.rs, deref.diff |
| `deref_mut` | MISSING_IN_VERUS | deref_mut_source.rs (MISSING in verus) |
| `drop` | MISMATCH | drop_source.rs, drop_verus.rs, drop.diff |
| `from_raw_parts` | MISMATCH | from_raw_parts_source.rs, from_raw_parts_verus.rs, from_raw_parts.diff |
| `get` | MISMATCH | get_source.rs, get_verus.rs, get.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `new_managed` | MISMATCH | new_managed_source.rs, new_managed_verus.rs, new_managed.diff |
| `new_unmanaged` | MISMATCH | new_unmanaged_source.rs, new_unmanaged_verus.rs, new_unmanaged.diff |
| `from_raw_addr` | EXTRA_IN_VERUS | from_raw_addr_verus.rs (EXTRA) |
| `len` | EXTRA_IN_VERUS | len_verus.rs (EXTRA) |
| `set` | EXTRA_IN_VERUS | set_verus.rs (EXTRA) |
| `storage_len` | EXTRA_IN_VERUS | storage_len_verus.rs (EXTRA) |

## Struct Issues

- `ExRawArrayStorage`: EXTRA_IN_VERUS
- `RawArray`: MISMATCH
