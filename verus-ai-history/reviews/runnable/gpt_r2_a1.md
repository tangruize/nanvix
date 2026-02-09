# Review: runnable (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- Location: RunnableProcess::{state,state_mut,find_thread,find_thread_mut,earliest_admission_time} (exec), runnable.spec.rs notes
  - Description: These original functions have no verified exec-level counterparts. `find_thread*` and `earliest_admission_time` are only spec-modeled, and `state/state_mut` are omitted entirely, violating the stated coverage requirement and leaving real behavior (especially `state_mut`) unchecked.
  - Suggested Fix: Add exec-level wrappers with precise postconditions (e.g., return an abstract tag or index for `find_thread*`, return an int for `earliest_admission_time` with `ensures` tied to `spec_earliest_admission_time`, and add frame/PID-preservation specs for `state/state_mut`). Then link these to the original API via refinement lemmas or verified adapters.

### Medium
- Location: EXIT_STATUS_INTERRUPTED / exit_status_interrupted_value (spec/exec)
  - Description: The exit status is hard-coded to `4` and introduced via an `external_body` function, with only a TODO to validate the mapping to `ErrorCode::Interrupted.into()`. This is a trust gap: if errno values change, the proof still passes while behavior diverges.
  - Suggested Fix: Prove or import a verified constant from the sysapi error module, or add a cross-module lemma tying `EXIT_STATUS_INTERRUPTED()` to `ErrorCode::Interrupted.into()`; avoid `external_body` for a pure constant.
- Location: RunnableProcess::wf / spec_ids_disjoint (spec)
  - Description: Thread-ID disjointness across ready/interrupted/sleeping/zombie lists is explicitly not enforced in `wf()`. Proofs about search order and list membership rely on a trust assumption rather than a checked invariant, weakening equivalence and safety reasoning.
  - Suggested Fix: Strengthen `wf()` (or add a required invariant on public APIs) to include pairwise disjointness, and prove that run/terminate/wakeup/add_thread preserve it.

### Low
- Location: RunnableProcess::run (exec/spec)
  - Description: The returned `InterruptReason`, context pointer, and user TDA are elided or unconstrained, so the verified model does not capture the dataflow of these values from `ReadyThread::run()`.
  - Suggested Fix: Model these as ghost outputs with minimal constraints or introduce boundary specs that relate them to the thread-level run semantics.

## Positive Observations
- The spec clearly documents modeling choices, trust boundaries, and oracle usage, with good rationale.
- `run()`, `terminate()`, `wakeup()`, and `add_thread()` have strong postconditions on list contents and PID preservation, with supporting lemmas.
- Well-formedness (non-empty ready list, parallel arrays, non-negative times) is maintained and used to avoid spurious corner cases.

## Summary
The verification is solid for the core state-transition logic and includes careful proofs for selection and list updates, but it falls short of full coverage and relies on a few trust assumptions that should be turned into checked invariants or cross-module links.
