# Review: kcall_scoreboard (gpt-5.2-codex)

## Grade: C+

## Issues Found

### Critical
- None.

### High
- **Location:** `ScoreBoard::handle` / sequential model assumptions (exec, `verus/split/kernel/kcall/scoreboard.rs`)
  - **Description:** The verification still assumes sequential execution and uses `&mut self` for `handle()`, while the original uses `&self` with atomic `try_down()` and allows concurrency. The trust-boundary text acknowledges this, but no proof connects the sequential model to the concurrent behavior, so the core concurrency assumption remains unverified.
  - **Suggested Fix:** Provide a refinement/linearizability argument tying the sequential model to the verified semaphore/mutex behavior, or model `handle()` with atomic `try_down()` and allow concurrent access in the proof.

- **Location:** `ScoreBoard::dispatch` wrapper (exec, `verus/split/kernel/kcall/scoreboard.rs`)
  - **Description:** The new `dispatch()` still does not match the original API: it introduces extra parameters (`ret`, `lock_acquired`, `down_interrupted`) and therefore does not model the actual control-flow of the original `dispatch()` call. This weakens equivalence, because the returned `DispatchOutcome` is no longer solely a function of the scoreboard and the handler’s behavior, and error cases from the real code are not fully represented.
  - **Suggested Fix:** Provide a wrapper with the original signature and model environment nondeterminism via spec-level choices (or ghost parameters) while proving refinement to the original error semantics.

### Medium
- **Location:** `ScoreBoard::dispatch` error path (`spec_dispatch_interrupted` + exec `dispatch` ensures) (spec/exec, `verus/split/kernel/kcall/scoreboard.spec.rs`, `scoreboard.rs`)
  - **Description:** The interrupted path is modeled only after the handler has completed (phase `Handled`) and the exec postconditions do not require argument/result preservation or semaphore state. In the real code, `handled.down()` can be interrupted before the handler signals, leaving the board in Signaled/Dispatched states; the current spec is too strong and the ensures are too weak to capture this.
  - **Suggested Fix:** Model interruption as `spec_abandon_dispatch` from any active phase and require `self@ == spec_abandon_dispatch(old(self)@)` (including args/result preservation) for error outcomes.

- **Location:** `Semaphore::up()` failure handling (`begin_dispatch`, `handled`, proof lemmas) (exec/proof, `verus/split/kernel/kcall/scoreboard.rs`, `scoreboard.proof.rs`)
  - **Description:** The proof still assumes `up()` cannot fail based solely on the semaphore value, but the real implementation can fail via `notify_first()`/`ProcessManager::wakeup()`. This means the spec is stronger than the implementation, and dispatch semantics for `SleepError::Generic` are still not modeled.
  - **Suggested Fix:** Model `up()` as potentially fallible in the scoreboard spec, or prove a stronger semaphore invariant showing wakeup cannot fail in this context and thread it into the scoreboard preconditions.

- **Location:** `ScoreBoardSlot::get_mut()` modeling (exec/spec, `verus/split/kernel/kcall/scoreboard.rs`)
  - **Description:** The verified model still only provides `try_get_board()` (bool) and an immutable `get_board()`. This omits the `Result` error semantics and the mutable reference return of the original `get_mut()`, so coverage/equivalence remains incomplete.
  - **Suggested Fix:** Add a verified `get_board_mut()`/`get_mut()` wrapper returning a `Result` matching `ErrorCode::TryAgain`, or prove a refinement mapping from the slot API to the original function.

### Low
- **Location:** `impl Debug for KcallArgs`, `pub fn init()` (original `src/kernel/src/kcall/mod.rs`)
  - **Description:** These remain out of scope, so strict coverage of all functions is still violated.
  - **Suggested Fix:** Add trivial spec stubs or documented wrappers to explicitly cover these functions.

- **Location:** Module header/doc comments (exec, `verus/split/kernel/kcall/scoreboard.rs`)
  - **Description:** The docs claim `get_board_mut()` exists, but no such function is implemented, which is misleading.
  - **Suggested Fix:** Either implement `get_board_mut()` or remove the reference from the documentation.

## Positive Observations
- A `dispatch()` wrapper and new spec lemmas were added, which is a step toward modeling the original API.
- The phase-based protocol, integrity properties, and split spec/proof structure remain clear and consistent.

## Summary
Several prior issues remain, especially the unproven concurrency refinement and the incomplete equivalence of `dispatch()` and `get_mut()`. The new wrapper is helpful but still does not reflect the real API or all error behaviors. The verification is improved but not yet complete or fully sound for the kernel’s concurrent setting.
