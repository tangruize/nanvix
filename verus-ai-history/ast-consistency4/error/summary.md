# Exec Diff: lib

**Source:** `src/libs/error/src/lib.rs`
**Verus:** `verus/split/libs/error/lib.rs`

## Full Diffs (source vs Verus with spec/proof)

Directory: `full/`

| Function | Status | Files |
|----------|--------|-------|
| `Error::new` | MISMATCH | Error__new_source.rs, Error__new_verus.rs, Error__new.diff |
| `ErrorCode::try_from` | MISMATCH | ErrorCode__try_from_source.rs, ErrorCode__try_from_verus.rs, ErrorCode__try_from.diff |
| `Error::log` | EXTRA_IN_VERUS | Error__log_verus.rs (EXTRA) |

## Exec-Only Diffs (source vs Verus stripped of ghost/proof)

Directory: `exec-only/`

These diffs show only the executable code differences, with all Verus
annotations (requires/ensures, proof blocks, ghost variables, invariants)
removed. This makes it easier to spot real exec logic changes.

| Function | Status | Files |
|----------|--------|-------|
| `Error::new` | MISMATCH | Error__new_source.rs, Error__new_verus_stripped.rs, Error__new.diff |
| `ErrorCode::try_from` | MISMATCH | ErrorCode__try_from_source.rs, ErrorCode__try_from_verus_stripped.rs, ErrorCode__try_from.diff |
| `Error::log` | EXTRA_IN_VERUS | Error__log_verus.rs (EXTRA) |
