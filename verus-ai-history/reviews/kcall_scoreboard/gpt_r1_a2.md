# Review: kcall_scoreboard (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Location:** `ScoreBoard::dispatch()` / `ScoreBoard::handled()` modeling (exec/spec).
  **Description:** Error paths for `lock()`, `handled.down()` and `handled.up()` are still not modeled. The verification uses preconditions that assume success, so `SleepError`/`Error` propagation and the potential stuck states (mutex held, dispatcher waiting indefinitely) remain unverified and semantically divergent from the original.
  **Suggested Fix:** Model `Result` outcomes explicitly for these operations and prove that error paths preserve invariants (or show they are impossible using verified mutex/semaphore properties).

### Medium
- **Location:** `ScoreBoardSlot::init()` (exec/spec).
  **Description:** The precondition `!old(self).spec_is_initialized()` forbids re-initialization, but the original `ScoreBoard::init()` can be called multiple times (it overwrites the global). This makes the spec stronger than the implementation.
  **Suggested Fix:** Allow re-initialization or model idempotent overwrites to match the original behavior.

- **Location:** `ScoreBoardSlot::try_get_board()` (exec/spec).
  **Description:** The model returns only a boolean and never provides or ties a mutable reference to the contained `ScoreBoard`. As a result, the verification does not show that operations like `dispatch/handle/handled` are performed on the initialized global instance, leaving a coverage/equivalence gap.
  **Suggested Fix:** Introduce a method that returns a ghost/mutable reference to the board on success, or add a coupling lemma connecting slot initialization with allowed scoreboard operations.

- **Location:** Sequential abstraction (`scoreboard.rs`, Trust Boundary T4).
  **Description:** The proof still relies on a sequential model with `&mut self` transitions; no formal refinement is provided to justify that this abstraction matches the concurrent behavior enforced by mutex + semaphores.
  **Suggested Fix:** Add a refinement argument or permissions-based concurrency model to justify the sequential abstraction.

### Low
- **Location:** `impl Debug for KcallArgs` and module-level `init()` (original `kcall/mod.rs`).
  **Description:** Formatting/logging behavior remains out of scope; these functions still have no verified counterparts.
  **Suggested Fix:** Either model these explicitly or document them as intentionally excluded from coverage.

## Positive Observations
- The global singleton access pattern is now modeled via `ScoreBoardSlot`, including the `TryAgain` error case.
- The `completed_cycles` counter is now a ghost field, eliminating the prior semantic divergence.
- `try_handle()` models the `try_down()` failure path and proves state preservation on failure.

## Summary
The update improves coverage and fixes the ghost-cycle divergence, but key error paths for dispatch/handled and the lack of a formal concurrency refinement still leave important gaps. Addressing those remaining items would make the verification substantially closer to complete and semantically equivalent.
