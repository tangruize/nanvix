# Review: interrupted_process (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `InterruptedProcess::find_thread` / `find_thread_mut` (exec, `verus/split/kernel/pm/process/state/interrupted.rs`)
  - **Description:** Still spec-only with the same refinement assumption; there is no exec-mode wrapper or proof tying the iterator order/predicate to `spec_find_thread`. The trust gap remains unchanged despite the update.
  - **Suggested Fix:** Provide an exec wrapper with `external_body` plus a refinement lemma, or discharge `spec_find_thread_integration_obligation` in integration proofs at all call sites.

- **Location:** `InterruptedProcess::resume` / `spec_resume_reason_integration_obligation` (exec/spec)
  - **Description:** The interrupt-reason propagation is still not modeled; the spec continues to defer to an obligation with no proof that `InterruptedThread::resume()` sets the ready thread's interrupt reason. This was flagged before and remains unaddressed.
  - **Suggested Fix:** Extend the thread model to track interrupt reasons or reference a verified proof in the thread module that discharges the obligation.

### Medium
- **Location:** `InterruptedProcess::resume` / `spec_admission_time_valid` (exec/spec)
  - **Description:** `resume()` still only requires `admission_time >= 0`. The `resume_with_valid_clock()` wrapper exists, but there is no proof that all call sites use it, so the clock equivalence is still optional and unverifiable.
  - **Suggested Fix:** Strengthen `resume()` to require `spec_admission_time_valid`, or prove that all call sites use `resume_with_valid_clock()`.

- **Location:** `state()` / `state_mut()` and new `spec_process_state_pid_integration_obligation` (spec/proof)
  - **Description:** A PID-linking obligation was added, but constructors (`new`/`from_sleeping`) still do not require or establish it. The new lemma only shows preservation if the link already holds, so the core link to real `ProcessState::pid()` remains unverified.
  - **Suggested Fix:** Add the obligation to constructors’ preconditions (or provide integration proofs that establish it at construction) and reference that proof at call sites.

### Low
- **Location:** `interrupt()` (exec/spec)
  - **Description:** The spec still models only ID + reason tag and does not capture any other state changes made by `SleepingThread::interrupt()` (if any).
  - **Suggested Fix:** Model additional state effects or add a thread-module obligation to cover them.

## Positive Observations
- The new `spec_process_state_pid_integration_obligation` and lemma make the PID-linking trust boundary explicit and document the intended integration point.
- Invariants and list-level safety properties remain well-specified and are preserved across operations.
- Spec/proof separation and trust-boundary documentation remain clear and consistent.

## Summary
Most of the previously reported gaps remain: search refinement, interrupt-reason propagation, and clock equivalence are still unproven, and the new PID-link obligation is not enforced at construction. The verification remains design-level with significant trust assumptions; it is not yet complete or sound with respect to the executable behavior.
