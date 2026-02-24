# Exec Consistency Fix: kcall_unlock_mutex

## Summary
- Mismatches fixed: 0
- Missing functions added: 1
- Documented equivalences: 3

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `unlock_mutex` [unlock_mutex.diff](unlock_mutex.diff) [unlock_mutex_source.rs](unlock_mutex_source.rs) [unlock_mutex_verus.rs](unlock_mutex_verus.rs) | Added wrapper | Added `unlock_mutex(pid, tid, mutex_addr)` wrapper that calls `unlock_mutex_model` and returns the exec result, matching the original function name and parameter signature. Follows the same pattern as `lock_mutex` and `sleep` wrappers in sibling modules. |
| `take_mutex_guard_model` [take_mutex_guard_model_verus.rs](take_mutex_guard_model_verus.rs) | Documented (kept) | `external_body` trust boundary modeling `ProcessManager::take_mutex_guard()`. Required to decompose the original implicit call into a verifiable step with explicit pre/postconditions. Not present in source because it models a PM-internal call. |
| `drop_guard_model` [drop_guard_model_verus.rs](drop_guard_model_verus.rs) | Documented (kept) | `external_body` trust boundary modeling `MutexGuard::drop()`. Required to model the implicit Rust drop semantics as an explicit verification step with guard-token consumption. Not present in source because Rust drops are implicit. |
| `unlock_mutex_model` [unlock_mutex_model_verus.rs](unlock_mutex_model_verus.rs) | Documented (kept) | Internal verification model with full ghost outputs (PM outcome, guard-drop flag). Called by the `unlock_mutex` wrapper. Exposes detailed postconditions needed by proof lemmas while the wrapper exposes only caller-relevant guarantees. |

## Verification: PASS
- 13 verified, 0 errors
- No `assume` or unjustified `external_body` added
- 2 justified `external_body` functions (trust boundaries T1, T2) pre-existing
