# Exec Diff: runnable

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/runnable.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/runnable.rs`

| Function | Status | Files |
|----------|--------|-------|
| `add_thread` | MISSING_IN_VERUS | add_thread_source.rs (MISSING in verus) |
| `earliest_admission_time` | MISSING_IN_VERUS | earliest_admission_time_source.rs (MISSING in verus) |
| `find_thread` | MISSING_IN_VERUS | find_thread_source.rs (MISSING in verus) |
| `find_thread_mut` | MISSING_IN_VERUS | find_thread_mut_source.rs (MISSING in verus) |
| `from_state` | MISMATCH | from_state_source.rs, from_state_verus.rs, from_state.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `run` | MISMATCH | run_source.rs, run_verus.rs, run.diff |
| `state` | MISSING_IN_VERUS | state_source.rs (MISSING in verus) |
| `state_mut` | MISSING_IN_VERUS | state_mut_source.rs (MISSING in verus) |
| `terminate` | MISMATCH | terminate_source.rs, terminate_verus.rs, terminate.diff |
| `wakeup` | MISSING_IN_VERUS | wakeup_source.rs (MISSING in verus) |
| `clock_now` | EXTRA_IN_VERUS | clock_now_verus.rs (EXTRA) |
| `pid_i32` | EXTRA_IN_VERUS | pid_i32_verus.rs (EXTRA) |
| `vec_concat` | EXTRA_IN_VERUS | vec_concat_verus.rs (EXTRA) |
| `vec_remove_at` | EXTRA_IN_VERUS | vec_remove_at_verus.rs (EXTRA) |
| `vec_search` | EXTRA_IN_VERUS | vec_search_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `InterruptedProcess` | EXTRA_IN_VERUS | struct_InterruptedProcess_verus.rs (EXTRA) |
| `RunnableProcess` | MISMATCH | struct_RunnableProcess_source.rs, struct_RunnableProcess_verus.rs, struct_RunnableProcess.diff |
| `RunningProcess` | EXTRA_IN_VERUS | struct_RunningProcess_verus.rs (EXTRA) |
| `ZombieProcess` | EXTRA_IN_VERUS | struct_ZombieProcess_verus.rs (EXTRA) |
