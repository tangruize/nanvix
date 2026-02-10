# Review: process_manager_unsafe (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `wf()` in `process_manager_unsafe.spec.rs` (spec).
  **Description:** The invariant still does not relate `current_tid` to membership in `current_pid`'s thread set. The spec continues to permit states where the running thread is not part of the running process, which is a core scheduler safety property.
  **Suggested Fix:** Extend `ProcessManagerInner` with a ghost PID→thread-set map (or expose an existing one) and add `current_tid ∈ threads(current_pid)` to `wf()`, then update inner transitions accordingly.
- **Location:** `join_thread()` in `process_manager_unsafe.rs` (exec).
  **Description:** Return modeling was added, but the function still returns `Ok(0)` on the wait path even though the real function blocks and does not return until a later iteration. This is not observationally equivalent and can allow proofs that assume a return where the real code suspends. The `outcome` flag remains a nondeterministic single-step model rather than a real loop.
  **Suggested Fix:** Model blocking behavior explicitly (e.g., a two-phase API or a small-step loop relation that does not return on the wait path), and tie the return to a terminal iteration only.
- **Location:** `exit`, `exit_thread`, `sleep`, `giveup`, `try_recv`, `wakeup` in `process_manager_unsafe.rs` (exec).
  **Description:** Core API parameters and Result semantics are still not tied to the model. `status`/`alarm` are not modeled, error propagation is split into separate stubs with no linkage, and `try_recv`/`wakeup` do not model message payloads or errors. This remains a major equivalence gap.
  **Suggested Fix:** Add parameters (including ghost parameters where needed) and model success/error branches within the same functions, relating outputs to inputs and inner transitions.

### Medium
- **Location:** `get_mutex`, `put_mutex_guard`, `get_cond`, `put_cond`, `take_mutex_guard` in `process_manager_unsafe.rs` (exec).
  **Description:** These wrappers are still modeled as read-only operations with no `inner` mutation, but the real code mutates synchronization tables/ownership in `ProcessManagerInner`. This makes the spec stronger than the implementation and hides necessary state changes.
  **Suggested Fix:** Thread `new_inner` through these operations (as with `wakeup`) and require/ensure `new_inner.wf()` so inner mutations are modeled.
- **Location:** `exit_thread()` and `join_thread_wait()` in `process_manager_unsafe.rs` (exec).
  **Description:** The model still does not capture `join_cond.notify_all()` on the exit path, so the connection between exit and join wakeup remains informal. Liveness remains undocumented as a formal predicate.
  **Suggested Fix:** Add a ghost/event predicate or explicit postcondition signaling that the join condition variable is notified on the exit path, and relate it to the join loop model.

### Low
- **Location:** `switch()` in `process_manager_unsafe.rs` (exec).
  **Description:** Performance counters and kernel-idle interrupt enable/wait behavior remain unmodeled. These are observable side effects even if not safety-critical.
  **Suggested Fix:** Add lightweight ghost counters or an external-effect predicate, or explicitly mark these as trusted non-functional side effects.

## Positive Observations
- `join_thread()` now returns a modeled result and documents its loop approximation, which is an improvement over the prior stub.
- The model continues to avoid `assume`/`external_body` and retains the correct stale-atomic PID comparison in `switch()`.
- Divergence handling via `ghost_diverged` remains sound and machine-checked.

## Summary
Some progress was made on join_thread return modeling, but key correctness gaps remain. The PID↔TID membership invariant is still absent, and several core APIs still do not model inputs/results or error propagation, leaving equivalence incomplete. The verification is not yet fully sound or complete.
