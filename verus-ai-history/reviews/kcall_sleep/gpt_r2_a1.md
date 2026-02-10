# Review: kcall_sleep (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `process_manager_sleep` external body (exec: `verus/split/kernel/pm/kcall/sleep.rs`).
  **Description:** The trust boundary for `ProcessManager::sleep` has no semantic postconditions tying the alarm time to the outcome. This leaves key liveness and time-correctness properties (e.g., returning `TimedOut` only after the alarm, or eventually returning once the alarm is reached) completely unmodeled in the kcall proof, so the verification only establishes result classification, not “sleep until alarm” behavior.
  **Suggested Fix:** Import and reference the verified PM specification (if available) or strengthen the external-body ensures to relate `alarm` to admissible outcomes (e.g., `TimedOut` implies alarm elapsed; `Ok`/`TimedOut` implies the call can eventually return), or add a refinement lemma that links this kcall to the PM’s verified behavior.

### Low
- **Location:** `ERROR_CODE_INVALID_ARGUMENT` (spec: `verus/split/kernel/pm/kcall/sleep.spec.rs`).
  **Description:** The error code is hard-coded as `22` instead of being tied to `ErrorCode::InvalidArgument`. This can silently drift if the error code mapping changes or if the project ports to a different errno set.
  **Suggested Fix:** Define a verified constant sourced from the error-code module’s Verus spec (or include a shared spec constant) and use that instead of a numeric literal.

- **Location:** `SleepResultView::GenericError` abstraction (spec: `verus/split/kernel/pm/kcall/sleep.spec.rs`).
  **Description:** The spec intentionally drops the error reason string ("invalid sleep time"). If any callers observe the reason (e.g., logs/tests), the spec is weaker than the implementation.
  **Suggested Fix:** Either model the reason string for the InvalidArgument case or explicitly prove (or document in the spec) that the reason string is not semantically observable for this kcall.

## Positive Observations
- Coverage is complete: the single original `sleep()` function is modeled end-to-end in `sleep_model`/`sleep_end_to_end`, including overflow handling and the 3‑arm result classification.
- The spec precisely models Duration normalization, checked-add overflow, and the Ok/TimedOut→Success mapping, matching the original control flow.
- Good separation of concerns: specs and lemmas are cleanly split into `sleep.spec.rs` and `sleep.proof.rs`, with the exec model mirroring the Rust implementation.

## Summary
The verification accurately captures the control-flow semantics and error classification of `kcall_sleep`, with solid well-formedness and overflow reasoning. The main gap is that the trust boundary for `ProcessManager::sleep` does not specify any timing/liveness behavior, so “sleep until alarm” correctness is not proven here. Tightening that interface (or linking to a verified PM spec) would make this proof more complete.
