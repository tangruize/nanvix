# Exec Consistency Report

**Source:** `src/libs/error/src/lib.rs`
**Verus:** `verus/split/libs/error/lib.rs`

## Summary

- Functions matched: 1/6
- Functions mismatched: 2
- Missing in Verus: 3
- Extra in Verus: 2
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `fmt` [fmt.diff](fmt.diff) | [fmt_source.rs](fmt_source.rs) | [fmt_verus.rs](fmt_verus.rs) | MISMATCH | 461-463 | 75-77 |
| `from` [from_source.rs](from_source.rs) | MISSING_IN_VERUS | 491-493 |  |
| `invalid_error_code` [invalid_error_code_source.rs](invalid_error_code_source.rs) | MISSING_IN_VERUS | 435-440 |  |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 449-451 | 52-58 |
| `try_from` [try_from_source.rs](try_from_source.rs) | MISSING_IN_VERUS | 499-508 |  |
| `log` [log_verus.rs](log_verus.rs) | EXTRA_IN_VERUS |  | 62-64 |
| `log_error_with_context` [log_error_with_context_verus.rs](log_error_with_context_verus.rs) | EXTRA_IN_VERUS |  | 69-71 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `fmt` [fmt.diff](fmt.diff) | [fmt_source.rs](fmt_source.rs) | [fmt_verus.rs](fmt_verus.rs) | MISMATCH | ❌ |
| `from` [from_source.rs](from_source.rs) | MISSING_IN_VERUS | ❌ |
| `get` | MATCH | ✅ |
| `invalid_error_code` [invalid_error_code_source.rs](invalid_error_code_source.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `try_from` [try_from_source.rs](try_from_source.rs) | MISSING_IN_VERUS | ❌ |
| `log` [log_verus.rs](log_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `log_error_with_context` [log_error_with_context_verus.rs](log_error_with_context_verus.rs) | EXTRA_IN_VERUS | ❌ |
