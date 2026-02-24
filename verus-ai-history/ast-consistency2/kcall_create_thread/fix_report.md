# Exec Consistency Fix: kcall_create_thread

## Summary
- Mismatches fixed: 0
- Missing functions added: 1
- Documented equivalences: 5

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `create_thread` [create_thread.diff](create_thread.diff) [create_thread_source.rs](create_thread_source.rs) [create_thread_verus.rs](create_thread_verus.rs) | Added wrapper | Added `create_thread` wrapper function matching the pattern used by `terminate`, `sleep`, `lock_mutex`, and other kcall modules. Delegates to the fully verified `create_thread_model`, discarding ghost witnesses. Includes requires/ensures for stack size, error code validity, and args linkage. |
| `create_thread_model` [create_thread_model_verus.rs](create_thread_model_verus.rs) | Documented (kept) | Verified exec model of the original `create_thread` control flow. This is the core verified function — the `create_thread` wrapper delegates to it. All kcall modules use this `_model` pattern (e.g., `terminate_model`, `sleep_model`). |
| `is_user_region` [is_user_region_verus.rs](is_user_region_verus.rs) | Documented (kept) | Trust boundary T1: `external_body` model of `Vmem::is_user_region(addr, size)`. Required for the validation pipeline model. Ghost parameters track validated addresses. |
| `is_user_addr` [is_user_addr_verus.rs](is_user_addr_verus.rs) | Documented (kept) | Trust boundary T2: `external_body` model of `Vmem::is_user_addr(addr)`. Required for user_fn and user_tda validation steps. Ghost parameter tracks validated address. |
| `copy_from_user` [copy_from_user_verus.rs](copy_from_user_verus.rs) | Documented (kept) | Trust boundary T3: `external_body` model of `pm::copy_from_user(pm, pid, dst, src)`. Models the fallible copy from user space with error code propagation. Returns `CopyOk { args }` on success to create explicit data flow to validation steps 3–5. |
| `assert_user_stack_size` [assert_user_stack_size_verus.rs](assert_user_stack_size_verus.rs) | Documented (kept) | Build-time verification bridge: `external_body` that asserts `USER_STACK_SIZE()` spec constant matches the runtime `config::memory_layout::USER_STACK_SIZE`. Prevents silent drift between spec and implementation. |
| `ThreadCreateArgsModel` [struct_ThreadCreateArgsModel_verus.rs](struct_ThreadCreateArgsModel_verus.rs) | Documented (kept) | Model struct for `ThreadCreateArgs` used in verification. Captures validation results and concrete addresses from the copied structure. Required by the model-based verification architecture. |

## Verification: PASS
- 24 verified, 0 errors
- Command: `./verus-ai/scripts/verify.sh kcall_create_thread`
