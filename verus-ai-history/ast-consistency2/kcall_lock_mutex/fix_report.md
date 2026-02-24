# Exec Consistency Fix: kcall_lock_mutex

## Summary
- Mismatches fixed: 0
- Missing functions added: 1
- Documented equivalences: 4

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `lock_mutex` [lock_mutex.diff](lock_mutex.diff) [lock_mutex_source.rs](lock_mutex_source.rs) [lock_mutex_verus.rs](lock_mutex_verus.rs) | Added to Verus exec file | Original function was missing. Added with matching signature (pid, tid, mutex_addr, timeout_s, timeout_ns) using model types. Delegates to `lock_mutex_model` which performs the verified pipeline logic. pid/tid are included for signature fidelity but unused (proven by `lemma_result_independent_of_pid_tid`). |
| `lock_mutex_model` [lock_mutex_model_verus.rs](lock_mutex_model_verus.rs) | Kept (documented) | Verification model of the original `lock_mutex` pipeline. Extracted as a separate function to isolate the core verified logic from the pid/tid parameters that only appear in trace logging. Mirrors the original control flow exactly: parse_timeout → get_mutex → lock → put_mutex_guard. This is the standard pattern used across all kcall verifications (cf. `sleep_model`). |
| `parse_timeout` [parse_timeout_verus.rs](parse_timeout_verus.rs) | Kept (documented) | Helper function extracted from the timeout parsing logic in `lock_mutex` for verification. Implements the `if timeout_s == usize::MAX && timeout_ns == usize::MAX { None } else { SystemTime::new(...) }` branch exactly as in the original, with postconditions linking to `spec_parse_timeout`. Necessary because Verus requires modular verification of sub-computations. |
| `system_time_new` [system_time_new_verus.rs](system_time_new_verus.rs) | Kept (documented) | Verified model of `SystemTime::new(seconds, nanoseconds)`. Returns `Some` iff `nanoseconds < 1_000_000_000`. This is a Trust Boundary (T1) that is fully verified (not external_body) because the logic is simple and well-specified. The `SystemTimeModel` struct it returns models the original `SystemTime` type. |
| `put_mutex_guard_model` [put_mutex_guard_model_verus.rs](put_mutex_guard_model_verus.rs) | Kept (documented) | External body modeling `ProcessManager::put_mutex_guard(mutex_addr, guard)`. Trust Boundary T4 — PM correctness is verified separately. Required to model the third pipeline step. Uses ghost guard token to formalize ownership chain. |
| `SystemTimeModel` [struct_SystemTimeModel_verus.rs](struct_SystemTimeModel_verus.rs) | Kept (documented) | Model struct for `SystemTime` used by `system_time_new`. Contains `seconds: u64` and `nanoseconds: u32` fields matching the original type's semantics. |

## Verification: PASS
- **Before fix**: 24 verified, 0 errors
- **After fix**: 25 verified, 0 errors
- **No assume, admit, or unjustified external_body added**
- External bodies (3): `get_mutex_model`, `mutex_lock_model`, `put_mutex_guard_model` — all pre-existing trust boundaries for PM/mutex module interactions, documented with postconditions constraining their behavior.
