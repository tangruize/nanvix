# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/libs/raw-array/src/lib.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/libs/raw_array/lib.rs`

## Summary

- Functions matched: 7/9
- Functions mismatched: 2
- Missing in Verus: 0
- Extra in Verus: 4
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `drop` [drop.diff](drop.diff) [drop_source.rs](drop_source.rs) [drop_verus.rs](drop_verus.rs) | MISMATCH | 275-288 | 124-137 |
| `get` [get.diff](get.diff) [get_source.rs](get_source.rs) [get_verus.rs](get_verus.rs) | MISMATCH | 182-191 | 253-260 |
| `from_raw_addr` [from_raw_addr_verus.rs](from_raw_addr_verus.rs) | EXTRA_IN_VERUS |  | 201-215 |
| `len` [len_verus.rs](len_verus.rs) | EXTRA_IN_VERUS |  | 244-249 |
| `set` [set_verus.rs](set_verus.rs) | EXTRA_IN_VERUS |  | 230-240 |
| `storage_len` [storage_len_verus.rs](storage_len_verus.rs) | EXTRA_IN_VERUS |  | 111-116 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `deref` | MATCH | ✅ |
| `deref_mut` | MATCH | ✅ |
| `drop` | MISMATCH | ❌ |
| `from_raw_parts` | MATCH | ✅ |
| `get` | MISMATCH | ❌ |
| `get_mut` | MATCH | ✅ |
| `new` | MATCH | ✅ |
| `new_managed` | MATCH | ✅ |
| `new_unmanaged` | MATCH | ✅ |
| `from_raw_addr` [from_raw_addr_verus.rs](from_raw_addr_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `len` [len_verus.rs](len_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `set` [set_verus.rs](set_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `storage_len` [storage_len_verus.rs](storage_len_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

| Struct | Status | Source Lines | Verus Lines |
|--------|--------|-------------|-------------|
| `ExRawArrayStorage` | EXTRA_IN_VERUS |  | 150-150 |
| `RawArray` | MISMATCH | 204-207 | 154-156 |
