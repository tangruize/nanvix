# Exec Diff: state

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/state.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/state.rs`

| Function | Status | Files |
|----------|--------|-------|
| `context_mut` | MISSING_IN_VERUS | context_mut_source.rs (MISSING in verus) |
| `drop` | MISSING_IN_VERUS | drop_source.rs (MISSING in verus) |
| `fmt` | MISSING_IN_VERUS | fmt_source.rs (MISSING in verus) |
| `fpu_state_mut` | MISSING_IN_VERUS | fpu_state_mut_source.rs (MISSING in verus) |
| `get_thread_data_area` | MISMATCH | get_thread_data_area_source.rs, get_thread_data_area_verus.rs, get_thread_data_area.diff |
| `id` | MISMATCH | id_source.rs, id_verus.rs, id.diff |
| `join_cond` | MISSING_IN_VERUS | join_cond_source.rs (MISSING in verus) |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `set_interrupt_reason` | MISMATCH | set_interrupt_reason_source.rs, set_interrupt_reason_verus.rs, set_interrupt_reason.diff |
| `store_mutex_guard` | MISMATCH | store_mutex_guard_source.rs, store_mutex_guard_verus.rs, store_mutex_guard.diff |
| `store_thread_data_area` | MISMATCH | store_thread_data_area_source.rs, store_thread_data_area_verus.rs, store_thread_data_area.diff |
| `take_interrupt_reason` | MISMATCH | take_interrupt_reason_source.rs, take_interrupt_reason_verus.rs, take_interrupt_reason.diff |
| `take_kernel_stack` | MISMATCH | take_kernel_stack_source.rs, take_kernel_stack_verus.rs, take_kernel_stack.diff |
| `take_mutex_guard` | MISMATCH | take_mutex_guard_source.rs, take_mutex_guard_verus.rs, take_mutex_guard.diff |
| `take_user_stack` | MISMATCH | take_user_stack_source.rs, take_user_stack_verus.rs, take_user_stack.diff |
| `check_drop_safe` | EXTRA_IN_VERUS | check_drop_safe_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ThreadState` | MISMATCH | struct_ThreadState_source.rs, struct_ThreadState_verus.rs, struct_ThreadState.diff |
