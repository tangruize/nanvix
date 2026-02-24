# Exec Diff: sleeping

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/sleeping.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/sleeping.rs`

| Function | Status | Files |
|----------|--------|-------|
| `alarm` | MISMATCH | alarm_source.rs, alarm_verus.rs, alarm.diff |
| `from_state` | MISMATCH | from_state_source.rs, from_state_verus.rs, from_state.diff |
| `get_thread_data_area` | MISMATCH | get_thread_data_area_source.rs, get_thread_data_area_verus.rs, get_thread_data_area.diff |
| `id` | MISMATCH | id_source.rs, id_verus.rs, id.diff |
| `interrupt` | MISMATCH | interrupt_source.rs, interrupt_verus.rs, interrupt.diff |
| `join_cond` | MISMATCH | join_cond_source.rs, join_cond_verus.rs, join_cond.diff |
| `set_thread_data_area` | MISMATCH | set_thread_data_area_source.rs, set_thread_data_area_verus.rs, set_thread_data_area.diff |
| `thread_state` | MISMATCH | thread_state_source.rs, thread_state_verus.rs, thread_state.diff |
| `wakeup` | MISMATCH | wakeup_source.rs, wakeup_verus.rs, wakeup.diff |
| `clock_now` | EXTRA_IN_VERUS | clock_now_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `Condvar` | EXTRA_IN_VERUS | struct_Condvar_verus.rs (EXTRA) |
| `InterruptedThread` | EXTRA_IN_VERUS | struct_InterruptedThread_verus.rs (EXTRA) |
| `ReadyThread` | EXTRA_IN_VERUS | struct_ReadyThread_verus.rs (EXTRA) |
| `SleepingThread` | MISMATCH | struct_SleepingThread_source.rs, struct_SleepingThread_verus.rs, struct_SleepingThread.diff |
