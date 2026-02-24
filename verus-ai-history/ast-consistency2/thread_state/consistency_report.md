# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/state.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/state.rs`

## Summary

- Functions matched: 0/15
- Functions mismatched: 10
- Missing in Verus: 5
- Extra in Verus: 1
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `context_mut` [context_mut_source.rs](context_mut_source.rs) | MISSING_IN_VERUS | 121-123 |  |
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | 283-291 |  |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | 277-279 |  |
| `fpu_state_mut` [fpu_state_mut_source.rs](fpu_state_mut_source.rs) | MISSING_IN_VERUS | 134-136 |  |
| `get_thread_data_area` [get_thread_data_area.diff](get_thread_data_area.diff) | [get_thread_data_area_source.rs](get_thread_data_area_source.rs) | [get_thread_data_area_verus.rs](get_thread_data_area_verus.rs) | MISMATCH | 271-273 | 507-512 |
| `id` [id.diff](id.diff) | [id_source.rs](id_source.rs) | [id_verus.rs](id_verus.rs) | MISMATCH | 147-149 | 190-195 |
| `join_cond` [join_cond_source.rs](join_cond_source.rs) | MISSING_IN_VERUS | 160-162 |  |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 91-110 | 155-183 |
| `set_interrupt_reason` [set_interrupt_reason.diff](set_interrupt_reason.diff) | [set_interrupt_reason_source.rs](set_interrupt_reason_source.rs) | [set_interrupt_reason_verus.rs](set_interrupt_reason_verus.rs) | MISMATCH | 204-206 | 258-274 |
| `store_mutex_guard` [store_mutex_guard.diff](store_mutex_guard.diff) | [store_mutex_guard_source.rs](store_mutex_guard_source.rs) | [store_mutex_guard_verus.rs](store_mutex_guard_verus.rs) | MISMATCH | 174-176 | 320-359 |
| `store_thread_data_area` [store_thread_data_area.diff](store_thread_data_area.diff) | [store_thread_data_area_source.rs](store_thread_data_area_source.rs) | [store_thread_data_area_verus.rs](store_thread_data_area_verus.rs) | MISMATCH | 256-258 | 485-500 |
| `take_interrupt_reason` [take_interrupt_reason.diff](take_interrupt_reason.diff) | [take_interrupt_reason_source.rs](take_interrupt_reason_source.rs) | [take_interrupt_reason_verus.rs](take_interrupt_reason_verus.rs) | MISMATCH | 217-219 | 281-300 |
| `take_kernel_stack` [take_kernel_stack.diff](take_kernel_stack.diff) | [take_kernel_stack_source.rs](take_kernel_stack_source.rs) | [take_kernel_stack_verus.rs](take_kernel_stack_verus.rs) | MISMATCH | 230-232 | 204-223 |
| `take_mutex_guard` [take_mutex_guard.diff](take_mutex_guard.diff) | [take_mutex_guard_source.rs](take_mutex_guard_source.rs) | [take_mutex_guard_verus.rs](take_mutex_guard_verus.rs) | MISMATCH | 191-193 | 378-458 |
| `take_user_stack` [take_user_stack.diff](take_user_stack.diff) | [take_user_stack_source.rs](take_user_stack_source.rs) | [take_user_stack_verus.rs](take_user_stack_verus.rs) | MISMATCH | 243-245 | 232-251 |
| `check_drop_safe` [check_drop_safe_verus.rs](check_drop_safe_verus.rs) | EXTRA_IN_VERUS |  | 469-478 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `context_mut` [context_mut_source.rs](context_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `drop` [drop_source.rs](drop_source.rs) | MISSING_IN_VERUS | ❌ |
| `fmt` [fmt_source.rs](fmt_source.rs) | MISSING_IN_VERUS | ❌ |
| `fpu_state_mut` [fpu_state_mut_source.rs](fpu_state_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `get_thread_data_area` [get_thread_data_area.diff](get_thread_data_area.diff) | [get_thread_data_area_source.rs](get_thread_data_area_source.rs) | [get_thread_data_area_verus.rs](get_thread_data_area_verus.rs) | MISMATCH | ❌ |
| `id` [id.diff](id.diff) | [id_source.rs](id_source.rs) | [id_verus.rs](id_verus.rs) | MISMATCH | ❌ |
| `join_cond` [join_cond_source.rs](join_cond_source.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `set_interrupt_reason` [set_interrupt_reason.diff](set_interrupt_reason.diff) | [set_interrupt_reason_source.rs](set_interrupt_reason_source.rs) | [set_interrupt_reason_verus.rs](set_interrupt_reason_verus.rs) | MISMATCH | ❌ |
| `store_mutex_guard` [store_mutex_guard.diff](store_mutex_guard.diff) | [store_mutex_guard_source.rs](store_mutex_guard_source.rs) | [store_mutex_guard_verus.rs](store_mutex_guard_verus.rs) | MISMATCH | ❌ |
| `store_thread_data_area` [store_thread_data_area.diff](store_thread_data_area.diff) | [store_thread_data_area_source.rs](store_thread_data_area_source.rs) | [store_thread_data_area_verus.rs](store_thread_data_area_verus.rs) | MISMATCH | ❌ |
| `take_interrupt_reason` [take_interrupt_reason.diff](take_interrupt_reason.diff) | [take_interrupt_reason_source.rs](take_interrupt_reason_source.rs) | [take_interrupt_reason_verus.rs](take_interrupt_reason_verus.rs) | MISMATCH | ❌ |
| `take_kernel_stack` [take_kernel_stack.diff](take_kernel_stack.diff) | [take_kernel_stack_source.rs](take_kernel_stack_source.rs) | [take_kernel_stack_verus.rs](take_kernel_stack_verus.rs) | MISMATCH | ❌ |
| `take_mutex_guard` [take_mutex_guard.diff](take_mutex_guard.diff) | [take_mutex_guard_source.rs](take_mutex_guard_source.rs) | [take_mutex_guard_verus.rs](take_mutex_guard_verus.rs) | MISMATCH | ❌ |
| `take_user_stack` [take_user_stack.diff](take_user_stack.diff) | [take_user_stack_source.rs](take_user_stack_source.rs) | [take_user_stack_verus.rs](take_user_stack_verus.rs) | MISMATCH | ❌ |
| `check_drop_safe` [check_drop_safe_verus.rs](check_drop_safe_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `ThreadState` [struct_ThreadState.diff](struct_ThreadState.diff) | [struct_ThreadState_source.rs](struct_ThreadState_source.rs) | [struct_ThreadState_verus.rs](struct_ThreadState_verus.rs): MISMATCH
