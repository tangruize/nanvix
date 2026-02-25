# Exec Consistency Report

**Source:** `src/libs/raw-array/src/lib.rs`
**Verus:** `verus/split/libs/raw_array/lib.rs`

## Summary

- Functions matched: 9/9
- Functions mismatched: 0
- Missing in Verus: 0
- Extra in Verus: 1
- **Consistent: YES**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `RawArray::set` [RawArray__set_verus.rs](RawArray__set_verus.rs) | EXTRA_IN_VERUS |  | 281-291 |

## All Functions

| Function | Status | Hash Match | Verification |
|----------|--------|------------|--------------|
| `RawArray::deref` | MATCH | ✅ | 🔒 external_body |
| `RawArray::deref_mut` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArray::drop` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArray::from_raw_parts` | MATCH | ✅ | 🔒 external_body |
| `RawArray::new` | MATCH | ✅ | 🔒 external_body |
| `RawArrayStorage::get` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArrayStorage::get_mut` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArrayStorage::new_managed` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArrayStorage::new_unmanaged` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `RawArray::set` [RawArray__set_verus.rs](RawArray__set_verus.rs) | EXTRA_IN_VERUS | ❌ | 🔒 external_body |

## Verification Coverage

**⚠️ 6 function(s) are UNVERIFIED** (outside `verus!` block):

- `RawArray::deref_mut` (lines 315-317)
- `RawArray::drop` (lines 321-334)
- `RawArrayStorage::get` (lines 160-169)
- `RawArrayStorage::get_mut` (lines 140-149)
- `RawArrayStorage::new_managed` (lines 57-83)
- `RawArrayStorage::new_unmanaged` (lines 108-129)

These functions are not checked by Verus at all. Justify why each
cannot be verified, or move them inside `verus!` with proper contracts.

**🔒 4 function(s) use `external_body`** (body not verified):

- `RawArray::deref` (lines 302-307)
- `RawArray::from_raw_parts` (lines 254-267)
- `RawArray::new` (lines 215-228)
- `RawArray::set` (lines 281-291)

These functions have requires/ensures contracts but the body is trusted.
Justify why `external_body` is necessary for each.

## Inconsistent Structs

| Struct | Status | Source Lines | Verus Lines |
|--------|--------|-------------|-------------|
| `ExRawArrayStorage` | EXTRA_IN_VERUS |  | 182-182 |
