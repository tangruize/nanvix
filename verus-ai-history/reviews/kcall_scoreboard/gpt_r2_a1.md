# Review: kcall_scoreboard (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `ScoreBoard::handle` / `ScoreBoard::begin_dispatch` (exec, `verus/split/kernel/kcall/scoreboard.rs`)
  - **Description:** The verified model enforces sequential execution by taking `&mut self` for `handle()` and by treating the protocol as a single-threaded state machine. The original implementation allows `handle(&self)` to run concurrently with a dispatcher holding the mutex (the semaphore’s `try_down()` is atomic), so the proof does not cover the key concurrent interleavings it relies on in production.
  - **Suggested Fix:** Add a refinement argument that connects the sequential model to the concurrent implementation using the verified semaphore/mutex specs, or model `handle()` with atomic `try_down()` and allow concurrent access in the spec/proof (e.g., via a rely/guarantee or linearizability-style lemma).

- **Location:** `ScoreBoard::dispatch` mapping (`ScoreBoard::try_begin_dispatch`, `complete_dispatch`, `abandon_dispatch`) (exec/spec, `verus/split/kernel/kcall/scoreboard.rs` and `scoreboard.spec.rs`)
  - **Description:** The original `dispatch()` returns `Result<KcallResult, SleepError>` and can fail on lock acquisition and on `handled.down()`. The verified model splits this into multiple steps and does not provide a verified `dispatch()` wrapper that returns the same observable result/error behavior. The `abandon_dispatch()` path is modeled, but it is not tied to a returned error, so the API-level semantics are not proven equivalent.
  - **Suggested Fix:** Add a verified `dispatch()` wrapper that composes `try_begin_dispatch` + `complete_dispatch`/`abandon_dispatch` and returns a `Result` aligned with the original `SleepError` cases, or explicitly prove the behavioral equivalence of the split API to the original function.

### Medium
- **Location:** `lemma_semaphore_up_dispatched_cannot_fail` / `lemma_semaphore_up_handled_cannot_fail` (proof, `verus/split/kernel/kcall/scoreboard.proof.rs`)
  - **Description:** The verification assumes `Semaphore::up()` cannot fail because the semaphore value is 0, but the actual implementation can still fail via `notify_first()` if `ProcessManager::wakeup()` returns an error. This makes the specification too strong unless an additional invariant guarantees wakeup infallibility for this usage.
  - **Suggested Fix:** Model `up()` as potentially fallible in the scoreboard spec, or add/verify a precondition/invariant showing that `ProcessManager::wakeup()` cannot fail for these threads.

- **Location:** `ScoreBoardSlot::try_get_board` / `get_board` (exec/spec, `verus/split/kernel/kcall/scoreboard.rs`)
  - **Description:** The original `ScoreBoard::get_mut()` returns `Result<&'static mut ScoreBoard, Error>`, while the verified model only provides a boolean `try_get_board()` plus an immutable `get_board()`. This omits the `Result`-level error semantics and the mutable reference return, so coverage/equivalence is incomplete.
  - **Suggested Fix:** Add a verified `get_board_mut()` (or `get_mut()` wrapper) that returns a `Result` matching `ErrorCode::TryAgain`, or prove that the current API is a refinement of the original access pattern.

### Low
- **Location:** `impl Debug for KcallArgs`, `pub fn init()` (original `src/kernel/src/kcall/mod.rs`)
  - **Description:** These functions are explicitly out of scope in the verified model, which violates the strict coverage requirement for “all functions.”
  - **Suggested Fix:** Add trivial spec stubs (or documented wrappers) to acknowledge these functions, even if their behavior is marked as non-critical/out-of-scope.

## Positive Observations
- The phase-based state machine (`Idle → Signaled → Dispatched → Handled → Idle`) is clearly specified and consistently reflected in exec/spec/proof.
- Argument/result integrity and semaphore signal/consume ordering are explicitly proven, which matches the core safety properties of the protocol.
- The spec/proof separation is clean, and the trust boundaries are documented clearly in the exec module header.

## Summary
The verification captures the sequential protocol and data integrity well, but it falls short on concurrency equivalence and API-level error semantics. Addressing the `dispatch()` error modeling and the concurrent `handle()` behavior would significantly strengthen the correspondence to the production code. Overall, the split is well-structured, but the remaining gaps are important for OS-kernel correctness.
