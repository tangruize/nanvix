# Review: interrupted_process (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedProcess::find_thread` / `find_thread_mut` (exec, `verus/split/kernel/pm/process/state/interrupted.rs`)
  - **Description:** The verified functions are spec-only (`Ghost<Option<int>>`) and do not model the executable iterator-based search that returns `ThreadRef`/`ThreadRefMut`. The equivalence to the real implementation is only described as an integration obligation; no refinement proof ties the iterator order/predicate to `spec_find_thread`. This leaves a semantic gap for an API that can affect correctness of thread selection.
  - **Suggested Fix:** Add an exec-mode wrapper with `external_body` and a proved refinement lemma that the iterator search (interrupted -> sleeping -> zombie, predicate `id()==tid`) matches `spec_find_thread`, or provide a cross-module proof discharging `spec_find_thread_integration_obligation` at all call sites.

- **Location:** `InterruptedProcess::resume` / `spec_resume_reason_integration_obligation` (exec/spec)
  - **Description:** The model abstracts threads to IDs and does not verify that `InterruptedThread::resume()` propagates the interrupt reason into the resulting ready thread (`state.set_interrupt_reason`). This is a key safety property for termination/interrupt semantics and is only captured as an unproven integration obligation.
  - **Suggested Fix:** Extend the thread model to include an interrupt-reason field (or add a ghost map from thread IDs to reasons) and prove that `resume()` preserves it, or discharge the obligation in the thread module and reference that proof here.

### Medium
- **Location:** `InterruptedProcess::resume` / `spec_admission_time_valid` (exec/spec)
  - **Description:** The verified `resume()` only requires `admission_time >= 0`, while the original code uses `clock::now()`; equivalence relies on callers using `resume_with_valid_clock`, but this is not enforced. If any caller uses `resume()` directly, the model permits values inconsistent with the real clock.
  - **Suggested Fix:** Strengthen `resume()` to require `spec_admission_time_valid` or add a verified wrapper and prove that all call sites use it; otherwise document and audit all call sites.

- **Location:** `state()` / `state_mut()` (exec/spec)
  - **Description:** `ProcessState` is abstracted to a PID with no invariant tying the ghost PID to the real `ProcessState::pid()` or to mutable fields reachable through `state_mut()`. This makes the spec too weak to detect mis-linking or incorrect process-state mutations.
  - **Suggested Fix:** Add an integration obligation linking ghost PID to `ProcessState`’s real PID, and model (or explicitly constrain) the effects of `state_mut()` on relevant state fields.

### Low
- **Location:** `interrupt()` (exec/spec)
  - **Description:** The spec only models an ID-preserving transition with a reason tag; it does not capture other state changes performed by `SleepingThread::interrupt()` (if any). This is acceptable for a design-level model but leaves potential state invariants unverified.
  - **Suggested Fix:** If `SleepingThread::interrupt()` mutates additional state beyond reason/ID, model those fields or add an integration obligation in the thread module.

## Positive Observations
- All original functions are represented in the Verus split, and the additional `resume_with_valid_clock` wrapper provides a clean integration hook.
- Well-formedness invariants (non-empty interrupted list, no duplicates, pairwise disjointness) are clearly stated and preserved across transitions.
- The proof includes useful lemmas for subrange/no-duplicate preservation and a runnable-boundary projection lemma, improving cross-module composability.
- Spec/proof separation is clean and well-documented, with explicit trust-boundary commentary.

## Summary
The verification is well-structured and captures key list-level safety properties, but it relies on notable trust gaps for thread search semantics, interrupt-reason propagation, and clock admission time equivalence. Addressing these integration obligations (or enforcing their use at call sites) would significantly improve soundness and semantic equivalence to the kernel implementation.
