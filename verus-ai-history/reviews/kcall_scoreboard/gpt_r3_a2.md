# Review: kcall_scoreboard (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Error-path modeling for `SleepError::Generic` and `Error` is still missing** (exec: `ScoreBoard::dispatch`, `ScoreBoard::handled`, spec: `spec_dispatch_*`).
  - **Description:** The original implementation can return `SleepError::Generic` from `lock()` and `handled.down()` and can return `Error` from `dispatched.up()`/`handled.up()` via `Condvar::notify_first`. The updated model still only distinguishes `LockFailed` (Interrupted) and `DownInterrupted`, and continues to treat `up()` as infallible under a trust-boundary note. This is not equivalent to the real behavior and leaves a real error outcome unmodeled.
  - **Suggested Fix:** Add a `GenericError` (or equivalent) path to `DispatchOutcome` and model `up()`/`down()` failures explicitly in the spec/exec, with proofs that capture state after partial progress.

### Medium
- **`dispatch()` interruption is still modeled only from the Signaled phase** (spec: `spec_dispatch_interrupted`; exec: `dispatch` postconditions).
  - **Description:** Although new lemmas characterize abandonment from any active phase, the actual `dispatch()` specification and ensures still force the interrupted case to be a Signaled-phase stuck state. In the real implementation, `handled.down()` can be interrupted after the handler has consumed the signal or set the result, so the modeled behavior remains too strict and not semantically equivalent.
  - **Suggested Fix:** Extend `dispatch()` to allow interruption from any active phase, or make the interrupted outcome nondeterministically equal to `spec_abandon_dispatch` of any active-phase view.

- **Concurrency/refinement gap for `handle()` remains** (exec: `handle(&mut self)`; docs T4).
  - **Description:** The verified model still requires exclusive access for `handle()` and assumes sequential ordering, while the real implementation uses `handle(&self)` with atomic `try_down()`. This remains an assumption and not a proven refinement of the concurrent behavior.
  - **Suggested Fix:** Add a stronger refinement argument or an explicit rely/guarantee lemma that justifies the shared-access `handle()` semantics.

### Low
- **Coverage still excludes `impl Debug for KcallArgs` and module `init()`** (docs: Verification Scope).
  - **Description:** These functions are still marked out-of-scope and have no verified counterparts, so coverage is not complete under the stated criterion.
  - **Suggested Fix:** Add trivial verified stubs or explain why they can be safely excluded under the project’s coverage policy.

- **Liveness remains unproven (now explicitly scoped out)** (docs: Verification Scope).
  - **Description:** The update only documents that progress is out of scope; no liveness properties are proven. Under the review criteria, key liveness properties are still missing.
  - **Suggested Fix:** Add progress assumptions/lemmas or incorporate a minimal liveness proof for non-interrupted dispatch cycles.

## Positive Observations
- Added explicit documentation that liveness is out of scope and expanded abandonment lemmas.
- Proofs continue to establish safety invariants and data integrity across the protocol.

## Summary
The update adds helpful documentation and generalized abandon lemmas, but the main semantic gaps remain: unmodeled generic error outcomes, interruption still fixed to the Signaled phase in `dispatch()`, and the concurrency refinement gap for `handle()`. The verification remains safety-focused and not fully equivalent to the original implementation’s error behavior.
