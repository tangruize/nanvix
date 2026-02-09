# Review: process_manager (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** Coverage/mapping notes (process_manager.spec.rs) and missing exec stubs.
  **Description:** Coverage is still incomplete. The verified module only models `ProcessManagerInner`; many original functions in `mod.rs` still lack verified counterparts (e.g., `forge_user_context`, `create_thread`, `try_add_thread`, `sleep`, `wakeup`, `try_wakeup`, `exit`, `exit_thread`, `terminate` wrapper, `check_alarm`, `take_earliest_ready`, `take_running`, `get_running`, `get_running_mut`, `find_process_by_tid`, `find_thread_mut`, and the entire outer `ProcessManager` API). This fails the “all functions verified” criterion.
  **Suggested Fix:** Add explicit verified stubs or wrapper functions for every missing original function (including outer `ProcessManager`), or split the module further so each original function has a corresponding verified model with pre/postconditions.

### Medium
- **Location:** `create_thread_in_ready` + trust-boundary notes (process_manager.rs/spec.rs).
  **Description:** The new stub only models the ready-process case, but the original `create_thread`/`try_add_thread` also moves a *sleeping* process to ready. There is no verified function tying thread creation to the suspended→ready transition, and no modeled error paths.
  **Suggested Fix:** Add verified stubs for the sleeping-case transition (e.g., `create_thread_from_suspended` delegating to `wakeup_to_ready`) and document/encode error paths as Result specs or explicit no-op stubs.

- **Location:** `full_schedule` (process_manager.rs).
  **Description:** The prover added `full_schedule`, but it still does not model `check_alarm` (suspended→interrupted) beyond external `alarm_interrupt` calls, and its postconditions omit the resulting ready set/ready_count. This remains a weak equivalence to the original `schedule()` semantics.
  **Suggested Fix:** Strengthen `full_schedule` postconditions to specify ready set/count, and either verify `check_alarm` directly or add a verified wrapper that sequences alarm transitions internally.

- **Location:** Wakeup path modeling (process_manager.rs).
  **Description:** Only no-op variants for running/ready and the suspended→ready transition are modeled. The error/not-found behavior of `wakeup`/`try_wakeup` (which returns an error without state change) is still not represented.
  **Suggested Fix:** Add verified stubs for the error path (Result with no state change), or explicitly model the full `wakeup` function with pre/postconditions covering both success and failure.

- **Location:** `terminate_ready_stays_ready` (process_manager.rs).
  **Description:** Still an immutable no-op stub; it does not model the thread-level termination effects that justify the `terminate() → resume()` flow in the original code. The spec is therefore too weak for equivalence.
  **Suggested Fix:** Add ghost state for per-process thread liveness or a refined spec capturing the termination intent, even if queue membership is unchanged.

- **Location:** Ready queue ordering (process_manager.spec.rs).
  **Description:** The model still uses a `Set<int>`, so FIFO/earliest-ready scheduling and fairness properties from `take_earliest_ready` are not captured. This leaves a liveness gap.
  **Suggested Fix:** Model the ready queue as an ordered `Seq<int>` or add predicates/lemmas specifying earliest-ready selection.

### Low
- **Location:** IPC accounting (process_manager.spec.rs `number_buffered_messages`).
  **Description:** The counter is not linked to per-process message queues, so message accounting correctness is not verified.
  **Suggested Fix:** Add a ghost model for per-process message counts and relate it to `number_buffered_messages`.

## Positive Observations
- The new `full_schedule` API documents the intended composition and improves the prior schedule mismatch.
- Additional stubs for query/sync operations close part of the coverage gap.
- No `assume`/`external_body` shortcuts were introduced; invariants remain well-formed.

## Summary
The update partially addresses the prior review by adding `full_schedule` and several no-op stubs, but the verification still does not cover all original functions and leaves key behavioral gaps (thread creation, wakeup errors, scheduling order, and termination semantics). Further stubs and stronger specs are needed to satisfy the coverage and equivalence criteria.
