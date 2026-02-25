# Exec Consistency Report

**Source:** `src/libs/error/src/lib.rs`
**Verus:** `verus/split/libs/error/lib.rs`

## Summary

- Functions matched: 2/10
- Functions mismatched: 0
- Missing in Verus: 8
- Extra in Verus: 3
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `ErrorCode::fmt` [ErrorCode__fmt_source.rs](ErrorCode__fmt_source.rs) | MISSING_IN_VERUS | 472-474 |  |
| `ErrorCode::try_from` [ErrorCode__try_from_source.rs](ErrorCode__try_from_source.rs) | MISSING_IN_VERUS | 510-519 |  |
| `i16::from` [i16__from_source.rs](i16__from_source.rs) | MISSING_IN_VERUS | 496-498 |  |
| `i32::from` [i32__from_source.rs](i32__from_source.rs) | MISSING_IN_VERUS | 484-486 |  |
| `i64::from` [i64__from_source.rs](i64__from_source.rs) | MISSING_IN_VERUS | 490-492 |  |
| `invalid_error_code` [invalid_error_code_source.rs](invalid_error_code_source.rs) | MISSING_IN_VERUS | 446-451 |  |
| `u16::from` [u16__from_source.rs](u16__from_source.rs) | MISSING_IN_VERUS | 502-504 |  |
| `u32::from` [u32__from_source.rs](u32__from_source.rs) | MISSING_IN_VERUS | 478-480 |  |
| `Error::fmt` [Error__fmt_verus.rs](Error__fmt_verus.rs) | EXTRA_IN_VERUS |  | 75-77 |
| `Error::log` [Error__log_verus.rs](Error__log_verus.rs) | EXTRA_IN_VERUS |  | 62-64 |
| `log_error_with_context` [log_error_with_context_verus.rs](log_error_with_context_verus.rs) | EXTRA_IN_VERUS |  | 69-71 |

## All Functions

| Function | Status | Hash Match | Verification |
|----------|--------|------------|--------------|
| `Error::new` | MATCH | ✅ | ✅ verified |
| `ErrorCode::fmt` | MISSING_IN_VERUS | ❌ |  |
| `ErrorCode::get` | MATCH | ✅ | ✅ verified |
| `ErrorCode::try_from` | MISSING_IN_VERUS | ❌ |  |
| `i16::from` | MISSING_IN_VERUS | ❌ |  |
| `i32::from` | MISSING_IN_VERUS | ❌ |  |
| `i64::from` | MISSING_IN_VERUS | ❌ |  |
| `invalid_error_code` | MISSING_IN_VERUS | ❌ |  |
| `u16::from` | MISSING_IN_VERUS | ❌ |  |
| `u32::from` | MISSING_IN_VERUS | ❌ |  |
| `Error::fmt` [Error__fmt_verus.rs](Error__fmt_verus.rs) | EXTRA_IN_VERUS | ❌ | ✅ verified |
| `Error::log` [Error__log_verus.rs](Error__log_verus.rs) | EXTRA_IN_VERUS | ❌ | 🔒 external_body |
| `log_error_with_context` [log_error_with_context_verus.rs](log_error_with_context_verus.rs) | EXTRA_IN_VERUS | ❌ | 🔒 external_body |

## Verification Coverage

**🔒 2 function(s) use `external_body`** (body not verified):

- `Error::log` (lines 62-64)
- `log_error_with_context` (lines 69-71)

These functions have requires/ensures contracts but the body is trusted.
Justify why `external_body` is necessary for each.
