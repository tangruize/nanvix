# Exec Consistency Fix: kcall_terminate

## Summary
- Mismatches fixed: 0
- Missing functions added: 1
- Documented equivalences: 0

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `terminate` [terminate.diff](terminate.diff) [terminate_source.rs](terminate_source.rs) [terminate_verus.rs](terminate_verus.rs) | Added wrapper function | The verified `terminate_model` contained the full exec logic but lacked a `terminate` wrapper matching the original function name. Added `terminate(arg0, Ghost(pm_pre))` that delegates to `terminate_model` and returns only the `KcallResultModel`, following the same pattern used by `sleep`, `lock_mutex`, and other kcall modules. Parameter abstraction (`&mut ProcessManager` → `Ghost<ProcessManagerStateView>`, `&KcallArgs` → `arg0: u32`) is consistent with the existing model approach. |

## Verification: PASS
- 21 verified, 0 errors
- No `assume` or `admit` added
- No new `external_body` added
