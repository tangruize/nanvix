# Exec Diff: sleeping

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/sleeping.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/sleeping.rs`

| Function | Status | Files |
|----------|--------|-------|
| `add_thread` | MISMATCH | add_thread_source.rs, add_thread_verus.rs, add_thread.diff |
| `find_thread` | MISMATCH | find_thread_source.rs, find_thread_verus.rs, find_thread.diff |
| `find_thread_mut` | MISMATCH | find_thread_mut_source.rs, find_thread_mut_verus.rs, find_thread_mut.diff |
| `new` | MISMATCH | new_source.rs, new_verus.rs, new.diff |
| `state` | MISMATCH | state_source.rs, state_verus.rs, state.diff |
| `state_mut` | MISMATCH | state_mut_source.rs, state_mut_verus.rs, state_mut.diff |
| `terminate` | MISMATCH | terminate_source.rs, terminate_verus.rs, terminate.diff |
| `wakeup` | MISMATCH | wakeup_source.rs, wakeup_verus.rs, wakeup.diff |
| `wakeup_alarm` | MISMATCH | wakeup_alarm_source.rs, wakeup_alarm_verus.rs, wakeup_alarm.diff |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `InterruptedProcess` | EXTRA_IN_VERUS | struct_InterruptedProcess_verus.rs (EXTRA) |
| `RunnableProcess` | EXTRA_IN_VERUS | struct_RunnableProcess_verus.rs (EXTRA) |
| `SleepingProcess` | MISMATCH | struct_SleepingProcess_source.rs, struct_SleepingProcess_verus.rs, struct_SleepingProcess.diff |
