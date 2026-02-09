# Review: interrupted_process (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- Location: InterruptedProcess::find_thread / find_thread_mut (exec, interrupted.rs) and spec_find_thread (spec, interrupted.spec.rs)
  - Description: These are still spec-only wrappers returning ghost values. The added documentation and “refinement assumption” lemma do not connect the executable iterator search to the spec, so semantic equivalence to the real implementation remains unverified.
  - Suggested Fix: Provide a trusted refinement boundary (external_body/assume) explicitly tying the real search to `spec_find_thread`, or an executable ghost search if Verus supports it.
- Location: InterruptedProcess::resume (exec/spec, interrupted.rs) and spec_admission_time_valid (spec)
  - Description: Although `spec_clock_now` and `spec_admission_time_valid` were added, `resume()` still only requires `admission_time >= 0` and does not require the oracle to equal `spec_clock_now(clock_state)`. The clock linkage is therefore not enforced, leaving equivalence to `clock::now()` unproven.
  - Suggested Fix: Strengthen `resume()`’s preconditions to require `spec_admission_time_valid(admission_time@, clock_state)` (or pass a clock_state ghost parameter), and update callers accordingly.

### Low
- None.

## Positive Observations
- Documentation has been updated to reflect the admission-time oracle and explicit `InterruptReason::Killed` tagging.
- `state()` and `state_mut()` remain pure ghost implementations with no external-body trust gap.
- Structural invariants and resume transition proofs remain consistent and verified.

## Summary
The new spec hooks and documentation are helpful, but the two main soundness gaps remain: unverified thread search behavior and an unenforced link between admission time and the real clock. Additional trusted boundaries or stronger preconditions are still required for full equivalence.
