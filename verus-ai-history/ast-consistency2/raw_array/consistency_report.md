# Exec Consistency Report

**Source:** `src/libs/raw-array/src/lib.rs`
**Verus:** `verus/split/libs/raw_array/lib.rs`

## Summary

- Functions matched: 1/9
- Functions mismatched: 7
- Missing in Verus: 1
- Extra in Verus: 4
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `deref` [diff](deref.diff) [source](deref_source.rs) [verus](deref_verus.rs) | MISMATCH | 263-265 | 237-242 |
| `deref_mut` [source](deref_mut_source.rs) | MISSING_IN_VERUS | 269-271 |  |
| `drop` [diff](drop.diff) [source](drop_source.rs) [verus](drop_verus.rs) | MISMATCH | 275-288 | 101-110 |
| `from_raw_parts` [diff](from_raw_parts.diff) [source](from_raw_parts_source.rs) [verus](from_raw_parts_verus.rs) | MISMATCH | 253-257 | 155-168 |
| `get` [diff](get.diff) [source](get_source.rs) [verus](get_verus.rs) | MISMATCH | 182-191 | 219-226 |
| `new` [diff](new.diff) [source](new_source.rs) [verus](new_verus.rs) | MISMATCH | 224-228 | 138-151 |
| `new_managed` [diff](new_managed.diff) [source](new_managed_source.rs) [verus](new_managed_verus.rs) | MISMATCH | 79-105 | 36-53 |
| `new_unmanaged` [diff](new_unmanaged.diff) [source](new_unmanaged_source.rs) [verus](new_unmanaged_verus.rs) | MISMATCH | 130-151 | 55-68 |
| `from_raw_addr` [verus](from_raw_addr_verus.rs) | EXTRA_IN_VERUS |  | 172-186 |
| `len` [verus](len_verus.rs) | EXTRA_IN_VERUS |  | 210-215 |
| `set` [verus](set_verus.rs) | EXTRA_IN_VERUS |  | 196-206 |
| `storage_len` [verus](storage_len_verus.rs) | EXTRA_IN_VERUS |  | 92-97 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `deref` [diff](deref.diff) [source](deref_source.rs) [verus](deref_verus.rs) | MISMATCH | ❌ |
| `deref_mut` [source](deref_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `drop` [diff](drop.diff) [source](drop_source.rs) [verus](drop_verus.rs) | MISMATCH | ❌ |
| `from_raw_parts` [diff](from_raw_parts.diff) [source](from_raw_parts_source.rs) [verus](from_raw_parts_verus.rs) | MISMATCH | ❌ |
| `get` [diff](get.diff) [source](get_source.rs) [verus](get_verus.rs) | MISMATCH | ❌ |
| `get_mut` | MATCH | ✅ |
| `new` [diff](new.diff) [source](new_source.rs) [verus](new_verus.rs) | MISMATCH | ❌ |
| `new_managed` [diff](new_managed.diff) [source](new_managed_source.rs) [verus](new_managed_verus.rs) | MISMATCH | ❌ |
| `new_unmanaged` [diff](new_unmanaged.diff) [source](new_unmanaged_source.rs) [verus](new_unmanaged_verus.rs) | MISMATCH | ❌ |
| `from_raw_addr` [verus](from_raw_addr_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `len` [verus](len_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `set` [verus](set_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `storage_len` [verus](storage_len_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `ExRawArrayStorage`: EXTRA_IN_VERUS
- `RawArray`: MISMATCH
