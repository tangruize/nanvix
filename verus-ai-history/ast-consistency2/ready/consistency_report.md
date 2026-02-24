# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/ready.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/ready.rs`

## Summary

- Functions matched: 1/9
- Functions mismatched: 7
- Missing in Verus: 1
- Extra in Verus: 5
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `admission_time` [admission_time.diff](admission_time.diff) | [admission_time_source.rs](admission_time_source.rs) | [admission_time_verus.rs](admission_time_verus.rs) | MISMATCH | 214-216 | 349-354 |
| `from_state` [from_state.diff](from_state.diff) | [from_state_source.rs](from_state_source.rs) | [from_state_verus.rs](from_state_verus.rs) | MISMATCH | 110-115 | 296-316 |
| `id` [id.diff](id.diff) | [id_source.rs](id_source.rs) | [id_verus.rs](id_verus.rs) | MISMATCH | 126-128 | 323-328 |
| `join_cond` [join_cond_source.rs](join_cond_source.rs) | MISSING_IN_VERUS | 201-203 |  |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 76-95 | 261-285 |
| `run` [run.diff](run.diff) | [run_source.rs](run_source.rs) | [run_verus.rs](run_verus.rs) | MISMATCH | 169-177 | 465-490 |
| `terminate` [terminate.diff](terminate.diff) | [terminate_source.rs](terminate_source.rs) | [terminate_verus.rs](terminate_verus.rs) | MISMATCH | 188-190 | 500-516 |
| `thread_state` [thread_state.diff](thread_state.diff) | [thread_state_source.rs](thread_state_source.rs) | [thread_state_verus.rs](thread_state_verus.rs) | MISMATCH | 139-141 | 336-342 |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | EXTRA_IN_VERUS |  | 75-80 |
| `exit_status_interrupted_value` [exit_status_interrupted_value_verus.rs](exit_status_interrupted_value_verus.rs) | EXTRA_IN_VERUS |  | 88-93 |
| `set_interrupt_reason` [set_interrupt_reason_verus.rs](set_interrupt_reason_verus.rs) | EXTRA_IN_VERUS |  | 365-385 |
| `store_mutex_guard` [store_mutex_guard_verus.rs](store_mutex_guard_verus.rs) | EXTRA_IN_VERUS |  | 396-415 |
| `take_mutex_guard` [take_mutex_guard_verus.rs](take_mutex_guard_verus.rs) | EXTRA_IN_VERUS |  | 426-443 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `admission_time` [admission_time.diff](admission_time.diff) | [admission_time_source.rs](admission_time_source.rs) | [admission_time_verus.rs](admission_time_verus.rs) | MISMATCH | ❌ |
| `from_state` [from_state.diff](from_state.diff) | [from_state_source.rs](from_state_source.rs) | [from_state_verus.rs](from_state_verus.rs) | MISMATCH | ❌ |
| `id` [id.diff](id.diff) | [id_source.rs](id_source.rs) | [id_verus.rs](id_verus.rs) | MISMATCH | ❌ |
| `join_cond` [join_cond_source.rs](join_cond_source.rs) | MISSING_IN_VERUS | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `run` [run.diff](run.diff) | [run_source.rs](run_source.rs) | [run_verus.rs](run_verus.rs) | MISMATCH | ❌ |
| `terminate` [terminate.diff](terminate.diff) | [terminate_source.rs](terminate_source.rs) | [terminate_verus.rs](terminate_verus.rs) | MISMATCH | ❌ |
| `thread_state` [thread_state.diff](thread_state.diff) | [thread_state_source.rs](thread_state_source.rs) | [thread_state_verus.rs](thread_state_verus.rs) | MISMATCH | ❌ |
| `thread_state_mut` | MATCH | ✅ |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `exit_status_interrupted_value` [exit_status_interrupted_value_verus.rs](exit_status_interrupted_value_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `set_interrupt_reason` [set_interrupt_reason_verus.rs](set_interrupt_reason_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `store_mutex_guard` [store_mutex_guard_verus.rs](store_mutex_guard_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `take_mutex_guard` [take_mutex_guard_verus.rs](take_mutex_guard_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `ReadyThread` [struct_ReadyThread.diff](struct_ReadyThread.diff) | [struct_ReadyThread_source.rs](struct_ReadyThread_source.rs) | [struct_ReadyThread_verus.rs](struct_ReadyThread_verus.rs): MISMATCH
- `RunResult` [struct_RunResult_verus.rs](struct_RunResult_verus.rs): EXTRA_IN_VERUS
- `RunningThread` [struct_RunningThread_verus.rs](struct_RunningThread_verus.rs): EXTRA_IN_VERUS
- `ZombieThread` [struct_ZombieThread_verus.rs](struct_ZombieThread_verus.rs): EXTRA_IN_VERUS
