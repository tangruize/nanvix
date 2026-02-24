# Exec Consistency Fix: kcall_join_thread

## Summary
- Mismatches fixed: 0
- Missing functions added: 1
- Documented equivalences: 0

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `join_thread` [join_thread.diff](join_thread.diff) [join_thread_source.rs](join_thread_source.rs) [join_thread_verus.rs](join_thread_verus.rs) | Added wrapper | The verus file had `join_thread_model` (the fully verified exec model with ghost witnesses) but was missing the `join_thread` wrapper that matches the original function name. Following the established pattern in other kcall files (e.g., `terminate`, `create_thread`, `lock_mutex`, `sleep`), added a `join_thread` wrapper that delegates to `join_thread_model` and discards ghost witnesses, returning only the `JoinThreadKcallResultModel`. The wrapper propagates the same safety preconditions (`spec_is_user_process`, `spec_pm_initialized`, `spec_mm_initialized`, `spec_no_resources_held`) and key postconditions (exhaustiveness, mutual exclusion, success implies `ExitStatus::ok()`). |

## Verification: PASS
- 22 verified, 0 errors
- No `assume` or `admit` added
- No unjustified `external_body` added
