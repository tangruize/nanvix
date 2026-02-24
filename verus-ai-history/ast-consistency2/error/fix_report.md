# Exec Consistency Fix: error

## Summary
- Mismatches fixed: 0
- Missing functions added: 0
- Documented equivalences: 5

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | EQUIVALENT | Exec logic is identical: `Self { code, reason }`. The only difference is the Verus version adds `ensures result.code == code, result.reason == reason` postconditions, which are verification-only annotations and do not affect exec behavior. |
| `fmt` [fmt.diff](fmt.diff) [fmt_source.rs](fmt_source.rs) [fmt_verus.rs](fmt_verus.rs) | DOCUMENTED | The AST tool matched these as the same function, but they are Display impls on *different types*: source implements `Display for ErrorCode` with `write!(f, "error={self:?}")`, while Verus implements `Display for Error` with `write!(f, "{:?}: {}", self.code, self.reason)`. The source's `Display for ErrorCode` is absent in Verus because Verus has a reduced `ErrorCode` enum (6 variants vs 100+). The Verus `Display for Error` is an extra impl needed for error formatting in the verified codebase, correctly marked `#[verifier::external]`. Both are trait impls which require `external` annotation in Verus. |
| `from` [from_source.rs](from_source.rs) | DOCUMENTED | Multiple `From<ErrorCode>` trait impls (`for u32`, `for i32`, `for i64`, `for i16`, `for u16`) exist in source but are absent in Verus. Verus does not support trait impls like `From`/`Into`; callers in the verified codebase use `ErrorCode::get()` (which returns `i32`) or direct casts instead. |
| `invalid_error_code` [invalid_error_code_source.rs](invalid_error_code_source.rs) | DOCUMENTED | Private helper function that constructs `Error { code: ErrorCode::InvalidArgument, reason: "invalid error code" }`. It is only called by `TryFrom<i32> for ErrorCode`, which is also absent in Verus. Since Verus does not support `TryFrom` trait impls, this helper is unnecessary. Callers in the verified codebase use `Error::new(ErrorCode::InvalidArgument, ...)` directly. |
| `try_from` [try_from_source.rs](try_from_source.rs) | DOCUMENTED | Two `TryFrom` impls exist in source: `TryFrom<i32> for ErrorCode` (large match on ~100 variants) and `TryFrom<i64> for ErrorCode` (delegates to the i32 version). Verus does not support `TryFrom` trait impls. The Verus `ErrorCode` enum has only 6 variants, so the exhaustive match is unnecessary; callers construct `ErrorCode` values directly. |

## Extra Functions in Verus (EXTRA_IN_VERUS)
| Function | Action | Justification |
|----------|--------|---------------|
| `log` [log_verus.rs](log_verus.rs) | EXTRA_JUSTIFIED | Method on `Error` marked `#[verifier::external_body]` with an empty body. Provides a verification-compatible stub for error logging (replacing `error!()` macro calls which Verus cannot verify). No exec logic to compare. |
| `log_error_with_context` [log_error_with_context_verus.rs](log_error_with_context_verus.rs) | EXTRA_JUSTIFIED | Free function marked `#[verifier::external_body]` that accepts an `Error` and context string. Provides a verification-compatible stub for contextual error logging. Body is `let _ = (error, context);` (no-op). No exec logic to compare. |

## Notes
- The Verus `ErrorCode` enum contains only 6 variants (`InvalidArgument`, `OutOfMemory`, `ResourceBusy`, `BadAddress`, `NoSuchEntry`, `NoSuchProcess`) versus ~100 variants in source. This is a deliberate reduction to the subset needed by the verified kernel modules, not a bug.
- The `get()` function is confirmed MATCH by the consistency report and is identical in both versions.

## Verification: PASS
