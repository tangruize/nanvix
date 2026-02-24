# Exec Consistency Fix: interrupted_process

## Summary
- Mismatches fixed: 0
- Missing functions added: 0
- Documented equivalences: 8 function mismatches + 2 extra items (all justified)

All 8 function mismatches and the extra function/struct are due to the
verification model's type abstraction (`Box<ProcessState>` → `u64`,
`NonEmptyVecDeque<*Thread>` → `Vec<u64>`) and Verus limitations (no
reference-typed returns). No executable logic was changed — the algorithms
are identical under the abstraction.

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `InterruptedProcess` (struct) | Documented equivalence | Type abstraction: `Box<ProcessState>` → `pid: u64`, `NonEmptyVecDeque<InterruptedThread>` → `Vec<u64>`, `Option<NonEmptyVecDeque<SleepingThread>>` → `Vec<u64>` (empty = None), `Option<NonEmptyVecDeque<ZombieThread>>` → `Vec<u64>` (empty = None). Fundamental to verification model. Fields made `pub` for Verus proof ergonomics. |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | Documented equivalence | Same algorithm: construct with no sleeping threads. Parameter types differ by abstraction (`Box<ProcessState>` → `u64` pid, `NonEmptyVecDeque<InterruptedThread>` → `Vec<u64>`, `Option<NonEmptyVecDeque<ZombieThread>>` → `Vec<u64>`). `sleeping_threads: None` maps to `sleeping_thread_ids: Vec::new()`. Semantically equivalent. |
| `from_sleeping` [from_sleeping.diff](from_sleeping.diff) [from_sleeping_source.rs](from_sleeping_source.rs) [from_sleeping_verus.rs](from_sleeping_verus.rs) | Documented equivalence | Same algorithm: construct with all four components. Parameter types differ by same abstraction as `new`. Semantically equivalent. |
| `state` [state.diff](state.diff) [state_source.rs](state_source.rs) [state_verus.rs](state_verus.rs) | Documented equivalence | Original returns `&self.state` (`&ProcessState`). Verus returns `self.pid` (`u64`). Since `ProcessState` is abstracted to PID, this is the model equivalent. Integration obligation `spec_process_state_pid_integration_obligation` documents the formal link. |
| `state_mut` [state_mut.diff](state_mut.diff) [state_mut_source.rs](state_mut_source.rs) [state_mut_verus.rs](state_mut_verus.rs) | Documented equivalence | Original returns `&mut self.state` (`&mut ProcessState`). Verus returns `self.pid` (`u64`). Same abstraction as `state()`. Frame condition (no mutation) holds trivially since PID is never mutated. |
| `resume` [resume.diff](resume.diff) [resume_source.rs](resume_source.rs) [resume_verus.rs](resume_verus.rs) | Documented equivalence (Verus limitation) | Core algorithm identical: pop front interrupted thread, resume it (ID-preserving), create `RunnableProcess` with that thread as only ready thread. **Difference**: Verus version takes extra `admission_time: u64` oracle parameter because original internally calls `clock::now()` inside `ReadyThread::from_state()`, which is a HAL boundary outside this module's scope. Oracle pattern documented in spec with `spec_admission_time_valid()` integration obligation. `Vec::remove(0)` is equivalent to `NonEmptyVecDeque::pop_front()`. |
| `find_thread` [find_thread.diff](find_thread.diff) [find_thread_source.rs](find_thread_source.rs) [find_thread_verus.rs](find_thread_verus.rs) | Documented equivalence (Verus limitation) | Original returns `Option<ThreadRef<'_>>` via `iter().find()` across interrupted → sleeping → zombie. Verus returns `Ghost<Option<int>>` via `spec_find_thread()`. **Verus limitation**: cannot express reference-typed return values (`ThreadRef<'_>`). Spec model captures same search order and priority. Trust gap documented with `lemma_find_thread_refinement_assumption` and integration obligation `spec_find_thread_integration_obligation`. |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) [find_thread_mut_source.rs](find_thread_mut_source.rs) [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | Documented equivalence (Verus limitation) | Same limitation as `find_thread`. Returns `Ghost<Option<int>>` instead of `Option<ThreadRefMut<'_>>`. Frame condition ensures `self` is unchanged. |
| `interrupt` [interrupt.diff](interrupt.diff) [interrupt_source.rs](interrupt_source.rs) [interrupt_verus.rs](interrupt_verus.rs) | Documented equivalence | Original: `thread.interrupt(InterruptReason::Killed)` on `SleepingThread` → `InterruptedThread`. Verus: `(sleeping_tid, 0u64)` → ID and reason tag. ID-preserving under abstraction. `INTERRUPT_REASON_KILLED` spec constant (0) models the enum variant. |
| `resume_with_valid_clock` (EXTRA) | Justified — kept | Verification-only helper wrapping `resume()` with clock validation precondition. Documented as not existing in original source. Provides stronger entry point for integration proofs with clock access. No new exec logic — delegates to `resume()`. |
| `RunnableProcess` (EXTRA struct) | Justified — kept | Boundary model of sibling module's `RunnableProcess`. Required for `resume()` return type. Well-formedness predicate mirrors original's structural invariants. |

## Verification Model

The verification model abstracts all complex kernel types to integer IDs:
- `Box<ProcessState>` → `pid: u64` (identity tracking only)
- `NonEmptyVecDeque<InterruptedThread>` → `Vec<u64>` (thread IDs, len ≥ 1)
- `Option<NonEmptyVecDeque<SleepingThread>>` → `Vec<u64>` (empty = None)
- `Option<NonEmptyVecDeque<ZombieThread>>` → `Vec<u64>` (empty = None)
- `InterruptReason::Killed` → spec constant `INTERRUPT_REASON_KILLED = 0`

This abstraction is fundamental and well-documented in the module's header
comments, spec file, and integration obligations.

## Trust Gaps (documented in source)

1. **Per-thread state mutation**: `InterruptedThread::resume()` calls
   `set_interrupt_reason()` — not modeled (threads are IDs). Deferred to
   thread module verification.
2. **Executable search**: `find_thread()`/`find_thread_mut()` iterator-based
   search is not verified — spec model only. Verus cannot express
   reference-typed returns.
3. **ProcessState PID link**: No verified link between model PID and real
   `ProcessState::pid()`. Integration obligation defined.
4. **Clock oracle**: `admission_time` parameter replaces internal
   `clock::now()` call. Integration obligation via `spec_admission_time_valid()`.

## Verification: PASS

```
verification results:: 38 verified, 0 errors
```
