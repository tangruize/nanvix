# Exec Consistency Report

**Source:** `src/libs/error/src/lib.rs`
**Verus:** `verus/split/libs/error/lib.rs`

## Summary

- Functions matched: 10/10
- Functions mismatched: 0
- Missing in Verus: 0
- Extra in Verus: 1
- **Consistent: YES**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `Error::log` [Error__log_verus.rs](Error__log_verus.rs) | EXTRA_IN_VERUS |  | 313-314 |

## All Functions

| Function | Status | Hash Match | Verification |
|----------|--------|------------|--------------|
| `Error::new` | MATCH | ✅ | ✅ verified |
| `ErrorCode::fmt` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `ErrorCode::get` | MATCH | ✅ | ✅ verified |
| `ErrorCode::try_from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `i16::from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `i32::from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `i64::from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `invalid_error_code` | MATCH | ✅ | ✅ verified |
| `u16::from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `u32::from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `Error::log` [Error__log_verus.rs](Error__log_verus.rs) | EXTRA_IN_VERUS | ❌ | 🔒 external_body |

## Verification Coverage

**⚠️ 7 function(s) are UNVERIFIED** (outside `verus!` block):

- `ErrorCode::fmt` (lines 352-354)
- `ErrorCode::try_from` (lines 540-549)
- `i16::from` (lines 380-382)
- `i32::from` (lines 366-368)
- `i64::from` (lines 373-375)
- `u16::from` (lines 387-389)
- `u32::from` (lines 359-361)

These functions are not checked by Verus at all. Justify why each
cannot be verified, or move them inside `verus!` with proper contracts.

**🔒 1 function(s) use `external_body`** (body not verified):

- `Error::log` (lines 313-314)

These functions have requires/ensures contracts but the body is trusted.
Justify why `external_body` is necessary for each.
