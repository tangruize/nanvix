# Exec Diff: lib

**Source:** `src/libs/error/src/lib.rs`
**Verus:** `verus/split/libs/error/lib.rs`

| Function | Status | Files |
|----------|--------|-------|
| `fmt` | MISMATCH | fmt_source.rs, fmt_verus.rs, fmt.diff |
| `from` | MISSING_IN_VERUS | from_source.rs (MISSING in verus) |
| `invalid_error_code` | MISSING_IN_VERUS | invalid_error_code_source.rs (MISSING in verus) |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `try_from` | MISSING_IN_VERUS | try_from_source.rs (MISSING in verus) |
| `log` | EXTRA_IN_VERUS | log_verus.rs (EXTRA) |
| `log_error_with_context` | EXTRA_IN_VERUS | log_error_with_context_verus.rs (EXTRA) |
