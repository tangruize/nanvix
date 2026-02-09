# Review: interrupted_process (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedProcess::find_thread` / `find_thread_mut` (exec),
  `spec_find_thread_integration_obligation` (spec),
  `lemma_find_thread_refinement_assumption` (proof).
  **Description:** The executable search is still not verified. The new
  “integration obligation” is only a spec predicate equating a real result to
  `spec_find_thread`, but there is no proof that the real `iter().find(...)`
  logic matches it. This is the same trust gap as before, just renamed.
  **Suggested Fix:** Provide a verified wrapper or integration proof that
  directly models the iterator-based search and discharges the obligation.

### Medium
- **Location:** `InterruptedProcess::resume` (exec), `spec_admission_time_valid`
  (spec).
  **Description:** `resume()` still only requires `admission_time >= 0` and does
  not require or enforce `spec_admission_time_valid()`. The new text marks it
  as an “integration obligation,” but there is no enforcement or proof that the
  oracle corresponds to `clock::now()`.
  **Suggested Fix:** Strengthen the precondition (or add a verified wrapper) to
  require `spec_admission_time_valid(admission_time, clock_state)` at call sites.

- **Location:** `spec_resume_reason_integration_obligation` (spec),
  `InterruptedProcess::resume` and standalone `interrupt()` (exec).
  **Description:** The per-thread `interrupt_reason` propagation is still not
  modeled or proven. The new obligation is a stub equality predicate and is not
  linked to the exec code or any cross-module proof, so the property remains
  unverified.
  **Suggested Fix:** Carry reason tags in the model or provide a concrete
  integration proof that the thread module establishes this property and that
  this module preserves it.

### Low
- **Location:** `InterruptedProcess::state_mut` (exec/spec).
  **Description:** The model still abstracts away mutations through the returned
  `&mut ProcessState`, so effects on non-PID fields remain outside the verified
  scope.
  **Suggested Fix:** Model the relevant `ProcessState` fields or explicitly
  discharge this trust boundary in integration proofs.

## Positive Observations
- Coverage remains complete for all functions in the original source.
- Invariants (non-empty interrupted list, uniqueness, disjointness) are still
  explicit and preserved.
- No new `assume` or `external_body` usage appeared in this module.

## Summary
The updates mainly add documentation and integration-obligation stubs without
closing the prior semantic gaps. The verification still relies on unchecked
assumptions for `find_thread`, the clock oracle, and interrupt-reason propagation.
