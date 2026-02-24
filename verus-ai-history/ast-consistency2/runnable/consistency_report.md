# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/runnable.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/runnable.rs`

## Summary

- Functions matched: 0/11
- Functions mismatched: 4
- Missing in Verus: 7
- Extra in Verus: 5
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `add_thread` [add_thread_source.rs](add_thread_source.rs) | MISSING_IN_VERUS | 210-214 |  |
| `earliest_admission_time` [earliest_admission_time_source.rs](earliest_admission_time_source.rs) | MISSING_IN_VERUS | 216-222 |  |
| `find_thread` [find_thread_source.rs](find_thread_source.rs) | MISSING_IN_VERUS | 238-266 |  |
| `find_thread_mut` [find_thread_mut_source.rs](find_thread_mut_source.rs) | MISSING_IN_VERUS | 282-320 |  |
| `from_state` [from_state.diff](from_state.diff) | [from_state_source.rs](from_state_source.rs) | [from_state_verus.rs](from_state_verus.rs) | MISMATCH | 72-86 | 354-390 |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 62-70 | 307-333 |
| `run` [run.diff](run.diff) | [run_source.rs](run_source.rs) | [run_verus.rs](run_verus.rs) | MISMATCH | 109-150 | 415-521 |
| `state` [state_source.rs](state_source.rs) | MISSING_IN_VERUS | 88-90 |  |
| `state_mut` [state_mut_source.rs](state_mut_source.rs) | MISSING_IN_VERUS | 92-94 |  |
| `terminate` [terminate.diff](terminate.diff) | [terminate_source.rs](terminate_source.rs) | [terminate_verus.rs](terminate_verus.rs) | MISMATCH | 152-184 | 540-626 |
| `wakeup` [wakeup_source.rs](wakeup_source.rs) | MISSING_IN_VERUS | 186-208 |  |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | EXTRA_IN_VERUS |  | 90-95 |
| `pid_i32` [pid_i32_verus.rs](pid_i32_verus.rs) | EXTRA_IN_VERUS |  | 393-398 |
| `vec_concat` [vec_concat_verus.rs](vec_concat_verus.rs) | EXTRA_IN_VERUS |  | 225-265 |
| `vec_remove_at` [vec_remove_at_verus.rs](vec_remove_at_verus.rs) | EXTRA_IN_VERUS |  | 195-222 |
| `vec_search` [vec_search_verus.rs](vec_search_verus.rs) | EXTRA_IN_VERUS |  | 268-286 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `add_thread` [add_thread_source.rs](add_thread_source.rs) | MISSING_IN_VERUS | ❌ |
| `earliest_admission_time` [earliest_admission_time_source.rs](earliest_admission_time_source.rs) | MISSING_IN_VERUS | ❌ |
| `find_thread` [find_thread_source.rs](find_thread_source.rs) | MISSING_IN_VERUS | ❌ |
| `find_thread_mut` [find_thread_mut_source.rs](find_thread_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `from_state` [from_state.diff](from_state.diff) | [from_state_source.rs](from_state_source.rs) | [from_state_verus.rs](from_state_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `run` [run.diff](run.diff) | [run_source.rs](run_source.rs) | [run_verus.rs](run_verus.rs) | MISMATCH | ❌ |
| `state` [state_source.rs](state_source.rs) | MISSING_IN_VERUS | ❌ |
| `state_mut` [state_mut_source.rs](state_mut_source.rs) | MISSING_IN_VERUS | ❌ |
| `terminate` [terminate.diff](terminate.diff) | [terminate_source.rs](terminate_source.rs) | [terminate_verus.rs](terminate_verus.rs) | MISMATCH | ❌ |
| `wakeup` [wakeup_source.rs](wakeup_source.rs) | MISSING_IN_VERUS | ❌ |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `pid_i32` [pid_i32_verus.rs](pid_i32_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `vec_concat` [vec_concat_verus.rs](vec_concat_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `vec_remove_at` [vec_remove_at_verus.rs](vec_remove_at_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `vec_search` [vec_search_verus.rs](vec_search_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `InterruptedProcess` [struct_InterruptedProcess_verus.rs](struct_InterruptedProcess_verus.rs): EXTRA_IN_VERUS
- `RunnableProcess` [struct_RunnableProcess.diff](struct_RunnableProcess.diff) | [struct_RunnableProcess_source.rs](struct_RunnableProcess_source.rs) | [struct_RunnableProcess_verus.rs](struct_RunnableProcess_verus.rs): MISMATCH
- `RunningProcess` [struct_RunningProcess_verus.rs](struct_RunningProcess_verus.rs): EXTRA_IN_VERUS
- `ZombieProcess` [struct_ZombieProcess_verus.rs](struct_ZombieProcess_verus.rs): EXTRA_IN_VERUS
