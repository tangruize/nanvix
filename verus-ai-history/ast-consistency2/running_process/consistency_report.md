# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/running.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/running.rs`

## Summary

- Functions matched: 0/13
- Functions mismatched: 13
- Missing in Verus: 0
- Extra in Verus: 3
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `exit` [exit.diff](exit.diff) [exit_source.rs](exit_source.rs) [exit_verus.rs](exit_verus.rs) | MISMATCH | 183-228 | 902-996 |
| `exit_thread` [exit_thread.diff](exit_thread.diff) [exit_thread_source.rs](exit_thread_source.rs) [exit_thread_verus.rs](exit_thread_verus.rs) | MISMATCH | 248-299 | 1021-1135 |
| `find_thread` [find_thread.diff](find_thread.diff) [find_thread_source.rs](find_thread_source.rs) [find_thread_verus.rs](find_thread_verus.rs) | MISMATCH | 411-446 | 679-684 |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) [find_thread_mut_source.rs](find_thread_mut_source.rs) [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | MISMATCH | 462-503 | 701-711 |
| `get_tid` [get_tid.diff](get_tid.diff) [get_tid_source.rs](get_tid_source.rs) [get_tid_verus.rs](get_tid_verus.rs) | MISMATCH | 301-303 | 445-450 |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | 73-89 | 397-436 |
| `running_mut` [running_mut.diff](running_mut.diff) [running_mut_source.rs](running_mut_source.rs) [running_mut_verus.rs](running_mut_verus.rs) | MISMATCH | 108-110 | 504-512 |
| `schedule` [schedule.diff](schedule.diff) [schedule_source.rs](schedule_source.rs) [schedule_verus.rs](schedule_verus.rs) | MISMATCH | 112-134 | 722-765 |
| `sleep` [sleep.diff](sleep.diff) [sleep_source.rs](sleep_source.rs) [sleep_verus.rs](sleep_verus.rs) | MISMATCH | 136-181 | 783-881 |
| `state` [state.diff](state.diff) [state_source.rs](state_source.rs) [state_verus.rs](state_verus.rs) | MISMATCH | 91-93 | 463-468 |
| `state_mut` [state_mut.diff](state_mut.diff) [state_mut_source.rs](state_mut_source.rs) [state_mut_verus.rs](state_mut_verus.rs) | MISMATCH | 95-97 | 482-490 |
| `try_join_thread` [try_join_thread.diff](try_join_thread.diff) [try_join_thread_source.rs](try_join_thread_source.rs) [try_join_thread_verus.rs](try_join_thread_verus.rs) | MISMATCH | 339-395 | 543-657 |
| `wakeup` [wakeup.diff](wakeup.diff) [wakeup_source.rs](wakeup_source.rs) [wakeup_verus.rs](wakeup_verus.rs) | MISMATCH | 305-336 | 1160-1290 |
| `interrupted_resume` [interrupted_resume_verus.rs](interrupted_resume_verus.rs) | EXTRA_IN_VERUS |  | 348-370 |
| `vec_push_all` [vec_push_all_verus.rs](vec_push_all_verus.rs) | EXTRA_IN_VERUS |  | 123-146 |
| `vec_remove_at` [vec_remove_at_verus.rs](vec_remove_at_verus.rs) | EXTRA_IN_VERUS |  | 149-203 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `exit` [exit.diff](exit.diff) [exit_source.rs](exit_source.rs) [exit_verus.rs](exit_verus.rs) | MISMATCH | ❌ |
| `exit_thread` [exit_thread.diff](exit_thread.diff) [exit_thread_source.rs](exit_thread_source.rs) [exit_thread_verus.rs](exit_thread_verus.rs) | MISMATCH | ❌ |
| `find_thread` [find_thread.diff](find_thread.diff) [find_thread_source.rs](find_thread_source.rs) [find_thread_verus.rs](find_thread_verus.rs) | MISMATCH | ❌ |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) [find_thread_mut_source.rs](find_thread_mut_source.rs) [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | MISMATCH | ❌ |
| `get_tid` [get_tid.diff](get_tid.diff) [get_tid_source.rs](get_tid_source.rs) [get_tid_verus.rs](get_tid_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `running_mut` [running_mut.diff](running_mut.diff) [running_mut_source.rs](running_mut_source.rs) [running_mut_verus.rs](running_mut_verus.rs) | MISMATCH | ❌ |
| `schedule` [schedule.diff](schedule.diff) [schedule_source.rs](schedule_source.rs) [schedule_verus.rs](schedule_verus.rs) | MISMATCH | ❌ |
| `sleep` [sleep.diff](sleep.diff) [sleep_source.rs](sleep_source.rs) [sleep_verus.rs](sleep_verus.rs) | MISMATCH | ❌ |
| `state` [state.diff](state.diff) [state_source.rs](state_source.rs) [state_verus.rs](state_verus.rs) | MISMATCH | ❌ |
| `state_mut` [state_mut.diff](state_mut.diff) [state_mut_source.rs](state_mut_source.rs) [state_mut_verus.rs](state_mut_verus.rs) | MISMATCH | ❌ |
| `try_join_thread` [try_join_thread.diff](try_join_thread.diff) [try_join_thread_source.rs](try_join_thread_source.rs) [try_join_thread_verus.rs](try_join_thread_verus.rs) | MISMATCH | ❌ |
| `wakeup` [wakeup.diff](wakeup.diff) [wakeup_source.rs](wakeup_source.rs) [wakeup_verus.rs](wakeup_verus.rs) | MISMATCH | ❌ |
| `interrupted_resume` [interrupted_resume_verus.rs](interrupted_resume_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `vec_push_all` [vec_push_all_verus.rs](vec_push_all_verus.rs) | EXTRA_IN_VERUS | ❌ |
| `vec_remove_at` [vec_remove_at_verus.rs](vec_remove_at_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `InterruptedProcess` [struct_InterruptedProcess_verus.rs](struct_InterruptedProcess_verus.rs): EXTRA_IN_VERUS
- `RunnableProcess` [struct_RunnableProcess_verus.rs](struct_RunnableProcess_verus.rs): EXTRA_IN_VERUS
- `RunningProcess` [struct_RunningProcess.diff](struct_RunningProcess.diff) [struct_RunningProcess_source.rs](struct_RunningProcess_source.rs) [struct_RunningProcess_verus.rs](struct_RunningProcess_verus.rs): MISMATCH
- `ScheduleResult` [struct_ScheduleResult_verus.rs](struct_ScheduleResult_verus.rs): EXTRA_IN_VERUS
- `SleepingProcess` [struct_SleepingProcess_verus.rs](struct_SleepingProcess_verus.rs): EXTRA_IN_VERUS
- `ZombieProcess` [struct_ZombieProcess_verus.rs](struct_ZombieProcess_verus.rs): EXTRA_IN_VERUS
