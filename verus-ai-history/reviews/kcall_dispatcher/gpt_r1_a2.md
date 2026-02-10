# Review: kcall_dispatcher (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** remote_dispatch (exec: verus/split/kernel/kcall/dispatcher.rs)
  - **Description:** The remote dispatch path is still an `external_body` that bundles `ScoreBoard::get_mut()`, `scoreboard.dispatch()`, and the sleep-error handling. This is core routing logic, so the behavior for remote kcals (including `SleepError` mapping and get_mut failure) is unverified and only assumed.
  - **Suggested Fix:** Split the scoreboard operations into smaller boundary functions (e.g., `scoreboard_get_mut`, `scoreboard_dispatch`) and reuse `convert_sleepable/handle_sleep_error` in verified code, or add strong postconditions to `remote_dispatch` that match the original semantics.

### Medium
- **Location:** do_kcall_dispatch / convert_sleepable / convert_fallible (exec)
  - **Description:** The verified postconditions are still too weak for many calls. For success cases that should return `ok()` (Recv, MutexLock, CondWait, Sleep, MutexUnlock, SchedulerYield), the model allows arbitrary success values, and error mapping is only captured as `is_success` without tying to specific error codes or `handle_sleep_error` outputs.
  - **Suggested Fix:** Strengthen outcome contracts or per-branch ensures to pin success payloads to 0 where appropriate and to relate error results to the source errors (including the `handle_sleep_error` mapping).
- **Location:** do_kcall_context (exec)
  - **Description:** The top-level verified wrapper does not expose conditional guarantees for GetPid/GetTid (e.g., when pid/tid retrieval succeeds, the result equals those values). Those properties exist in `do_kcall_dispatch` but are not surfaced at the context/ABI boundary, leaving the main contract weaker than the original intent.
  - **Suggested Fix:** Add conditional postconditions to `do_kcall_context` (and possibly `do_kcall`) that relate successful pid/tid retrieval to the returned results.

### Low
- **Location:** lemma_kcall_constants_consistency (proof/spec)
  - **Description:** The constant consistency check is still manual and not mechanically linked to `src/libs/sys/src/sys/number.rs`, so drift in the source enum could still go unnoticed.
  - **Suggested Fix:** Generate spec constants from the source or add a build-time check to ensure the values match the real enum.
- **Location:** do_kcall (exec)
  - **Description:** The ABI entrypoint remains an `external_body` with no verified connection to `do_kcall_context`. This is acceptable for the ABI gap but still a trust assumption.
  - **Suggested Fix:** If possible, add a small verified wrapper that models the ABI conversion and proves it delegates to `do_kcall_context`.

## Positive Observations
- Core dispatcher logic is now implemented and verified in `do_kcall_dispatch`/`do_kcall_context`, addressing the prior unverified match structure.
- The LocalImmediate spec was weakened appropriately to allow pid/tid retrieval failure, aligning with the original source behavior.
- Divergence on `SleepError::Interrupted(Killed)` is modeled explicitly, preventing an unsound return value in that path.

## Summary
The prover fixed the biggest gap by moving the match logic into verified functions and corrected the overly strong LocalImmediate guarantee. However, the remote dispatch path remains an external-body trust boundary, and key value-level behaviors for many calls are still unspecified. Overall, verification coverage is improved but not yet complete or fully sound for the dispatcher’s core semantics.
