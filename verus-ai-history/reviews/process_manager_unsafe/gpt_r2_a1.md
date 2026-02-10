# Review: process_manager_unsafe (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `wf()` in `process_manager_unsafe.spec.rs` (spec).
  **Description:** The invariant does not relate `current_tid` to membership in `current_pid`'s thread set. This allows proofs to proceed even if the running thread does not belong to the running process, which is a core scheduler safety property.
  **Suggested Fix:** Extend `ProcessManagerInner` with a ghost PID→thread-set map (or expose an existing one) and add `current_tid ∈ threads(current_pid)` to `wf()`, then update inner transitions accordingly.
- **Location:** Missing `join_thread` entry point in `process_manager_unsafe.rs` (exec).
  **Description:** The original `join_thread` loop is not modeled as a single verified function; only `join_thread_harvest`, `join_thread_wait`, and `join_thread_error` exist. This fails the coverage requirement and omits verification of the loop control flow and error propagation.
  **Suggested Fix:** Add an exec-level `join_thread` that models the loop with a nondeterministic `try_join_thread` result and ties the three paths together with a `Result<ExitStatus, SleepError>` return.
- **Location:** `exit`, `exit_thread`, `sleep`, `giveup`, `wakeup`, `try_recv_*` in `process_manager_unsafe.rs` (exec).
  **Description:** The verification model drops key inputs and return/error behavior (e.g., `status`, `alarm`, and `Result` error paths). As a result, the model does not prove that these parameters are forwarded to inner state transitions or that error returns preserve state.
  **Suggested Fix:** Add ghost/value parameters for `status`/`alarm`, model the `Result` branches explicitly (including borrow/schedule failures), and relate successful inner transitions to those inputs.

### Medium
- **Location:** `get_mutex`, `put_mutex_guard`, `get_cond`, `put_cond`, `take_mutex_guard` in `process_manager_unsafe.rs` (exec).
  **Description:** These functions are modeled as pure read-only operations that leave `inner` unchanged, but the real code mutates synchronization tables/ownership in `ProcessManagerInner`. This makes the spec stronger than the implementation and can mask state changes needed by callers.
  **Suggested Fix:** Thread an updated `new_inner` through these wrappers (as is done for `wakeup`) and require/ensure `new_inner.wf()` so inner mutations are modeled.
- **Location:** `exit_thread`/`join_thread_wait` in `process_manager_unsafe.rs` (exec).
  **Description:** The model does not capture `join_cond.notify_all()` in `exit_thread`, so the verification cannot connect exit to the eventual wakeup required for `join_thread` termination. Liveness is only documented, not established.
  **Suggested Fix:** Add a ghost/event predicate or explicit postcondition that the join condition variable is notified on the exit path, and relate it to the join loop model.

### Low
- **Location:** `switch` in `process_manager_unsafe.rs` (exec).
  **Description:** Performance counters and kernel-idle interrupt enable/wait behavior are not modeled. This is likely non-functional but still part of the observable behavior.
  **Suggested Fix:** Add lightweight ghost counters or an external-effect predicate to record these side effects, or explicitly mark them as trusted non-functional behavior in the spec.

## Positive Observations
- The model clearly documents trust boundaries (T5–T13) and avoids `assume`/`external_body` in this module.
- `switch()` accurately models the stale-atomic PID comparison and quantum reset, which is the most subtle control-flow detail in the original code.
- Divergence of `exit`/`exit_thread` is handled via `ghost_diverged`, preventing post-exit reasoning in a machine-checked way.

## Summary
The verification captures the core atomic/inner-state transition logic for context switching and quantum management, but it omits several essential correctness properties and full-function coverage. The largest gaps are missing `join_thread` modeling, lack of PID↔TID membership invariants, and dropped parameters/error behavior for key APIs. Addressing these would materially strengthen equivalence and make the proofs more faithful to the kernel’s semantics.
