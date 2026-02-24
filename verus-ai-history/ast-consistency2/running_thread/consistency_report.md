# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/running.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/running.rs`

## Summary

- Functions matched: 1/10
- Functions mismatched: 8
- Missing in Verus: 1
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `exit` [exit.diff](exit.diff) | [exit_source.rs](exit_source.rs) | [exit_verus.rs](exit_verus.rs) | MISMATCH | 171-174 | 381-395 |
| `from_state` [from_state.diff](from_state.diff) | [from_state_source.rs](from_state_source.rs) | [from_state_verus.rs](from_state_verus.rs) | MISMATCH | 67-69 | 260-274 |
| `id` [id.diff](id.diff) | [id_source.rs](id_source.rs) | [id_verus.rs](id_verus.rs) | MISMATCH | 114-116 | 336-343 |
| `join_cond` [join_cond_source.rs](join_cond_source.rs) | MISSING_IN_VERUS | 153-156 |  |
| `put_mutex_guard` [put_mutex_guard.diff](put_mutex_guard.diff) | [put_mutex_guard_source.rs](put_mutex_guard_source.rs) | [put_mutex_guard_verus.rs](put_mutex_guard_verus.rs) | MISMATCH | 186-188 | 422-438 |
| `schedule` [schedule.diff](schedule.diff) | [schedule_source.rs](schedule_source.rs) | [schedule_verus.rs](schedule_verus.rs) | MISMATCH | 100-103 | 316-329 |
| `sleep` [sleep.diff](sleep.diff) | [sleep_source.rs](sleep_source.rs) | [sleep_verus.rs](sleep_verus.rs) | MISMATCH | 85-88 | 290-304 |
| `take_mutex_guard` [take_mutex_guard.diff](take_mutex_guard.diff) | [take_mutex_guard_source.rs](take_mutex_guard_source.rs) | [take_mutex_guard_verus.rs](take_mutex_guard_verus.rs) | MISMATCH | 203-205 | 457-472 |
| `thread_state` [thread_state.diff](thread_state.diff) | [thread_state_source.rs](thread_state_source.rs) | [thread_state_verus.rs](thread_state_verus.rs) | MISMATCH | 127-129 | 351-359 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `exit` [exit.diff](exit.diff) | [exit_source.rs](exit_source.rs) | [exit_verus.rs](exit_verus.rs) | MISMATCH | ❌ |
| `from_state` [from_state.diff](from_state.diff) | [from_state_source.rs](from_state_source.rs) | [from_state_verus.rs](from_state_verus.rs) | MISMATCH | ❌ |
| `id` [id.diff](id.diff) | [id_source.rs](id_source.rs) | [id_verus.rs](id_verus.rs) | MISMATCH | ❌ |
| `join_cond` [join_cond_source.rs](join_cond_source.rs) | MISSING_IN_VERUS | ❌ |
| `put_mutex_guard` [put_mutex_guard.diff](put_mutex_guard.diff) | [put_mutex_guard_source.rs](put_mutex_guard_source.rs) | [put_mutex_guard_verus.rs](put_mutex_guard_verus.rs) | MISMATCH | ❌ |
| `schedule` [schedule.diff](schedule.diff) | [schedule_source.rs](schedule_source.rs) | [schedule_verus.rs](schedule_verus.rs) | MISMATCH | ❌ |
| `sleep` [sleep.diff](sleep.diff) | [sleep_source.rs](sleep_source.rs) | [sleep_verus.rs](sleep_verus.rs) | MISMATCH | ❌ |
| `take_mutex_guard` [take_mutex_guard.diff](take_mutex_guard.diff) | [take_mutex_guard_source.rs](take_mutex_guard_source.rs) | [take_mutex_guard_verus.rs](take_mutex_guard_verus.rs) | MISMATCH | ❌ |
| `thread_state` [thread_state.diff](thread_state.diff) | [thread_state_source.rs](thread_state_source.rs) | [thread_state_verus.rs](thread_state_verus.rs) | MISMATCH | ❌ |
| `thread_state_mut` | MATCH | ✅ |

## Inconsistent Structs

- `ReadyThread` [struct_ReadyThread_verus.rs](struct_ReadyThread_verus.rs): EXTRA_IN_VERUS
- `RunningThread` [struct_RunningThread.diff](struct_RunningThread.diff) | [struct_RunningThread_source.rs](struct_RunningThread_source.rs) | [struct_RunningThread_verus.rs](struct_RunningThread_verus.rs): MISMATCH
- `SleepingThread` [struct_SleepingThread_verus.rs](struct_SleepingThread_verus.rs): EXTRA_IN_VERUS
- `ZombieThread` [struct_ZombieThread_verus.rs](struct_ZombieThread_verus.rs): EXTRA_IN_VERUS
