# Exec Consistency Report

**Source:** `src/libs/error/src/lib.rs`
**Verus:** `verus/split/libs/error/lib.rs`

## Summary

- Functions matched: 8/10
- Functions mismatched: 2
- Missing in Verus: 0
- Extra in Verus: 1
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `Error::new` [Error__new.diff](Error__new.diff) [Error__new_source.rs](Error__new_source.rs) [Error__new_verus.rs](Error__new_verus.rs) | MISMATCH | 460-462 | 598-604 |
| `ErrorCode::try_from` [ErrorCode__try_from.diff](ErrorCode__try_from.diff) [ErrorCode__try_from_source.rs](ErrorCode__try_from_source.rs) [ErrorCode__try_from_verus.rs](ErrorCode__try_from_verus.rs) | MISMATCH | 510-519 | 665-671 |
| `Error::log` [Error__log_verus.rs](Error__log_verus.rs) | EXTRA_IN_VERUS |  | 613-615 |

## All Functions

| Function | Status | Hash Match | Verification |
|----------|--------|------------|--------------|
| `Error::new` | MISMATCH | ❌ | 🔒 external_body |
| `ErrorCode::fmt` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `ErrorCode::get` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `ErrorCode::try_from` | MISMATCH | ❌ | ⚠️ UNVERIFIED |
| `i16::from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `i32::from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `i64::from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `invalid_error_code` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `u16::from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `u32::from` | MATCH | ✅ | ⚠️ UNVERIFIED |
| `Error::log` [Error__log_verus.rs](Error__log_verus.rs) | EXTRA_IN_VERUS | ❌ | 🔒 external_body |

## Verification Coverage

**⚠️ 9 function(s) are UNVERIFIED** (outside `verus!` block):

- `ErrorCode::fmt` (lines 627-629)
- `ErrorCode::get` (lines 419-421)
- `ErrorCode::try_from` (lines 665-671)
- `i16::from` (lines 651-653)
- `i32::from` (lines 639-641)
- `i64::from` (lines 645-647)
- `invalid_error_code` (lines 577-582)
- `u16::from` (lines 657-659)
- `u32::from` (lines 633-635)

These functions are not checked by Verus at all. Justify why each
cannot be verified, or move them inside `verus!` with proper contracts.

**🔒 2 function(s) use `external_body`** (body not verified):

- `Error::new` (lines 598-604)
- `Error::log` (lines 613-615)

These functions have requires/ensures contracts but the body is trusted.
Justify why `external_body` is necessary for each.
