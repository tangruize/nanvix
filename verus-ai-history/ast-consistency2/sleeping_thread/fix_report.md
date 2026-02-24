# Exec Consistency Fix: sleeping_thread

## Summary
- Mismatches fixed: 0 (all 8 are documented equivalences)
- Missing functions added: 1 (`join_cond` [join_cond.diff](join_cond.diff) | [join_cond_source.rs](join_cond_source.rs) | [join_cond_verus.rs](join_cond_verus.rs))
- Documented equivalences: 8
- Extra exec functions retained: 1 (`clock_now` [clock_now_verus.rs](clock_now_verus.rs), justified)
- Extra structs retained: 2 (`ReadyThread` [struct_ReadyThread_verus.rs](struct_ReadyThread_verus.rs), `InterruptedThread` [struct_InterruptedThread_verus.rs](struct_InterruptedThread_verus.rs), justified)

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `from_state` [from_state.diff](from_state.diff) | [from_state_source.rs](from_state_source.rs) | [from_state_verus.rs](from_state_verus.rs) | Documented equivalence | Exec body identical (`Self { state, alarm }`). Type changes: `Box<ThreadState>`→`ThreadState` (Box transparent, documented in module header), `Option<SystemTime>`→`Option<int>` (type modeling), `pub(super)`→`pub` (Verus proof ergonomics). |
| `wakeup` [wakeup.diff](wakeup.diff) | [wakeup_source.rs](wakeup_source.rs) | [wakeup_verus.rs](wakeup_verus.rs) | Documented equivalence | Exec body identical (`ReadyThread::from_state(self.state)`). Only added Verus requires/ensures annotations. |
| `interrupt` [interrupt.diff](interrupt.diff) | [interrupt_source.rs](interrupt_source.rs) | [interrupt_verus.rs](interrupt_verus.rs) | Documented equivalence | Exec body identical (`InterruptedThread::from_state(self.state, reason)`). `InterruptReason`→`int` (type modeling for enum, documented with `spec_valid_reason` predicate). |
| `id` [id.diff](id.diff) | [id_source.rs](id_source.rs) | [id_verus.rs](id_verus.rs) | Documented equivalence | Exec body identical (`self.state.id()`). Only added Verus requires/ensures annotations. |
| `thread_state` [thread_state.diff](thread_state.diff) | [thread_state_source.rs](thread_state_source.rs) | [thread_state_verus.rs](thread_state_verus.rs) | Documented equivalence | Exec body identical (`&self.state`). Only added Verus requires/ensures annotations. |
| `alarm` [alarm.diff](alarm.diff) | [alarm_source.rs](alarm_source.rs) | [alarm_verus.rs](alarm_verus.rs) | Documented equivalence | Exec body identical (`self.alarm`). Return type `Option<SystemTime>`→`Option<int>` (type modeling). |
| `set_thread_data_area` [set_thread_data_area.diff](set_thread_data_area.diff) | [set_thread_data_area_source.rs](set_thread_data_area_source.rs) | [set_thread_data_area_verus.rs](set_thread_data_area_verus.rs) | Documented equivalence | Exec body identical (`self.state.store_thread_data_area(user_tda)`). Interleaved `proof {}` and `let ghost` blocks are ghost code stripped at compilation. `Option<VirtualAddress>`→`Option<int>` (type modeling). |
| `get_thread_data_area` [get_thread_data_area.diff](get_thread_data_area.diff) | [get_thread_data_area_source.rs](get_thread_data_area_source.rs) | [get_thread_data_area_verus.rs](get_thread_data_area_verus.rs) | Documented equivalence | Exec body identical (`self.state.get_thread_data_area()`). Return type `Option<VirtualAddress>`→`Option<int>` (type modeling). |
| `join_cond` [join_cond.diff](join_cond.diff) | [join_cond_source.rs](join_cond_source.rs) | [join_cond_verus.rs](join_cond_verus.rs) | Added (MISSING_IN_VERUS) | Added as `#[verifier::external]` with `Condvar` [struct_Condvar_verus.rs](struct_Condvar_verus.rs) stub, following the identical pattern in `interrupted.rs`. `Condvar` [struct_Condvar_verus.rs](struct_Condvar_verus.rs) is an opaque sync primitive elided from the entire verification model (project-wide architectural decision). The function is a read-only pass-through with no effect on verified properties. |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | Retained (EXTRA_IN_VERUS) | Boundary model of `clock::now()` used by `ReadyThread::from_state()`. Documented `#[verifier::external_body]` helper with postcondition `result >= 0`. Required for verifying `wakeup()` transition. |
| `ReadyThread` [struct_ReadyThread_verus.rs](struct_ReadyThread_verus.rs) (struct) | Retained (EXTRA_IN_VERUS) | Boundary model of sibling module type, needed to verify `wakeup()` state transition. Documented with cross-module verification obligations. |
| `InterruptedThread` [struct_InterruptedThread_verus.rs](struct_InterruptedThread_verus.rs) (struct) | Retained (EXTRA_IN_VERUS) | Boundary model of sibling module type, needed to verify `interrupt()` state transition. Documented with cross-module verification obligations. |
| `SleepingThread` [struct_SleepingThread.diff](struct_SleepingThread.diff) | [struct_SleepingThread_source.rs](struct_SleepingThread_source.rs) | [struct_SleepingThread_verus.rs](struct_SleepingThread_verus.rs) (struct) | Documented equivalence | Fields match original: `state` (`Box<ThreadState>`→`ThreadState`, Box transparent) and `alarm` [alarm.diff](alarm.diff) | [alarm_source.rs](alarm_source.rs) | [alarm_verus.rs](alarm_verus.rs) (`Option<SystemTime>`→`Option<int>`, type modeling). Fields are `pub` for Verus proof ergonomics (documented). |

## Type Modeling Summary

All type changes are part of the documented verification model (see module header):
- `Box<ThreadState>` → `ThreadState`: Box is a transparent wrapper, elided.
- `Option<SystemTime>` → `Option<int>`: Abstract timestamp modeling.
- `InterruptReason` → `int`: Enum modeled as int tag (0=Killed, 1=TimedOut) with `spec_valid_reason` predicate.
- `VirtualAddress` → `int`: Already modeled as `Option<int>` in ThreadState.
- `Condvar` [struct_Condvar_verus.rs](struct_Condvar_verus.rs) → elided (opaque sync primitive, project-wide decision).

## Verification: PASS
- 33 verified, 0 errors
- No assume, admit, or unjustified external_body added
- Only justified `#[verifier::external]` on `join_cond()` (Condvar is opaque sync type, consistent with `interrupted.rs` pattern)
