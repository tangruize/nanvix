# Review: interrupted_process (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- Location: InterruptedProcess::find_thread / find_thread_mut (exec, interrupted.rs) and spec_find_thread (spec, interrupted.spec.rs)
  - Description: These remain spec-only wrappers returning a ghost `Option<int>`. The new “refinement assumption” lemma is definitional and does not connect the real iterator-based search to the spec, so semantic equivalence is still unverified.
  - Suggested Fix: Provide an executable ghost search (if supported) or add a trusted refinement boundary (external_body/assume) that explicitly ties the real implementation to `spec_find_thread` and documents the trust.
- Location: InterruptedProcess::resume (exec/spec, interrupted.rs)
  - Description: Admission time is now a ghost oracle parameter, but there is still no spec linking it to `clock::now()` or any time model. This weakens equivalence to the original and leaves scheduling-relevant properties unproved.
  - Suggested Fix: Introduce a spec for `clock::now()` and require `admission_time` equals it (or add a time model in callers that is connected to this oracle).

### Low
- Location: Module docs in interrupted.rs and verification model summary in interrupted.spec.rs
  - Description: Comments still state that `InterruptReason` is elided and that `resume()` initializes admission times to `[0]`, which is no longer true. Documentation is now inconsistent with the model.
  - Suggested Fix: Update docs to reflect the oracle admission time and explicit `InterruptReason::Killed` tagging.

## Positive Observations
- `state()` and `state_mut()` are now pure ghost implementations, removing the prior external_body trust gap.
- `interrupt()` now models the `Killed` reason explicitly.
- Structural invariants and list disjointness proofs remain intact and well-scoped.

## Summary
The update removes the accessor soundness gap and improves interrupt modeling, but semantic equivalence for `find_thread()` and time-based admission remains unverified. Completing the clock linkage and either verifying or explicitly trusting the search logic is still needed for a fully sound review.
