# Review: process_manager_unsafe (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `wf()` in `process_manager_unsafe.spec.rs` (spec).
  **Description:** The invariant still does not relate `current_tid` to membership in `current_pid`'s thread set. This permits states where the running thread is not part of the running process, which is a core scheduler safety property.
  **Suggested Fix:** Extend `ProcessManagerInner` with a ghost PID→thread-set map (or expose an existing one) and add `current_tid ∈ threads(current_pid)` to `wf()`, then update inner transitions accordingly.
- **Location:** `join_thread()` in `process_manager_unsafe.rs` (exec).
  **Description:** A unified entry point was added, but it still does not model the `Result<ExitStatus, SleepError>` return or the loop semantics; it only models one iteration with an `outcome` flag. This is too weak to establish equivalence with the original function’s observable behavior.
  **Suggested Fix:** Model the return value explicitly (including `ExitStatus` on harvest and `SleepError` on error), and either encode a loop invariant or provide a small-step loop model that ties iterations together.
- **Location:** `exit`, `exit_thread`, `sleep`, `giveup`, `try_recv`, `wakeup` in `process_manager_unsafe.rs` (exec).
  **Description:** Parameters and `Result` behavior are still not connected to the model. `exit/exit_thread` ignore `status`, `sleep` ignores `alarm` and the interrupt reason, `giveup`/`sleep`/`exit` error paths are separate stubs with no linkage, and `try_recv` lacks error/message payload modeling. This leaves key API semantics unverified.
  **Suggested Fix:** Add parameters (including ghost parameters where needed) and model success/error branches within the same function signatures, relating output values to the inputs and inner transitions.

### Medium
- **Location:** `get_mutex`, `put_mutex_guard`, `get_cond`, `put_cond`, `take_mutex_guard` in `process_manager_unsafe.rs` (exec).
  **Description:** These wrappers are still modeled as read-only operations with no `inner` mutation, but the real code mutates synchronization tables/ownership in `ProcessManagerInner`. This makes the spec stronger than the implementation and masks state changes required by callers.
  **Suggested Fix:** Thread `new_inner` through these operations (as with `wakeup`) and require/ensure `new_inner.wf()` so inner mutations are modeled.
- **Location:** `exit_thread()` and `join_thread_wait()` in `process_manager_unsafe.rs` (exec).
  **Description:** The model still does not capture `join_cond.notify_all()` on the exit path, so the connection between exit and join wakeup remains only informal. Liveness is documented but not tied to a postcondition or event predicate.
  **Suggested Fix:** Add a ghost/event predicate or explicit postcondition signaling that the join condition variable is notified on the exit path, and relate it to the join loop model.

### Low
- **Location:** `switch()` in `process_manager_unsafe.rs` (exec).
  **Description:** Performance counters and kernel-idle interrupt enable/wait behavior remain unmodeled. These are side effects visible to the system, even if not safety-critical.
  **Suggested Fix:** Add lightweight ghost counters or an external-effect predicate, or explicitly mark these as trusted non-functional side effects.

## Positive Observations
- A unified `join_thread()` entry point was added and the error-path stubs for `sleep/exit/exit_thread` now exist, which is a step toward full coverage.
- The model still avoids `assume`/`external_body` and retains the correct stale-atomic PID comparison in `switch()`.
- Divergence handling via `ghost_diverged` remains sound and machine-checked.

## Summary
Several earlier issues remain unresolved. The key gaps are the missing PID↔TID membership invariant and the still-weak modeling of results/parameters for core APIs, including `join_thread`’s return value. Verification is improved but not yet complete or fully equivalent to the original code.
