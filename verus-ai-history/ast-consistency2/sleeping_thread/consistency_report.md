# Exec Consistency Report

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/thread/sleeping.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/thread/sleeping.rs`

## Summary

- Functions matched: 1/10
- Functions mismatched: 8
- Missing in Verus: 1
- Extra in Verus: 1
- **Consistent: NO**

## Inconsistent Functions

| Function | Status | Source Lines | Verus Lines |
|----------|--------|-------------|-------------|
| `alarm` [alarm.diff](alarm.diff) [alarm_source.rs](alarm_source.rs) [alarm_verus.rs](alarm_verus.rs) | MISMATCH | 156-158 | 358-365 |
| `from_state` [from_state.diff](from_state.diff) [from_state_source.rs](from_state_source.rs) [from_state_verus.rs](from_state_verus.rs) | MISMATCH | 61-63 | 259-276 |
| `get_thread_data_area` [get_thread_data_area.diff](get_thread_data_area.diff) [get_thread_data_area_source.rs](get_thread_data_area_source.rs) [get_thread_data_area_verus.rs](get_thread_data_area_verus.rs) | MISMATCH | 183-185 | 406-413 |
| `id` [id.diff](id.diff) [id_source.rs](id_source.rs) [id_verus.rs](id_verus.rs) | MISMATCH | 104-106 | 328-335 |
| `interrupt` [interrupt.diff](interrupt.diff) [interrupt_source.rs](interrupt_source.rs) [interrupt_verus.rs](interrupt_verus.rs) | MISMATCH | 91-93 | 308-321 |
| `join_cond` [join_cond.diff](join_cond.diff) [join_cond_source.rs](join_cond_source.rs) [join_cond_verus.rs](join_cond_verus.rs) | MISSING_IN_VERUS | 143-145 |  |
| `set_thread_data_area` [set_thread_data_area.diff](set_thread_data_area.diff) [set_thread_data_area_source.rs](set_thread_data_area_source.rs) [set_thread_data_area_verus.rs](set_thread_data_area_verus.rs) | MISMATCH | 169-171 | 372-399 |
| `thread_state` [thread_state.diff](thread_state.diff) [thread_state_source.rs](thread_state_source.rs) [thread_state_verus.rs](thread_state_verus.rs) | MISMATCH | 117-119 | 343-351 |
| `wakeup` [wakeup.diff](wakeup.diff) [wakeup_source.rs](wakeup_source.rs) [wakeup_verus.rs](wakeup_verus.rs) | MISMATCH | 74-76 | 284-296 |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | EXTRA_IN_VERUS |  | 89-94 |

## All Functions

| Function | Status | Hash Match |
|----------|--------|------------|
| `alarm` [alarm.diff](alarm.diff) [alarm_source.rs](alarm_source.rs) [alarm_verus.rs](alarm_verus.rs) | MISMATCH | ❌ |
| `from_state` [from_state.diff](from_state.diff) [from_state_source.rs](from_state_source.rs) [from_state_verus.rs](from_state_verus.rs) | MISMATCH | ❌ |
| `get_thread_data_area` [get_thread_data_area.diff](get_thread_data_area.diff) [get_thread_data_area_source.rs](get_thread_data_area_source.rs) [get_thread_data_area_verus.rs](get_thread_data_area_verus.rs) | MISMATCH | ❌ |
| `id` [id.diff](id.diff) [id_source.rs](id_source.rs) [id_verus.rs](id_verus.rs) | MISMATCH | ❌ |
| `interrupt` [interrupt.diff](interrupt.diff) [interrupt_source.rs](interrupt_source.rs) [interrupt_verus.rs](interrupt_verus.rs) | MISMATCH | ❌ |
| `join_cond` [join_cond.diff](join_cond.diff) [join_cond_source.rs](join_cond_source.rs) [join_cond_verus.rs](join_cond_verus.rs) | MISSING_IN_VERUS | ❌ |
| `set_thread_data_area` [set_thread_data_area.diff](set_thread_data_area.diff) [set_thread_data_area_source.rs](set_thread_data_area_source.rs) [set_thread_data_area_verus.rs](set_thread_data_area_verus.rs) | MISMATCH | ❌ |
| `thread_state` [thread_state.diff](thread_state.diff) [thread_state_source.rs](thread_state_source.rs) [thread_state_verus.rs](thread_state_verus.rs) | MISMATCH | ❌ |
| `thread_state_mut` | MATCH | ✅ |
| `wakeup` [wakeup.diff](wakeup.diff) [wakeup_source.rs](wakeup_source.rs) [wakeup_verus.rs](wakeup_verus.rs) | MISMATCH | ❌ |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | EXTRA_IN_VERUS | ❌ |

## Inconsistent Structs

- `InterruptedThread` [struct_InterruptedThread_verus.rs](struct_InterruptedThread_verus.rs): EXTRA_IN_VERUS
- `ReadyThread` [struct_ReadyThread_verus.rs](struct_ReadyThread_verus.rs): EXTRA_IN_VERUS
- `SleepingThread` [struct_SleepingThread.diff](struct_SleepingThread.diff) [struct_SleepingThread_source.rs](struct_SleepingThread_source.rs) [struct_SleepingThread_verus.rs](struct_SleepingThread_verus.rs): MISMATCH
