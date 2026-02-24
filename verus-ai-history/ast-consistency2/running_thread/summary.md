# Exec Diff: running

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/running.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/running.rs`

| Function | Status | Files |
|----------|--------|-------|
| `exit` | MISMATCH | exit_source.rs, exit_verus.rs, exit.diff |
| `from_state` | MISMATCH | from_state_source.rs, from_state_verus.rs, from_state.diff |
| `id` | MISMATCH | id_source.rs, id_verus.rs, id.diff |
| `join_cond` | MISSING_IN_VERUS | join_cond_source.rs (MISSING in verus) |
| `put_mutex_guard` | MISMATCH | put_mutex_guard_source.rs, put_mutex_guard_verus.rs, put_mutex_guard.diff |
| `schedule` | MISMATCH | schedule_source.rs, schedule_verus.rs, schedule.diff |
| `sleep` | MISMATCH | sleep_source.rs, sleep_verus.rs, sleep.diff |
| `take_mutex_guard` | MISMATCH | take_mutex_guard_source.rs, take_mutex_guard_verus.rs, take_mutex_guard.diff |
| `thread_state` | MISMATCH | thread_state_source.rs, thread_state_verus.rs, thread_state.diff |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ReadyThread` | EXTRA_IN_VERUS | struct_ReadyThread_verus.rs (EXTRA) |
| `RunningThread` | MISMATCH | struct_RunningThread_source.rs, struct_RunningThread_verus.rs, struct_RunningThread.diff |
| `SleepingThread` | EXTRA_IN_VERUS | struct_SleepingThread_verus.rs (EXTRA) |
| `ZombieThread` | EXTRA_IN_VERUS | struct_ZombieThread_verus.rs (EXTRA) |
