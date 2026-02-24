# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/state/sleeping.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/state/sleeping.rs`

## Summary

- Functions matched: 0/9
- Functions mismatched: 9
- Missing in Verus: 0
- Extra in Verus: 0
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `add_thread` [add_thread.diff](add_thread.diff) | [add_thread_source.rs](add_thread_source.rs) | [add_thread_verus.rs](add_thread_verus.rs) | MISMATCH | 178-187 | 561-602 |
| `find_thread` [find_thread.diff](find_thread.diff) | [find_thread_source.rs](find_thread_source.rs) | [find_thread_verus.rs](find_thread_verus.rs) | MISMATCH | 203-219 | 624-633 |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) | [find_thread_mut_source.rs](find_thread_mut_source.rs) | [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | MISMATCH | 235-251 | 650-666 |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | 51-61 | 140-188 |
| `state` [state.diff](state.diff) | [state_source.rs](state_source.rs) | [state_verus.rs](state_verus.rs) | MISMATCH | 63-65 | 201-206 |
| `state_mut` [state_mut.diff](state_mut.diff) | [state_mut_source.rs](state_mut_source.rs) | [state_mut_verus.rs](state_mut_verus.rs) | MISMATCH | 67-69 | 220-228 |
| `terminate` [terminate.diff](terminate.diff) | [terminate_source.rs](terminate_source.rs) | [terminate_verus.rs](terminate_verus.rs) | MISMATCH | 71-83 | 239-265 |
| `wakeup` [wakeup.diff](wakeup.diff) | [wakeup_source.rs](wakeup_source.rs) | [wakeup_verus.rs](wakeup_verus.rs) | MISMATCH | 85-105 | 287-431 |
| `wakeup_alarm` [wakeup_alarm.diff](wakeup_alarm.diff) | [wakeup_alarm_source.rs](wakeup_alarm_source.rs) | [wakeup_alarm_verus.rs](wakeup_alarm_verus.rs) | MISMATCH | 107-176 | 457-546 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `add_thread` [add_thread.diff](add_thread.diff) | [add_thread_source.rs](add_thread_source.rs) | [add_thread_verus.rs](add_thread_verus.rs) | MISMATCH | ❌ |
| `find_thread` [find_thread.diff](find_thread.diff) | [find_thread_source.rs](find_thread_source.rs) | [find_thread_verus.rs](find_thread_verus.rs) | MISMATCH | ❌ |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) | [find_thread_mut_source.rs](find_thread_mut_source.rs) | [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | MISMATCH | ❌ |
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | MISMATCH | ❌ |
| `state` [state.diff](state.diff) | [state_source.rs](state_source.rs) | [state_verus.rs](state_verus.rs) | MISMATCH | ❌ |
| `state_mut` [state_mut.diff](state_mut.diff) | [state_mut_source.rs](state_mut_source.rs) | [state_mut_verus.rs](state_mut_verus.rs) | MISMATCH | ❌ |
| `terminate` [terminate.diff](terminate.diff) | [terminate_source.rs](terminate_source.rs) | [terminate_verus.rs](terminate_verus.rs) | MISMATCH | ❌ |
| `wakeup` [wakeup.diff](wakeup.diff) | [wakeup_source.rs](wakeup_source.rs) | [wakeup_verus.rs](wakeup_verus.rs) | MISMATCH | ❌ |
| `wakeup_alarm` [wakeup_alarm.diff](wakeup_alarm.diff) | [wakeup_alarm_source.rs](wakeup_alarm_source.rs) | [wakeup_alarm_verus.rs](wakeup_alarm_verus.rs) | MISMATCH | ❌ |

## Inconsistent Structs

- `InterruptedProcess` [struct_InterruptedProcess_verus.rs](struct_InterruptedProcess_verus.rs): EXTRA_IN_VERUS
- `RunnableProcess` [struct_RunnableProcess_verus.rs](struct_RunnableProcess_verus.rs): EXTRA_IN_VERUS
- `SleepingProcess` [struct_SleepingProcess.diff](struct_SleepingProcess.diff) | [struct_SleepingProcess_source.rs](struct_SleepingProcess_source.rs) | [struct_SleepingProcess_verus.rs](struct_SleepingProcess_verus.rs): MISMATCH
