# Review: interrupted_process (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedProcess::find_thread` / `InterruptedProcess::find_thread_mut` (exec),
  `lemma_find_thread_refinement_assumption` (proof).  
  **Description:** The verified functions are spec-only and do not model the executable
  iterator-based search in the real implementation. Correctness depends on an explicit
  refinement assumption, so bugs in the real search (wrong predicate or list order) are
  not caught.  
  **Suggested Fix:** If possible, model the executable search or add a verified wrapper
  that connects the actual `iter().find(...)` semantics to `spec_find_thread()`. At
  minimum, move the assumption to a centralized trust boundary inventory and require an
  explicit integration proof that the real search order/predicate matches the spec.

### Medium
- **Location:** `InterruptedProcess::resume` (exec), `spec_admission_time_valid` (spec).  
  **Description:** `resume()` only requires `admission_time >= 0`; it does not require
  `spec_admission_time_valid()` nor link the oracle to `clock::now()`. This makes the
  spec weaker than the real behavior and leaves equivalence to the clock source
  unproven.  
  **Suggested Fix:** Strengthen the precondition to require
  `spec_admission_time_valid(admission_time, clock_state)` (or an equivalent integration
  obligation) and propagate the clock-state parameter through the API or provide a
  verified wrapper that supplies `clock::now()` and discharges the obligation.

- **Location:** `InterruptedProcess::resume` and standalone `interrupt()` (exec/spec).  
  **Description:** The model only tracks thread IDs and does not capture the
  `interrupt_reason` propagation performed in the real thread transition
  (`InterruptedThread::resume()` and `SleepingThread::interrupt()`).
  If downstream code relies on this reason being set, this property is unverified here.  
  **Suggested Fix:** Extend the model to carry per-thread reason tags (even as ghost
  metadata) or add a cross-module proof that the thread-level verification establishes
  this property and that this module’s abstractions preserve it.

### Low
- **Location:** `InterruptedProcess::state_mut` (exec/spec).  
  **Description:** `state_mut()` is modeled as a pure ghost return with a strong frame
  condition. This abstracts away any real mutations to `ProcessState` via the returned
  mutable reference, so effects on non-PID fields are outside the model.  
  **Suggested Fix:** If `ProcessState` fields are relevant to correctness, model the
  needed subset (even as ghost) or explicitly document and audit this as a trust
  boundary in integration proofs.

## Positive Observations
- All functions from the original source are covered with verified counterparts.
- The well-formedness invariants (non-empty interrupted list, uniqueness, and
  pairwise disjointness) are explicit and preserved across operations.
- The spec/proof split is clean, with clear documentation of the modeling choices and
  trust boundaries.
- No unjustified `assume` or `external_body` appears in the core module.

## Summary
The verification provides solid coverage and strong structural invariants, but key
behavioral gaps remain in executable `find_thread` refinement and in clock/interrupt
reason modeling for `resume()` and `interrupt()`. Strengthening these links (or making
the integration obligations explicit) would raise confidence to an A-range grade.
