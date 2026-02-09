# Review: runnable (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- Location: RunnableProcess::{state,state_mut,find_thread,find_thread_mut,earliest_admission_time} (exec), runnable.rs Trust Boundary section
  - Description: These functions are still omitted from the exec-level model; comments explicitly state they are spec-only or elided. This means coverage is still incomplete and real behavior (especially `state_mut`) remains unchecked.
  - Suggested Fix: Provide verified exec-level wrappers or adapters that expose abstract results (e.g., tag/index for `find_thread*`, int for `earliest_admission_time`, frame/PID-preservation for `state/state_mut`) and connect them via refinement lemmas.

### Medium
- Location: EXIT_STATUS_INTERRUPTED (spec), terminate() (exec)
  - Description: The hard-coded constant `4` remains, with only a TODO to validate it against `ErrorCode::Interrupted.into()`. The external_body helper was removed, but the proof still does not establish equivalence to the real error code value.
  - Suggested Fix: Add a verified cross-module lemma or import a verified constant from sysapi, so the exit status is linked to the actual error definition.
- Location: RunnableProcess::wf / spec_ids_disjoint (spec)
  - Description: Thread-ID disjointness across ready/interrupted/sleeping/zombie lists is still excluded from `wf()` and treated as a trust assumption. This weakens safety reasoning about membership/search and allows inconsistent ghost states.
  - Suggested Fix: Strengthen `wf()` (or require a disjointness invariant on public APIs) and prove it is preserved by run/terminate/wakeup/add_thread.

### Low
- Location: RunnableProcess::run (exec/spec)
  - Description: The interrupt reason, context pointer, and user TDA remain elided/unconstrained, so the verified model does not capture the dataflow of these values.
  - Suggested Fix: Model these as ghost outputs with minimal constraints or add boundary specs linking to thread-level run semantics.

## Positive Observations
- The verification still passes and the core transition proofs for run/terminate/wakeup/add_thread remain strong.
- Removing the `external_body` helper for the exit status is a small soundness improvement, even though the value is still unchecked.

## Summary
The rereview shows that the key outstanding issues from the previous review were not actually fixed: coverage is still incomplete and disjointness/exit-status linkage remain unproven. The module is closer to soundness than before, but verification is still not complete.
