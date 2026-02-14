# Exec Consistency Fix: sleeping_thread

## Summary
- Mismatches fixed: 0 (all 8 are documented equivalences)
- Missing functions added: 1 (`join_cond`)
- Documented equivalences: 8
- Extra exec functions retained: 1 (`clock_now`, justified)
- Extra structs retained: 2 (`ReadyThread`, `InterruptedThread`, justified)

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `from_state` | Documented equivalence | Exec body identical (`Self { state, alarm }`). Type changes: `Box<ThreadState>`→`ThreadState` (Box transparent, documented in module header), `Option<SystemTime>`→`Option<int>` (type modeling), `pub(super)`→`pub` (Verus proof ergonomics). |
| `wakeup` | Documented equivalence | Exec body identical (`ReadyThread::from_state(self.state)`). Only added Verus requires/ensures annotations. |
| `interrupt` | Documented equivalence | Exec body identical (`InterruptedThread::from_state(self.state, reason)`). `InterruptReason`→`int` (type modeling for enum, documented with `spec_valid_reason` predicate). |
| `id` | Documented equivalence | Exec body identical (`self.state.id()`). Only added Verus requires/ensures annotations. |
| `thread_state` | Documented equivalence | Exec body identical (`&self.state`). Only added Verus requires/ensures annotations. |
| `alarm` | Documented equivalence | Exec body identical (`self.alarm`). Return type `Option<SystemTime>`→`Option<int>` (type modeling). |
| `set_thread_data_area` | Documented equivalence | Exec body identical (`self.state.store_thread_data_area(user_tda)`). Interleaved `proof {}` and `let ghost` blocks are ghost code stripped at compilation. `Option<VirtualAddress>`→`Option<int>` (type modeling). |
| `get_thread_data_area` | Documented equivalence | Exec body identical (`self.state.get_thread_data_area()`). Return type `Option<VirtualAddress>`→`Option<int>` (type modeling). |
| `join_cond` | Added (MISSING_IN_VERUS) | Added as `#[verifier::external]` with `Condvar` stub, following the identical pattern in `interrupted.rs`. `Condvar` is an opaque sync primitive elided from the entire verification model (project-wide architectural decision). The function is a read-only pass-through with no effect on verified properties. |
| `clock_now` | Retained (EXTRA_IN_VERUS) | Boundary model of `clock::now()` used by `ReadyThread::from_state()`. Documented `#[verifier::external_body]` helper with postcondition `result >= 0`. Required for verifying `wakeup()` transition. |
| `ReadyThread` (struct) | Retained (EXTRA_IN_VERUS) | Boundary model of sibling module type, needed to verify `wakeup()` state transition. Documented with cross-module verification obligations. |
| `InterruptedThread` (struct) | Retained (EXTRA_IN_VERUS) | Boundary model of sibling module type, needed to verify `interrupt()` state transition. Documented with cross-module verification obligations. |
| `SleepingThread` (struct) | Documented equivalence | Fields match original: `state` (`Box<ThreadState>`→`ThreadState`, Box transparent) and `alarm` (`Option<SystemTime>`→`Option<int>`, type modeling). Fields are `pub` for Verus proof ergonomics (documented). |

## Type Modeling Summary

All type changes are part of the documented verification model (see module header):
- `Box<ThreadState>` → `ThreadState`: Box is a transparent wrapper, elided.
- `Option<SystemTime>` → `Option<int>`: Abstract timestamp modeling.
- `InterruptReason` → `int`: Enum modeled as int tag (0=Killed, 1=TimedOut) with `spec_valid_reason` predicate.
- `VirtualAddress` → `int`: Already modeled as `Option<int>` in ThreadState.
- `Condvar` → elided (opaque sync primitive, project-wide decision).

## Verification: PASS
- 33 verified, 0 errors
- No assume, admit, or unjustified external_body added
- Only justified `#[verifier::external]` on `join_cond()` (Condvar is opaque sync type, consistent with `interrupted.rs` pattern)
