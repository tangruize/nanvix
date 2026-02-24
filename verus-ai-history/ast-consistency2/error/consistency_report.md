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
| `fmt` [diff](fmt.diff) [source](fmt_source.rs) [verus](fmt_verus.rs) | MISMATCH | 461-463 | 75-77 |
| `from` [source](from_source.rs) | MISSING_IN_VERUS | 491-493 |  |
| `invalid_error_code` [source](invalid_error_code_source.rs) | MISSING_IN_VERUS | 435-440 |  |
| `new` [diff](new.diff) [source](new_source.rs) [verus](new_verus.rs) | MISMATCH | 449-451 | 52-58 |
| `try_from` [source](try_from_source.rs) | MISSING_IN_VERUS | 499-508 |  |
| `log` [verus](log_verus.rs) | EXTRA_IN_VERUS |  | 62-64 |
| `log_error_with_context` [verus](log_error_with_context_verus.rs) | EXTRA_IN_VERUS |  | 69-71 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `fmt` [diff](fmt.diff) [source](fmt_source.rs) [verus](fmt_verus.rs) | MISMATCH | ❌ |
| `from` [source](from_source.rs) | MISSING_IN_VERUS | ❌ |
| `get` | MATCH | ✅ |
| `invalid_error_code` [source](invalid_error_code_source.rs) | MISSING_IN_VERUS | ❌ |
| `new` [diff](new.diff) [source](new_source.rs) [verus](new_verus.rs) | MISMATCH | ❌ |
| `try_from` [source](try_from_source.rs) | MISSING_IN_VERUS | ❌ |
| `log` [verus](log_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `log_error_with_context` [verus](log_error_with_context_verus.rs) | EXTRA_IN_VERUS | ❌ |
