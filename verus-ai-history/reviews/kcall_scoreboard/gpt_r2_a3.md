# Review: kcall_scoreboard (gpt-5.2-codex)

## Grade: C+

## Issues Found

### Critical
- None.

### High
- **Location:** `ScoreBoard::handle` / sequential model assumptions (exec, `verus/split/kernel/kcall/scoreboard.rs`)
  - **Description:** The model is still sequential and `handle()` still requires `&mut self`, while the original uses `&self` with atomic `try_down()` and allows concurrency. The trust-boundary text acknowledges this but there is still no refinement proof that the sequential model covers the concurrent interleavings used in production.
  - **Suggested Fix:** Provide a refinement/linearizability argument tying the sequential model to the verified semaphore/mutex behavior, or model `handle()` with atomic `try_down()` and concurrent access in the proof.

- **Location:** `ScoreBoard::dispatch` wrapper signature (exec, `verus/split/kernel/kcall/scoreboard.rs`)
  - **Description:** The wrapper still does not match the original API; it takes extra runtime parameters (`ret`, `lock_acquired`, `down_interrupted`). This means it is not a faithful model of the real call and its observable outcomes are not solely determined by the scoreboard and handler execution, weakening equivalence.
  - **Suggested Fix:** Provide a wrapper with the original signature and model environmental nondeterminism via spec-level choices/ghost parameters, then prove refinement to the original error semantics.

### Medium
- **Location:** `spec_dispatch_interrupted` / `dispatch()` interruption path (spec/exec, `verus/split/kernel/kcall/scoreboard.spec.rs`, `scoreboard.rs`)
  - **Description:** The interrupted path was improved (data and semaphore state preserved), but it still models interruption only from the Signaled phase. In reality, `handled.down()` can be interrupted in Signaled, Dispatched, or Handled states, so the spec remains an under-approximation of actual behavior.
  - **Suggested Fix:** Model interruption as `spec_abandon_dispatch` from an arbitrary active phase and require the exec postconditions to match that nondeterministic phase.

- **Location:** `Semaphore::up()` failure handling (`begin_dispatch`, `handled`, proof lemmas) (exec/proof, `verus/split/kernel/kcall/scoreboard.rs`, `scoreboard.proof.rs`)
  - **Description:** The proofs still assume `up()` cannot fail solely because the semaphore value is 0, but the real `up()` can fail via `notify_first()`/`ProcessManager::wakeup()`. The `SleepError::Generic` path for `dispatched.up()` is still not modeled.
  - **Suggested Fix:** Model `up()` as potentially fallible (and propagate a distinct error), or prove a stronger invariant that wakeups cannot fail in this context and include it in preconditions.

- **Location:** `ScoreBoardSlot::get_mut()` modeling (exec/spec, `verus/split/kernel/kcall/scoreboard.rs`)
  - **Description:** The model still only provides `try_get_board()` (bool) and an immutable `get_board()`. This omits the `Result` error semantics and the mutable reference return of the original `get_mut()`, so coverage/equivalence remains incomplete.
  - **Suggested Fix:** Add a verified `get_board_mut()`/`get_mut()` wrapper returning a `Result` matching `ErrorCode::TryAgain`, or prove a refinement mapping from the slot API to the original function.

### Low
- **Location:** `impl Debug for KcallArgs`, `pub fn init()` (original `src/kernel/src/kcall/mod.rs`)
  - **Description:** These remain out of scope, so strict coverage of all functions is still violated.
  - **Suggested Fix:** Add trivial spec stubs or documented wrappers to explicitly cover these functions.

## Positive Observations
- The interrupted-dispatch spec now preserves args/result and semaphore state, which is a real improvement.
- The state-machine, integrity, and proof structure remain clean and consistent with prior revisions.

## Summary
Some previous issues were partially addressed, but key gaps remain: concurrency refinement, API-equivalence for `dispatch()` and `get_mut()`, and modeling of `up()` failures. The verification is still not complete or fully sound relative to the original concurrent kernel code.
