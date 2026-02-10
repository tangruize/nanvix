# Review: kcall_scoreboard (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `ScoreBoard::get_mut` and global `init()` (original `src/kernel/src/kcall/mod.rs`), missing from verified model (`scoreboard.rs`).
  **Description:** The verification does not cover the global singleton (`static mut SCOREBOARD`) initialization path or the `get_mut()` error behavior (returns `ErrorCode::TryAgain` when uninitialized). This violates the coverage requirement and leaves the most safety-sensitive part (unsafe global access) unverified.
  **Suggested Fix:** Add a verified wrapper that models the global `Option<ScoreBoard>` state and the `init()/get_mut()` behaviors, or mark these as `external_body` with a documented proof obligation and a checkable contract (e.g., post-init, `get_mut()` succeeds).

- **Location:** `ScoreBoard::dispatch`, `ScoreBoard::handle`, `ScoreBoard::handled` (exec/spec).
  **Description:** Error paths and blocking behavior are not modeled. The Verus model replaces `Result`-returning calls (`lock`, `try_down`, `down`, `up`) with preconditions that guarantee success. This omits `SleepError` and `Error` propagation and hides potential stuck states (e.g., interrupted sleep after locking or failure to signal handled), which are liveness-relevant in the original.
  **Suggested Fix:** Model `Result` outcomes explicitly (success/error branches) and prove that errors preserve invariants and do not corrupt state; or, if errors are deemed impossible by design, encode and justify those assumptions as explicit preconditions tied to verified properties of `Mutex`/`Semaphore`.

### Medium
- **Location:** `ScoreBoard::handle` signature and sequential model (`scoreboard.rs`).
  **Description:** The verified model is purely sequential and requires `&mut self` for `handle()`, which diverges from the original concurrent design where the handler runs while the dispatcher holds the mutex. This misses concurrency-specific safety/liveness properties (e.g., correct interleaving and absence of race conditions).
  **Suggested Fix:** Introduce a concurrency-aware model (e.g., permissions/ghost tokens for mutex ownership and semaphore signals) or provide a proof that the sequential abstraction is a sound refinement of the concurrent execution model.

- **Location:** `ScoreBoard::complete_dispatch` (exec/spec).
  **Description:** The verification introduces a `completed_cycles` counter and a precondition `completed_cycles < u64::MAX`, which adds a behavior not present in the original code (extra failure condition after many cycles). This is a semantic divergence.
  **Suggested Fix:** Make the counter a ghost field (non-executable) or remove the overflow guard; alternatively, justify the bound as an explicit, documented design constraint in the original implementation.

### Low
- **Location:** `impl Debug for KcallArgs` (original), no verified counterpart.
  **Description:** The formatting behavior is not modeled. While not safety-critical, it is a coverage gap relative to the original source.
  **Suggested Fix:** Add a lightweight spec for formatting output or explicitly scope it out of verification in the module documentation.

## Positive Observations
- The state-machine protocol (Idle → Signaled → Dispatched → Handled → Idle) is precisely specified and proved with clear phase/semaphore invariants.
- Argument/result integrity properties are proven, ensuring the handler sees exactly dispatched args and the dispatcher receives the exact handled result.
- Spec/proof separation is clean, and trust boundaries are clearly documented (mutex/semaphore correctness, global singleton, sequential model).

## Summary
The verification provides a solid sequential state-machine proof for the main success-path protocol, but it omits coverage of the global singleton access and the error/blocked behaviors that are central to the real implementation. Addressing those gaps—especially error handling and the unsafe global access—would substantially improve coverage and equivalence for this kernel component.
