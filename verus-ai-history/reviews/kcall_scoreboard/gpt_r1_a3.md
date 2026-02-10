# Review: kcall_scoreboard (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `ScoreBoard::begin_dispatch()` / `ScoreBoard::handled()` (exec/spec) and proof lemmas `lemma_semaphore_up_dispatched_cannot_fail`, `lemma_semaphore_up_handled_cannot_fail`.
  **Description:** The model still assumes `Semaphore::up()` cannot fail and justifies it solely by the semaphore count. In the real code, `Semaphore::up()` can fail via `Condvar::notify_first()` (which calls `ProcessManager::wakeup()` and returns `Error`). This error path is not modeled, so the verification is unsound for failure cases of `dispatched.up()` and `handled.up()`.
  **Suggested Fix:** Model the `up()` error path explicitly (returning `Result`) and prove state preservation on failure, or prove (with verified properties of `Condvar`/`ProcessManager`) that `notify_first()` cannot fail in these contexts.

### Medium
- **Location:** `ScoreBoardSlot::get_board()` / `try_get_board()` (exec/spec).
  **Description:** The global accessor still does not provide mutable access equivalent to `get_mut()`. `get_board()` returns `&ScoreBoard`, but all operational methods (`begin_dispatch`, `handle`, `handled`, `complete_dispatch`, `abandon_dispatch`) require `&mut self`, so the model does not connect successful `get_mut()` with actual state transitions on the global instance.
  **Suggested Fix:** Add a `get_board_mut(&mut self) -> &mut ScoreBoard` (or a method that performs dispatch/handle on the slot directly) with appropriate pre/postconditions to model `get_mut()` precisely.

- **Location:** Sequential abstraction (Trust Boundary T4).
  **Description:** The proof still relies on a sequential `&mut self` state machine without a formal refinement argument to the concurrent behavior enforced by mutex + semaphore in the original. The trust boundary is documented but not justified.
  **Suggested Fix:** Provide a refinement proof or a permissions-based concurrency model showing that the sequential transitions are a sound abstraction of the concurrent protocol.

### Low
- **Location:** `impl Debug for KcallArgs` (original `kcall/mod.rs`).
  **Description:** Formatting behavior remains out of scope and unverified. This is a coverage gap but not safety-critical.
  **Suggested Fix:** Add a lightweight spec or explicitly exclude it in the verification scope statement (if not already).

## Positive Observations
- Re-initialization of the global slot is now modeled to match the original behavior.
- Lock acquisition failure (`SleepError::Interrupted`) and `handled.down()` interruption are explicitly modeled via `try_begin_dispatch()` and `abandon_dispatch()`.
- The ghost cycle counter remains non-semantic, avoiding the previous divergence.

## Summary
The update fixes the re-init and some error-path modeling, but the unmodeled `Semaphore::up()` failure remains a significant soundness gap, and global mutable access is still not faithfully represented. The verification is improved but still incomplete and not fully equivalent to the original concurrent implementation.
