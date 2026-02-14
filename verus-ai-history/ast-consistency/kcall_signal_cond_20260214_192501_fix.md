# Exec Consistency Fix: kcall_signal_cond

## Summary
- Mismatches fixed: 0
- Missing functions added: 1
- Documented equivalences: 3

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `signal_cond` | Added wrapper to verus exec file | Wrapper delegates to `signal_cond_model`, matching the original function's signature (`pid`, `tid`, `cond_addr`, `broadcast`). `pid` and `tid` are forwarded as ghost since they only affect trace logging in the original. Follows the same pattern as `lock_mutex` and `wait_cond` wrappers. |
| `signal_cond_model` | Kept (documented) | Main verified exec model of the `signal_cond` kernel call. Mirrors the original control flow using model types (external_body trust boundaries). Necessary because the original uses `ProcessManager`, `Condvar`, and `ConditionAddress` types unavailable in Verus context. |
| `drop_cond_model` | Kept (documented) | External_body function modeling trust boundary T3 (`Condvar::drop()`). Models the implicit drop of the Condvar at scope exit in the original code. Required to verify resource cleanup properties. |
| `put_cond_model` | Kept (documented) | External_body function modeling trust boundary T4 (`ProcessManager::put_cond()`). Models the explicit put_cond call in the original code. Required to verify the condition variable slot release. |

## Verification: PASS
- 22 verified, 0 errors
- No assume, admit, or unjustified external_body added
