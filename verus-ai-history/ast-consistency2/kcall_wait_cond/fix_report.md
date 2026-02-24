# Exec Consistency Fix: kcall_wait_cond

## Summary
- Mismatches fixed: 0
- Missing functions added: 1
- Documented equivalences: 5

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `wait_cond` [wait_cond.diff](wait_cond.diff) [wait_cond_source.rs](wait_cond_source.rs) [wait_cond_verus.rs](wait_cond_verus.rs) | ADDED | Added thin wrapper matching the original `pub unsafe fn wait_cond(...)` signature. Delegates to `wait_cond_model` and extracts the exec result, following the established pattern used by `lock_mutex`, `unlock_mutex`, and `sleep`. The `pid` and `tid` parameters are forwarded as ghost values because they affect only trace logging and PM-internal ownership in the original, not the pipeline control flow. |
| `cond_wait_model` [cond_wait_model_verus.rs](cond_wait_model_verus.rs) | KEPT (documented) | Trust boundary (external_body) modeling `cond.wait(alarm)`. Required for verification — the real `Condvar::wait` cannot be verified directly. |
| `get_cond_and_wait_model` [get_cond_and_wait_model_verus.rs](get_cond_and_wait_model_verus.rs) | KEPT (documented) | Verified helper combining `get_cond` + `cond.wait` into a "stored result" computation. Extracted to encapsulate the block at original lines 116–129 where `get_cond` failure stores the error and `cond.wait` is never called. This decomposition enables precise spec linkage for the stored-result semantics. |
| `mutex_lock_model` [mutex_lock_model_verus.rs](mutex_lock_model_verus.rs) | KEPT (documented) | Trust boundary (external_body) modeling `Mutex::lock(None)` for mutex reacquisition. Postcondition proves `TimedOut` is impossible with `None` timeout. |
| `parse_timeout_model` [parse_timeout_model_verus.rs](parse_timeout_model_verus.rs) | KEPT (documented) | Verified model of timeout parsing logic (original lines 94–109). Extracted as a standalone function to enable independent verification of the three-way timeout parsing (infinite/finite/invalid). |
| `wait_cond_model` [wait_cond_model_verus.rs](wait_cond_model_verus.rs) | KEPT (documented) | Core verified exec model of the `wait_cond` pipeline. Contains the full control flow mirroring the original function with exec-spec equivalence postcondition (`spec_wait_cond_result`). The new `wait_cond` wrapper delegates to this function. |

## Verification: PASS
- 31 verified, 0 errors
- Duration: 10s
- No `assume` or unjustified `external_body` added
