# Review: interrupted_process (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedProcess::find_thread` / `find_thread_mut` (exec, `verus/split/kernel/pm/process/state/interrupted.rs`)
  - **Description:** Still spec-only with the same refinement assumption; no exec-mode wrapper or proof ties the iterator order/predicate to `spec_find_thread`. The trust gap remains unchanged.
  - **Suggested Fix:** Provide an exec wrapper with `external_body` plus a refinement lemma, or discharge `spec_find_thread_integration_obligation` in integration proofs at all call sites.

- **Location:** `InterruptedProcess::resume` / `spec_resume_reason_integration_obligation` (exec/spec)
  - **Description:** Interrupt-reason propagation is still not modeled; the spec continues to defer to an obligation with no proof that `InterruptedThread::resume()` sets the ready thread's interrupt reason. This remains unaddressed.
  - **Suggested Fix:** Extend the thread model to track interrupt reasons or reference a verified proof in the thread module that discharges the obligation.

### Medium
- **Location:** `InterruptedProcess::resume` / `spec_admission_time_valid` (exec/spec)
  - **Description:** `resume()` still only requires `admission_time >= 0`. The `resume_with_valid_clock()` wrapper exists, but there is no proof that all call sites use it, so clock equivalence remains optional.
  - **Suggested Fix:** Strengthen `resume()` to require `spec_admission_time_valid`, or prove that all call sites use `resume_with_valid_clock()`.

- **Location:** `spec_process_state_pid_integration_obligation` + new lemmas (spec/proof)
  - **Description:** New lemmas (`lemma_new_establishes_pid_obligation`, `lemma_from_sleeping_establishes_pid_obligation`) only show the obligation holds if the caller already provides a matching PID, but constructors do not require or establish the obligation. The link to real `ProcessState::pid()` is still unverified.
  - **Suggested Fix:** Add the obligation to constructors’ preconditions (or provide integration proofs that establish it at construction) and reference that proof at call sites.

### Low
- **Location:** `interrupt()` (exec/spec)
  - **Description:** The spec still models only ID + reason tag and does not capture other state changes made by `SleepingThread::interrupt()` (if any).
  - **Suggested Fix:** Model additional state effects or add a thread-module obligation to cover them.

## Positive Observations
- The PID-linking obligation and accompanying lemmas better document the intended integration boundary.
- Invariants and list-level safety properties remain well-specified and preserved across operations.
- Spec/proof separation and trust-boundary documentation remain clear and consistent.

## Summary
The update adds documentation/lemmas for PID linkage but does not enforce the obligation at construction, and the core trust gaps for search refinement, interrupt-reason propagation, and clock equivalence remain. Verification is still design-level with significant unproven assumptions, so it is not yet complete or sound w.r.t. executable behavior.
