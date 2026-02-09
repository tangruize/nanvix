# Review: interrupted_process (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedProcess::find_thread` / `find_thread_mut` (exec),
  `spec_find_thread_integration_obligation` (spec),
  `lemma_find_thread_refinement_assumption` (proof).
  **Description:** The executable search is still unverified. The new obligations
  remain spec-only and do not connect the real `iter().find(...)` logic to the
  spec, so bugs in search predicate or ordering would not be caught. This is the
  same trust gap as before.
  **Suggested Fix:** Provide a verified wrapper or an integration proof that
  models the iterator search and discharges the obligation, or explicitly mark
  it as a trusted boundary with audit requirements.

### Medium
- **Location:** `resume()` and `resume_with_valid_clock()` (exec),
  `spec_admission_time_valid` (spec).
  **Description:** A verified wrapper now enforces `spec_admission_time_valid`,
  but the base `resume()` still accepts any non-negative oracle and there is no
  proof that all call sites use the wrapper. The clock linkage remains optional,
  so equivalence to `clock::now()` is still not guaranteed globally.
  **Suggested Fix:** Either strengthen `resume()` itself or add a proof that all
  external callers route through `resume_with_valid_clock()`.

- **Location:** `spec_resume_reason_integration_obligation` (spec),
  `lemma_interrupt_reason_satisfies_obligation` (proof).
  **Description:** The new lemma is a tautology about equal tags and does not
  connect to the thread module’s actual `resume()` behavior. The per-thread
  interrupt reason propagation remains unverified in this module.
  **Suggested Fix:** Add a cross-module proof that the real thread transition
  sets the interrupt reason and that this module’s abstractions preserve it.

### Low
- **Location:** `InterruptedProcess::state_mut` (exec/spec).
  **Description:** The model still abstracts away mutations through the returned
  `&mut ProcessState`, so effects on non-PID fields are outside the verified
  scope.
  **Suggested Fix:** Model relevant `ProcessState` fields or explicitly discharge
  this trust boundary in integration proofs.

## Positive Observations
- The new `resume_with_valid_clock()` wrapper is a concrete improvement for
  callers that can supply a verified clock state.
- Coverage and core invariants remain intact, and the module still avoids
  `assume`/`external_body` usage.

## Summary
The update adds a stronger wrapper for the clock oracle, but the original
high-risk gaps remain: `find_thread` is still unverified, clock linkage is not
mandatory for all callers, and interrupt-reason propagation lacks a real
cross-module proof. Verification is improved but not yet fully sound.
