# Review: kcall_scoreboard (gpt-5.2-codex)

## Grade: C+

## Issues Found

### Critical
- None.

### High
- **`UpFailed` state modeling is inconsistent with real `dispatched.up()` behavior** (exec: `ScoreBoard::dispatch`, spec: `spec_dispatch_up_failed`).
  - **Description:** In the original `Semaphore::up()`, the counter is incremented *before* `notify_first()`. If `notify_first()` fails, `dispatch()` returns `SleepError::Generic` but the semaphore value has still increased to 1. The updated model returns to a clean Idle state with `dispatched_value == 0` and no pending signal, which is not equivalent to the implementation. The real state is a stuck Signaled-like state with lock dropped and a pending signal.
  - **Suggested Fix:** Model `UpFailed` as a stuck state equivalent to `spec_abandon_dispatch` from the Signaled phase (args written, dispatched_value=1, lock released), or otherwise capture the incremented semaphore value and possible handler wakeup.

- **Generic error paths still missing for `lock()`/`handled.down()` and `handled.up()`** (exec/spec).
  - **Description:** The model now includes `UpFailed` for `dispatched.up()` but still does not represent `SleepError::Generic` from `lock()` or `handled.down()`, and continues to treat `handled.up()` as infallible even though it can return `Error` via `notify_first()`. These are observable behaviors in the original code and remain unmodeled.
  - **Suggested Fix:** Add explicit outcomes/specs for generic failures from `lock()`/`down()` and for `handled.up()` failure, with corresponding state characterizations.

### Medium
- **Concurrency/refinement gap for `handle()` remains** (exec: `handle(&mut self)`, docs T4).
  - **Description:** The verified model still assumes exclusive access for `handle()`, while the real implementation uses `handle(&self)` with atomic `try_down()`. This remains an assumption rather than a proven refinement.
  - **Suggested Fix:** Provide a stronger refinement argument or model `handle()` with shared access and atomic semantics.

### Low
- **Coverage still excludes `impl Debug for KcallArgs` and module `init()`** (docs: Verification Scope).
  - **Description:** These functions remain out of scope, so coverage does not fully meet the “all functions covered” criterion.
  - **Suggested Fix:** Add trivial verified stubs or formalize a coverage exemption policy.

- **Liveness remains unproven (explicitly out of scope)** (docs: Verification Scope).
  - **Description:** Progress properties are still not established; only safety is proved.
  - **Suggested Fix:** Add progress assumptions or minimal liveness lemmas for non-interrupted cycles.

## Positive Observations
- Down-interruption modeling now covers all active phases via `handler_progress`, fixing the prior narrow Signaled-only model.
- Documentation of error-handling paths is clearer and the new `UpFailed` outcome addresses part of the generic error gap.

## Summary
The update fixes the down-interruption phase coverage and adds an `UpFailed` outcome, but the new `UpFailed` state is not semantically accurate and several generic error paths remain unmodeled. The verification is still safety-focused with a concurrency refinement gap and incomplete coverage.
