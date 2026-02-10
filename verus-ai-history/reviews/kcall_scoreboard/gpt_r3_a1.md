# Review: kcall_scoreboard (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Missing error-path modeling for semaphore/mutex failures** (exec: `ScoreBoard::dispatch`, `ScoreBoard::handled`, spec: `spec_dispatch_*`).
  - **Description:** The original code can return `SleepError::Generic` from `lock()` or `handled.down()` and can return `Error` from `dispatched.up()`/`handled.up()` (via `Condvar::notify_first`). The verified model only represents `LockFailed` (interrupted lock) and `DownInterrupted`, and treats `up()` as infallible. This omits real error behaviors and makes the spec too strong relative to the implementation.
  - **Suggested Fix:** Extend `DispatchOutcome` (or use `Result`) to include a `GenericError` case, model `up()`/`down()` failures explicitly, and add specs/lemmas for these paths (including state after partial progress).

### Medium
- **Interruption modeled only from the Signaled phase** (spec: `spec_dispatch_interrupted`, exec: `dispatch` ensures).
  - **Description:** In the implementation, `handled.down()` can be interrupted after the handler has already consumed `dispatched` or even after it has set the result. The spec only allows interruption from the `Signaled` phase, forcing `result` to remain unchanged and `dispatched_value` to stay at 1. This is stricter than the real behavior and breaks semantic equivalence for some interleavings.
  - **Suggested Fix:** Allow interruption from any active phase (Signaled/Dispatched/Handled) within `dispatch()` or model the interruption nondeterministically and relate it to `spec_abandon_dispatch` to cover all valid states.

- **Sequential model assumes mutual exclusion between `handle()` and `dispatch()`** (exec: `handle(&mut self)` and docs T4).
  - **Description:** The verified model requires exclusive access for `handle()` and assumes a sequential ordering, while the real code uses `handle(&self)` with an atomic `try_down()`. This assumption is documented but not proven; it leaves a refinement gap for concurrent executions.
  - **Suggested Fix:** Either model `handle()` with shared access and an atomic `try_down()` spec (if supported), or add a formal refinement/assumption lemma explicitly bounding the concurrent behavior and its effect on `args`/`result` integrity.

### Low
- **Coverage gaps for non-scoreboard functions** (orig: `impl Debug for KcallArgs`, module-level `pub fn init()`; not modeled in verified code).
  - **Description:** The verification explicitly excludes these functions. This violates the strict “all functions covered” criterion, even though these are low-risk wrappers/formatting.
  - **Suggested Fix:** Add trivial verified stubs or spec notes showing they are behaviorally irrelevant, or move them into a separate verified/no-verify module with explicit justification.

- **No liveness guarantee beyond stuck-state characterization** (spec/proof).
  - **Description:** The proofs focus on safety (state machine and data integrity) and characterize stuck states, but do not prove progress (e.g., “if no interrupt occurs, dispatch eventually returns”).
  - **Suggested Fix:** Add explicit progress assumptions/lemmas (e.g., handler eventually calls `handled()` when signaled) or document liveness as out of scope in the verification scope.

## Positive Observations
- The spec cleanly captures the four-phase handshake and ties semaphore values to phases.
- Argument and result integrity are proved, along with mutual exclusion during active phases.
- Spec/proof separation is clean and comprehensive, with helpful documentation of trust boundaries.

## Summary
The verification provides strong safety guarantees for the sequential protocol and cleanly documents its assumptions, but it omits real error-path behaviors and constrains interruption to a single phase. Closing the missing error modeling and widening the interruption semantics would significantly improve semantic equivalence and coverage.
