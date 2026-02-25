# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/libs/raw-array/src/lib.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/libs/raw_array/lib.rs`

## Summary

- Functions matched: 8/9
- Functions mismatched: 0
- Missing in Verus: 1
- Extra in Verus: 4
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `RawArray::drop` [RawArray__drop_source.rs](RawArray__drop_source.rs) | MISSING_IN_VERUS | 275-288 |  |
| `RawArray::get` [RawArray__get_verus.rs](RawArray__get_verus.rs) | EXTRA_IN_VERUS |  | 227-234 |
| `RawArray::len` [RawArray__len_verus.rs](RawArray__len_verus.rs) | EXTRA_IN_VERUS |  | 215-223 |
| `RawArray::set` [RawArray__set_verus.rs](RawArray__set_verus.rs) | EXTRA_IN_VERUS |  | 201-211 |
| `RawArrayStorage::drop` [RawArrayStorage__drop_verus.rs](RawArrayStorage__drop_verus.rs) | EXTRA_IN_VERUS |  | 115-128 |

## All Functions

| Function | Status | Hash Match | Verification |
|----------|--------|------------|--------------|
| `RawArray::deref` | MATCH | ✅ | 🔒 external_body |
| `RawArray::deref_mut` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArray::drop` | MISSING_IN_VERUS | ❌ |  |
| `RawArray::from_raw_parts` | MATCH | ✅ | 🔒 external_body |
| `RawArray::new` | MATCH | ✅ | 🔒 external_body |
| `RawArrayStorage::get` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArrayStorage::get_mut` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArrayStorage::new_managed` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArrayStorage::new_unmanaged` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArray::get` [RawArray__get_verus.rs](RawArray__get_verus.rs) | EXTRA_IN_VERUS | ❌ | 🔒 external_body |
| `RawArray::len` [RawArray__len_verus.rs](RawArray__len_verus.rs) | EXTRA_IN_VERUS | ❌ | 🔒 external_body |
| `RawArray::set` [RawArray__set_verus.rs](RawArray__set_verus.rs) | EXTRA_IN_VERUS | ❌ | 🔒 external_body |
| `RawArrayStorage::drop` [RawArrayStorage__drop_verus.rs](RawArrayStorage__drop_verus.rs) | EXTRA_IN_VERUS | ❌ | ⚠️ UNVERIFIED |

## Verification Coverage

**⚠️ 6 function(s) are UNVERIFIED** (outside `verus!` block):

- `RawArray::deref_mut` (lines 258-260)
- `RawArrayStorage::get` (lines 98-107)
- `RawArrayStorage::get_mut` (lines 87-96)
- `RawArrayStorage::new_managed` (lines 36-62)
- `RawArrayStorage::new_unmanaged` (lines 64-85)
- `RawArrayStorage::drop` (lines 115-128)

These functions are not checked by Verus at all. Justify why each
cannot be verified, or move them inside `verus!` with proper contracts.

**🔒 6 function(s) use `external_body`** (body not verified):

- `RawArray::deref` (lines 245-250)
- `RawArray::from_raw_parts` (lines 173-186)
- `RawArray::new` (lines 156-169)
- `RawArray::get` (lines 227-234)
- `RawArray::len` (lines 215-223)
- `RawArray::set` (lines 201-211)

These functions have requires/ensures contracts but the body is trusted.
Justify why `external_body` is necessary for each.

## Inconsistent Structs

| Struct | Status | Source Lines | Verus Lines |
|--------|--------|-------------|-------------|
| `ExRawArrayStorage` | EXTRA_IN_VERUS |  | 141-141 |
| `RawArray` | MISMATCH | 204-207 | 145-147 |
