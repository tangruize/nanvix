# Exec Diff: lib

**Source:** `src/libs/error/src/lib.rs`
**Verus:** `verus/split/libs/error/lib.rs`

## Full Diffs (source vs Verus with spec/proof)

Directory: `full/`

| Function | Status | Files |
|----------|--------|-------|
| `ErrorCode::fmt` | MISSING_IN_VERUS | ErrorCode__fmt_source.rs (MISSING in verus) |
| `ErrorCode::try_from` | MISSING_IN_VERUS | ErrorCode__try_from_source.rs (MISSING in verus) |
| `i16::from` | MISSING_IN_VERUS | i16__from_source.rs (MISSING in verus) |
| `i32::from` | MISSING_IN_VERUS | i32__from_source.rs (MISSING in verus) |
| `i64::from` | MISSING_IN_VERUS | i64__from_source.rs (MISSING in verus) |
| `invalid_error_code` | MISSING_IN_VERUS | invalid_error_code_source.rs (MISSING in verus) |
| `u16::from` | MISSING_IN_VERUS | u16__from_source.rs (MISSING in verus) |
| `u32::from` | MISSING_IN_VERUS | u32__from_source.rs (MISSING in verus) |
| `Error::fmt` | EXTRA_IN_VERUS | Error__fmt_verus.rs (EXTRA) |
| `Error::log` | EXTRA_IN_VERUS | Error__log_verus.rs (EXTRA) |
| `log_error_with_context` | EXTRA_IN_VERUS | log_error_with_context_verus.rs (EXTRA) |

## Exec-Only Diffs (source vs Verus stripped of ghost/proof)

Directory: `exec-only/`

These diffs show only the executable code differences, with all Verus
annotations (requires/ensures, proof blocks, ghost variables, invariants)
removed. This makes it easier to spot real exec logic changes.

| Function | Status | Files |
|----------|--------|-------|
| `ErrorCode::fmt` | MISSING_IN_VERUS | ErrorCode__fmt_source.rs (MISSING in verus) |
| `ErrorCode::try_from` | MISSING_IN_VERUS | ErrorCode__try_from_source.rs (MISSING in verus) |
| `i16::from` | MISSING_IN_VERUS | i16__from_source.rs (MISSING in verus) |
| `i32::from` | MISSING_IN_VERUS | i32__from_source.rs (MISSING in verus) |
| `i64::from` | MISSING_IN_VERUS | i64__from_source.rs (MISSING in verus) |
| `invalid_error_code` | MISSING_IN_VERUS | invalid_error_code_source.rs (MISSING in verus) |
| `u16::from` | MISSING_IN_VERUS | u16__from_source.rs (MISSING in verus) |
| `u32::from` | MISSING_IN_VERUS | u32__from_source.rs (MISSING in verus) |
| `Error::fmt` | EXTRA_IN_VERUS | Error__fmt_verus.rs (EXTRA) |
| `Error::log` | EXTRA_IN_VERUS | Error__log_verus.rs (EXTRA) |
| `log_error_with_context` | EXTRA_IN_VERUS | log_error_with_context_verus.rs (EXTRA) |
