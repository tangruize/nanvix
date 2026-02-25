# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/libs/raw_array/src/lib.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/libs/raw_array/lib.rs`

## Summary

- Functions matched: 4/9
- Functions mismatched: 4
- Missing in Verus: 1
- Extra in Verus: 4
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `deref_mut` [deref_mut_source.rs](deref_mut_source.rs) | MISSING_IN_VERUS | 269-271 |  |
| `drop` [drop.diff](drop.diff) [drop_source.rs](drop_source.rs) [drop_verus.rs](drop_verus.rs) | MISMATCH | 275-288 | 101-110 |
| `get` [get.diff](get.diff) [get_source.rs](get_source.rs) [get_verus.rs](get_verus.rs) | MISMATCH | 182-191 | 219-226 |
| `new_managed` [new_managed.diff](new_managed.diff) [new_managed_source.rs](new_managed_source.rs) [new_managed_verus.rs](new_managed_verus.rs) | MISMATCH | 79-105 | 36-53 |
| `new_unmanaged` [new_unmanaged.diff](new_unmanaged.diff) [new_unmanaged_source.rs](new_unmanaged_source.rs) [new_unmanaged_verus.rs](new_unmanaged_verus.rs) | MISMATCH | 130-151 | 55-68 |
| `from_raw_addr` [from_raw_addr_verus.rs](from_raw_addr_verus.rs) | EXTRA_IN_VERUS |  | 172-186 |
| `len` [len_verus.rs](len_verus.rs) | EXTRA_IN_VERUS |  | 210-215 |
| `set` [set_verus.rs](set_verus.rs) | EXTRA_IN_VERUS |  | 196-206 |
| `storage_len` [storage_len_verus.rs](storage_len_verus.rs) | EXTRA_IN_VERUS |  | 92-97 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `deref` | MATCH | ✅ |
| `deref_mut` | MISSING_IN_VERUS | ❌ |
| `drop` | MISMATCH | ❌ |
| `from_raw_parts` | MATCH | ✅ |
| `get` | MISMATCH | ❌ |
| `get_mut` | MATCH | ✅ |
| `new` | MATCH | ✅ |
| `new_managed` | MISMATCH | ❌ |
| `new_unmanaged` | MISMATCH | ❌ |
| `from_raw_addr` [from_raw_addr_verus.rs](from_raw_addr_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `len` [len_verus.rs](len_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `set` [set_verus.rs](set_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `storage_len` [storage_len_verus.rs](storage_len_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

| Struct | Status | Source Lines | Verus Lines |
|--------|--------|-------------|-------------|
| `ExRawArrayStorage` | EXTRA_IN_VERUS |  | 123-123 |
| `RawArray` | MISMATCH | 204-207 | 127-129 |
