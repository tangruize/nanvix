# Exec Integrity: interrupted_process

## Summary
- Total differences: 12
- Acceptable (ghost annotations / type abstraction): 12
- Fixed: 0
- Unfixable (documented): 0

## Files Compared
- **Original**: `src/kernel/src/pm/process/state/interrupted.rs`
- **Verified exec**: `verus/split/kernel/pm/process/state/interrupted.rs`
- **Spec**: `verus/split/kernel/pm/process/state/interrupted.spec.rs`
- **Proof**: `verus/split/kernel/pm/process/state/interrupted.proof.rs`

## Differences

| # | Type | Location | Description | Action |
|---|------|----------|-------------|--------|
| 1 | TYPE_CHANGE | `struct InterruptedProcess` | Fields changed from `Box<ProcessState>`, `Option<NonEmptyVecDeque<SleepingThread>>`, `NonEmptyVecDeque<InterruptedThread>`, `Option<NonEmptyVecDeque<ZombieThread>>` to `pid: u64`, `sleeping_thread_ids: Vec<u64>`, `interrupted_thread_ids: Vec<u64>`, `zombie_thread_ids: Vec<u64>`. Fields made `pub` for Verus proof ergonomics. | Acceptable — verification abstraction. Complex kernel types abstracted to ID-level model. Documented in module header. |
| 2 | TYPE_CHANGE | `struct RunnableProcess` (boundary model) | Boundary model struct added in verified code. Original imports `RunnableProcess` from sibling module. | Acceptable — required for verification module isolation. Verus modules are verified independently. |
| 3 | GHOST_ANNOTATION | `fn new()` | Added `requires` (non-empty, no-duplicates, disjointness) and `ensures` (wf, field equality). Parameters changed to match abstracted types (`u64`, `Vec<u64>`). | Acceptable — preconditions enforce original `NonEmptyVecDeque` invariant and ownership semantics. |
| 4 | GHOST_ANNOTATION | `fn from_sleeping()` | Added `requires` (non-empty, no-duplicates, pairwise disjointness) and `ensures` (wf, field equality). Parameters changed to match abstracted types. | Acceptable — preconditions enforce original invariants. |
| 5 | TYPE_CHANGE | `fn state()` | Returns `u64` instead of `&ProcessState`. | Acceptable — `ProcessState` abstracted to PID. Integration obligation `spec_process_state_pid_integration_obligation` documents the trust gap. |
| 6 | TYPE_CHANGE | `fn state_mut()` | Returns `u64` instead of `&mut ProcessState`. Added frame-preservation postconditions. | Acceptable — same abstraction as `state()`. Frame condition holds trivially since PID is never mutated. |
| 7 | TYPE_CHANGE | `fn resume()` | Extra parameter `admission_time: u64` added. Original calls `clock::now()` internally; verified uses oracle pattern. Return type changed to boundary `RunnableProcess` model. | Acceptable — clock is HAL boundary outside module scope. Oracle pattern with `spec_admission_time_valid()` integration obligation is documented. Core logic preserved: pop front interrupted thread, create singleton ready list, preserve other thread lists. |
| 8 | GHOST_ANNOTATION | `fn resume()` | Added `requires` (wf) and extensive `ensures` (PID preservation, ready thread identity, admission time, tail subrange, sleeping/zombie preservation, result wf). Proof block added for disjointness/uniqueness of resulting lists. | Acceptable — ghost annotations and proof block only. |
| 9 | INVENTED_FUNCTION | `fn resume_with_valid_clock()` | Verification-only wrapper that delegates to `resume()` with additional `spec_admission_time_valid()` precondition. Not in original source. | Acceptable — explicitly documented as verification-only helper. No exec logic beyond delegation to `resume()`. Provides stronger entry point for integration proofs with clock access. |
| 10 | TYPE_CHANGE | `fn find_thread()` | Returns `Ghost<Option<int>>` instead of `Option<ThreadRef<'_>>`. Implementation uses `spec_find_thread()` directly instead of `iter().find()` search. | Acceptable — Verus cannot express reference-typed returns. Trust gap documented with `lemma_find_thread_refinement_assumption` and `spec_find_thread_integration_obligation`. Search order (interrupted → sleeping → zombie) matches original. |
| 11 | TYPE_CHANGE | `fn find_thread_mut()` | Returns `Ghost<Option<int>>` instead of `Option<ThreadRefMut<'_>>`. Same spec-level model as `find_thread()`. Frame-preservation postconditions added. | Acceptable — same trust gap as `find_thread()`. |
| 12 | TYPE_CHANGE | `fn interrupt()` (standalone) | Takes `sleeping_tid: u64` instead of `SleepingThread`. Returns `(u64, u64)` instead of `InterruptedThread`. Models `InterruptReason::Killed` as spec constant `INTERRUPT_REASON_KILLED` (value 0). | Acceptable — ID-preserving transition. Reason tag modeled as abstract constant. `spec_resume_reason_integration_obligation` defines the formal contract for downstream verification. |

## Exec Logic Faithfulness

All original functions are present in the verified code. The core executable logic is preserved:

1. **`new()`**: Constructs struct with empty sleeping list — identical to original.
2. **`from_sleeping()`**: Constructs struct with all provided lists — identical to original.
3. **`state()`**: Returns process identity — faithful to original (PID abstraction).
4. **`state_mut()`**: Returns process identity — faithful to original (PID abstraction).
5. **`resume()`**: Pops front of interrupted list, creates singleton ready list, preserves sleeping/zombie — faithful to original logic. The `admission_time` oracle parameter replaces the internal `clock::now()` call (HAL boundary).
6. **`find_thread()`/`find_thread_mut()`**: Spec-level model with documented trust gap. Search priority order matches original.
7. **`interrupt()`**: ID-preserving with Killed reason — faithful to original.

## Trust Gaps (Documented)

1. **Per-thread state mutation**: `InterruptedThread::resume()` sets `interrupt_reason` — not modeled (threads are IDs). Covered by `spec_resume_reason_integration_obligation`.
2. **ProcessState PID linking**: PID field assumed to match `ProcessState::pid()`. Covered by `spec_process_state_pid_integration_obligation`.
3. **find_thread iterator search**: Executable `iter().find()` not verified. Covered by `lemma_find_thread_refinement_assumption` and `spec_find_thread_integration_obligation`.
4. **Clock/admission time**: `clock::now()` not modeled. Covered by `spec_admission_time_valid()` oracle pattern.

## Prohibited Patterns

- No `assume` statements found.
- No `admit` statements found.
- No unjustified `external_body` found.

## Verification Status
- Before: PASS (37 verified, 0 errors)
- After: PASS (37 verified, 0 errors) — no changes needed
