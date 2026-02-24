# Exec Consistency Fix: ready

## Summary
- Mismatches fixed: 0 (all 7 documented as semantically equivalent)
- Missing functions added: 0 (1 documented as intentionally omitted)
- Documented equivalences: 8

## Analysis

All AST mismatches are due to verification modeling decisions, not executable
logic changes. The Verus model abstracts HAL/sync boundary types that cannot
be expressed in Verus. These are all documented in the file header
(lines 26–51 of `ready.rs`).

### Key Modeling Abstractions

- `Box<ThreadState>` → `ThreadState` directly (Box is transparent wrapper).
- `SystemTime` → `int` (abstract timestamp).
- `clock::now()` → `clock_now()` external_body helper.
- `ContextInformation`, `FpuState` → elided (opaque HAL types).
- `Condvar` → elided (sync boundary); `join_cond()` omitted.
- `ErrorCode::Interrupted.into()` → `exit_status_interrupted_value()` external_body helper.
- `run()` return: 4-tuple with `*mut ContextInformation` → `RunResult` [struct_RunResult_verus.rs](struct_RunResult_verus.rs) struct (raw pointer omitted).

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | Documented equivalence | Omits `context`/`fpu_state` params (HAL boundary, cannot be modeled). Core logic identical: creates ThreadState + captures clock time. Documented in file header lines 28–29. |
| `from_state` [from_state.diff](from_state.diff) | [from_state_source.rs](from_state_source.rs) | [from_state_verus.rs](from_state_verus.rs) | Documented equivalence | Uses `ThreadState` instead of `Box<ThreadState>` (Box is transparent). Logic identical: wraps state + clock_now(). Documented in file header line 29. |
| `id` [id.diff](id.diff) | [id_source.rs](id_source.rs) | [id_verus.rs](id_verus.rs) | Documented equivalence | Exec body identical (`self.state.id()`). AST mismatch from Verus named-return syntax `(result: T)` required for ensures clauses. |
| `thread_state` [thread_state.diff](thread_state.diff) | [thread_state_source.rs](thread_state_source.rs) | [thread_state_verus.rs](thread_state_verus.rs) | Documented equivalence | Exec body identical (`&self.state`). AST mismatch from named-return syntax. |
| `admission_time` [admission_time.diff](admission_time.diff) | [admission_time_source.rs](admission_time_source.rs) | [admission_time_verus.rs](admission_time_verus.rs) | Documented equivalence | Exec body identical (`self.admission_time`). AST mismatch from return type `int` vs `SystemTime` and named-return syntax. |
| `run` [run.diff](run.diff) | [run_source.rs](run_source.rs) | [run_verus.rs](run_verus.rs) | Documented equivalence | Returns `RunResult` [struct_RunResult_verus.rs](struct_RunResult_verus.rs) struct instead of 4-tuple — necessary because `*mut ContextInformation` (raw HAL pointer) cannot be modeled. Omits `context_mut()` call (HAL only). Core logic preserved: `take_interrupt_reason` + `get_thread_data_area` + `RunningThread::from_state`. Documented in file header lines 37–38. |
| `terminate` [terminate.diff](terminate.diff) | [terminate_source.rs](terminate_source.rs) | [terminate_verus.rs](terminate_verus.rs) | Documented equivalence | Uses `exit_status_interrupted_value()` helper to model `ErrorCode::Interrupted.into()` — Verus cannot call `.into()` on external types. Semantically identical: both produce exit status = 4 (EINTR). Documented in file header line 34, spec constant at `ready.spec.rs:76`. |
| `join_cond` [join_cond_source.rs](join_cond_source.rs) | Intentionally omitted | Returns `Condvar` (opaque sync primitive). The underlying `ThreadState` model excludes `join_cond` [join_cond_source.rs](join_cond_source.rs) field entirely (documented in `state.rs` lines 49–53). Cannot be added without modifying the ThreadState model, which is a separate verified module. Documented in file header line 47 and spec file line 27. |
| `clock_now` [clock_now_verus.rs](clock_now_verus.rs) | Retained (justified) | external_body helper modeling `clock::now()`. Required because `clock::now()` is a kernel function unavailable in Verus. Postcondition `result >= 0` reflects non-negative `SystemTime`. |
| `exit_status_interrupted_value` [exit_status_interrupted_value_verus.rs](exit_status_interrupted_value_verus.rs) | Retained (justified) | external_body helper bridging spec/exec boundary for `EXIT_STATUS_INTERRUPTED()` constant. Required because Verus exec code cannot directly construct `int` literals from spec constants. |
| `set_interrupt_reason` [set_interrupt_reason_verus.rs](set_interrupt_reason_verus.rs) | Retained (justified) | Verified forwarding method providing checked path instead of unverified `thread_state_mut()`. Documented as verification-only API extension (lines 357–363). |
| `store_mutex_guard` [store_mutex_guard_verus.rs](store_mutex_guard_verus.rs) | Retained (justified) | Verified forwarding method for mutex accounting. Documented as verification-only API extension (lines 387–394). |
| `take_mutex_guard` [take_mutex_guard_verus.rs](take_mutex_guard_verus.rs) | Retained (justified) | Verified forwarding method for mutex accounting. Documented as verification-only API extension (lines 417–424). |

### Extra Structs

| Struct | Action | Justification |
|--------|--------|---------------|
| `ReadyThread` [struct_ReadyThread.diff](struct_ReadyThread.diff) | [struct_ReadyThread_source.rs](struct_ReadyThread_source.rs) | [struct_ReadyThread_verus.rs](struct_ReadyThread_verus.rs) | Documented equivalence | Fields model `Box<ThreadState>` as `ThreadState`, `SystemTime` as `int`. Fields are `pub` for Verus proof ergonomics (documented in file lines 106–108). |
| `RunResult` [struct_RunResult_verus.rs](struct_RunResult_verus.rs) | Retained (justified) | Return type for `run()`. Needed because original 4-tuple includes `*mut ContextInformation` which cannot be modeled in Verus. |
| `RunningThread` [struct_RunningThread_verus.rs](struct_RunningThread_verus.rs) | Retained (justified) | Boundary model of sibling module. Required to verify `run()` state transition. Cross-module check obligations documented (lines 175–181). |
| `ZombieThread` [struct_ZombieThread_verus.rs](struct_ZombieThread_verus.rs) | Retained (justified) | Boundary model of sibling module. Required to verify `terminate()` state transition. Cross-module check obligations documented (lines 216–224). |

## Verification: PASS

```
verification results:: 34 verified, 0 errors
Duration: 9s
```

No code changes were needed. All inconsistencies are verification modeling
decisions that are already documented in the source. The exec logic is
semantically equivalent to the original for all modeled properties.
