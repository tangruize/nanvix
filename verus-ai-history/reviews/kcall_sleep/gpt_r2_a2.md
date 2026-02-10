# Review: kcall_sleep (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `spec_pm_result_admissible` and `process_manager_sleep` postcondition (spec: `verus/split/kernel/pm/kcall/sleep.spec.rs`, exec: `verus/split/kernel/pm/kcall/sleep.rs`).
  **Description:** The attempted fix does not actually constrain PM behavior. `spec_pm_result_admissible` returns `true` for every PM result, so the `process_manager_sleep` ensures clause is vacuous. This still leaves the key liveness/timing property (“TimedOut only when the alarm is reached; sleep eventually returns”) unmodeled and unproven within this kcall, despite the comments claiming PM provides it.
  **Suggested Fix:** Replace the tautological predicate with a non-trivial admissibility relation imported from the PM/clock specs (or add a refinement lemma that links `process_manager_sleep` to those specs). If such properties are intentionally out-of-scope, explicitly state the kcall does not prove timing/liveness semantics.

### Low
- None.

## Positive Observations
- The error-code constant is now tied to `ErrorCode::InvalidArgument` via `lemma_error_code_matches`, preventing silent drift in the spec.
- The reason-string abstraction is explicitly documented in the spec, clarifying that only the error code is semantically relevant for this kcall.
- Coverage and equivalence of the core control flow (Duration normalization, checked-add overflow, and 3‑arm result classification) remain solid.

## Summary
The update fixed the error-code drift concern and clarified the reason-string abstraction, but the PM timing/liveness gap remains. The new `spec_pm_result_admissible` is a tautology, so the proof still does not capture “sleep until alarm” semantics. Strengthening that trust boundary (or clearly scoping it out) would complete the verification story.
