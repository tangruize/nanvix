# Exec Diff: running

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/running.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/running.rs`

| Function | Status | Files |
|----------|--------|-------|
| `exit` | MISMATCH | exit_source.rs, exit_verus.rs, exit.diff |
| `exit_thread` | MISMATCH | exit_thread_source.rs, exit_thread_verus.rs, exit_thread.diff |
| `find_thread` | MISMATCH | find_thread_source.rs, find_thread_verus.rs, find_thread.diff |
| `find_thread_mut` | MISMATCH | find_thread_mut_source.rs, find_thread_mut_verus.rs, find_thread_mut.diff |
| `get_tid` | MISMATCH | get_tid_source.rs, get_tid_verus.rs, get_tid.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `running_mut` | MISMATCH | running_mut_source.rs, running_mut_verus.rs, running_mut.diff |
| `schedule` | MISMATCH | schedule_source.rs, schedule_verus.rs, schedule.diff |
| `sleep` | MISMATCH | sleep_source.rs, sleep_verus.rs, sleep.diff |
| `state` | MISMATCH | state_source.rs, state_verus.rs, state.diff |
| `state_mut` | MISMATCH | state_mut_source.rs, state_mut_verus.rs, state_mut.diff |
| `try_join_thread` | MISMATCH | try_join_thread_source.rs, try_join_thread_verus.rs, try_join_thread.diff |
| `wakeup` | MISMATCH | wakeup_source.rs, wakeup_verus.rs, wakeup.diff |
| `interrupted_resume` | EXTRA_IN_VERUS | interrupted_resume_verus.rs (EXTRA) |
| `vec_push_all` | EXTRA_IN_VERUS | vec_push_all_verus.rs (EXTRA) |
| `vec_remove_at` | EXTRA_IN_VERUS | vec_remove_at_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `InterruptedProcess` | EXTRA_IN_VERUS | struct_InterruptedProcess_verus.rs (EXTRA) |
| `RunnableProcess` | EXTRA_IN_VERUS | struct_RunnableProcess_verus.rs (EXTRA) |
| `RunningProcess` | MISMATCH | struct_RunningProcess_source.rs, struct_RunningProcess_verus.rs, struct_RunningProcess.diff |
| `ScheduleResult` | EXTRA_IN_VERUS | struct_ScheduleResult_verus.rs (EXTRA) |
| `SleepingProcess` | EXTRA_IN_VERUS | struct_SleepingProcess_verus.rs (EXTRA) |
| `ZombieProcess` | EXTRA_IN_VERUS | struct_ZombieProcess_verus.rs (EXTRA) |
