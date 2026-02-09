# Review: runnable (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- Location: RunnableProcess::{state,state_mut,find_thread,find_thread_mut,earliest_admission_time} (exec), runnable.rs Trust Boundary section
  - Description: These functions remain omitted from the exec-level model; the file still states they are spec-only or elided. Coverage is still incomplete and the concrete behavior (especially `state_mut`) remains unverified.
  - Suggested Fix: Provide exec-level wrappers/adapters with abstract outputs (tag/index/int) and prove refinement to the spec models; add frame/PID preservation for state accessors.

### Medium
- Location: EXIT_STATUS_INTERRUPTED (spec), terminate() (exec)
  - Description: The exit status is still a hard-coded `4` with only a TODO to validate against `ErrorCode::Interrupted.into()`. The proof does not establish equivalence to the real error code.
  - Suggested Fix: Add a verified cross-module lemma or import the constant from the sysapi error module to tie the spec to the real value.
- Location: RunnableProcess::wf / spec_ids_disjoint (spec)
  - Description: Thread-ID disjointness across ready/interrupted/sleeping/zombie lists is still excluded from `wf()` and treated as a trust assumption, weakening safety reasoning about membership and search order.
  - Suggested Fix: Strengthen `wf()` (or require disjointness on public APIs) and prove preservation across run/terminate/wakeup/add_thread.

### Low
- Location: RunnableProcess::run (exec/spec)
  - Description: The interrupt reason, context pointer, and user TDA remain elided/unconstrained, so the verified model still does not capture their dataflow.
  - Suggested Fix: Model these as ghost outputs with minimal constraints or add boundary specs linking them to thread-level run semantics.

## Positive Observations
- Core state-transition proofs for run/terminate/wakeup/add_thread remain strong and verification still passes.
- No new soundness regressions were introduced in the updated files.

## Summary
The rereview shows no substantive fixes to the previously reported issues: coverage is still incomplete, disjointness and exit-status linkage remain unproven, and run’s auxiliary outputs are still abstracted away. Verification is not yet complete or fully sound for the original runnable API.
