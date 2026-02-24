# Exec Diff: ready

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/ready.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/ready.rs`

| Function | Status | Files |
|----------|--------|-------|
| `admission_time` | MISMATCH | admission_time_source.rs, admission_time_verus.rs, admission_time.diff |
| `from_state` | MISMATCH | from_state_source.rs, from_state_verus.rs, from_state.diff |
| `id` | MISMATCH | id_source.rs, id_verus.rs, id.diff |
| `join_cond` | MISSING_IN_VERUS | join_cond_source.rs (MISSING in verus) |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `run` | MISMATCH | run_source.rs, run_verus.rs, run.diff |
| `terminate` | MISMATCH | terminate_source.rs, terminate_verus.rs, terminate.diff |
| `thread_state` | MISMATCH | thread_state_source.rs, thread_state_verus.rs, thread_state.diff |
| `clock_now` | EXTRA_IN_VERUS | clock_now_verus.rs (EXTRA) |
| `exit_status_interrupted_value` | EXTRA_IN_VERUS | exit_status_interrupted_value_verus.rs (EXTRA) |
| `set_interrupt_reason` | EXTRA_IN_VERUS | set_interrupt_reason_verus.rs (EXTRA) |
| `store_mutex_guard` | EXTRA_IN_VERUS | store_mutex_guard_verus.rs (EXTRA) |
| `take_mutex_guard` | EXTRA_IN_VERUS | take_mutex_guard_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ReadyThread` | MISMATCH | struct_ReadyThread_source.rs, struct_ReadyThread_verus.rs, struct_ReadyThread.diff |
| `RunResult` | EXTRA_IN_VERUS | struct_RunResult_verus.rs (EXTRA) |
| `RunningThread` | EXTRA_IN_VERUS | struct_RunningThread_verus.rs (EXTRA) |
| `ZombieThread` | EXTRA_IN_VERUS | struct_ZombieThread_verus.rs (EXTRA) |
