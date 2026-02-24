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
| `deref` [deref.diff](deref.diff) [deref_source.rs](deref_source.rs) [deref_verus.rs](deref_verus.rs) | MISMATCH | 263-265 | 237-242 |
| `deref_mut` [deref_mut_source.rs](deref_mut_source.rs) | MISSING_IN_VERUS | 269-271 |  |
| `drop` [drop.diff](drop.diff) [drop_source.rs](drop_source.rs) [drop_verus.rs](drop_verus.rs) | MISMATCH | 275-288 | 101-110 |
| `from_raw_parts` [from_raw_parts.diff](from_raw_parts.diff) [from_raw_parts_source.rs](from_raw_parts_source.rs) [from_raw_parts_verus.rs](from_raw_parts_verus.rs) | MISMATCH | 253-257 | 155-168 |
| `get` [get.diff](get.diff) [get_source.rs](get_source.rs) [get_verus.rs](get_verus.rs) | MISMATCH | 182-191 | 219-226 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 224-228 | 138-151 |
| `new_managed` [new_managed.diff](new_managed.diff) [new_managed_source.rs](new_managed_source.rs) [new_managed_verus.rs](new_managed_verus.rs) | MISMATCH | 79-105 | 36-53 |
| `new_unmanaged` [new_unmanaged.diff](new_unmanaged.diff) [new_unmanaged_source.rs](new_unmanaged_source.rs) [new_unmanaged_verus.rs](new_unmanaged_verus.rs) | MISMATCH | 130-151 | 55-68 |
| `from_raw_addr` [from_raw_addr_verus.rs](from_raw_addr_verus.rs) | EXTRA_IN_VERUS |  | 172-186 |
| `len` [len_verus.rs](len_verus.rs) | EXTRA_IN_VERUS |  | 210-215 |
| `set` [set_verus.rs](set_verus.rs) | EXTRA_IN_VERUS |  | 196-206 |
| `storage_len` [storage_len_verus.rs](storage_len_verus.rs) | EXTRA_IN_VERUS |  | 92-97 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `deref` [deref.diff](deref.diff) [deref_source.rs](deref_source.rs) [deref_verus.rs](deref_verus.rs) | MISMATCH | ❌ |
| `deref_mut` [deref_mut_source.rs](deref_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `drop` [drop.diff](drop.diff) [drop_source.rs](drop_source.rs) [drop_verus.rs](drop_verus.rs) | MISMATCH | ❌ |
| `from_raw_parts` [from_raw_parts.diff](from_raw_parts.diff) [from_raw_parts_source.rs](from_raw_parts_source.rs) [from_raw_parts_verus.rs](from_raw_parts_verus.rs) | MISMATCH | ❌ |
| `get` [get.diff](get.diff) [get_source.rs](get_source.rs) [get_verus.rs](get_verus.rs) | MISMATCH | ❌ |
| `get_mut` | MATCH | ✅ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `new_managed` [new_managed.diff](new_managed.diff) [new_managed_source.rs](new_managed_source.rs) [new_managed_verus.rs](new_managed_verus.rs) | MISMATCH | ❌ |
| `new_unmanaged` [new_unmanaged.diff](new_unmanaged.diff) [new_unmanaged_source.rs](new_unmanaged_source.rs) [new_unmanaged_verus.rs](new_unmanaged_verus.rs) | MISMATCH | ❌ |
| `from_raw_addr` [from_raw_addr_verus.rs](from_raw_addr_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `len` [len_verus.rs](len_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `set` [set_verus.rs](set_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `storage_len` [storage_len_verus.rs](storage_len_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `ExRawArrayStorage` [struct_ExRawArrayStorage_verus.rs](struct_ExRawArrayStorage_verus.rs): EXTRA_IN_VERUS
- `RawArray` [struct_RawArray.diff](struct_RawArray.diff) [struct_RawArray_source.rs](struct_RawArray_source.rs) [struct_RawArray_verus.rs](struct_RawArray_verus.rs): MISMATCH
