# Review: kcall_unlock_mutex (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `unlock_mutex_model` preconditions (exec: `verus/split/kernel/pm/kcall/unlock_mutex.rs`).
  **Description:** The model requires `spec_is_currently_running(pid, tid)`, but the original `unlock_mutex` does not enforce or use `pid/tid` for behavior (they are only used for logging in `take_mutex_guard`). This strengthens the contract and makes equivalence depend on an external dispatcher proof that the passed ids are always the running thread.
  **Suggested Fix:** Either (a) move this requirement to the verified kcall dispatch layer and reference that proof here, or (b) drop the precondition and avoid using `pid/tid` to establish `spec_thread_owns_mutex` in this module.

- **Location:** `take_mutex_guard_model` postconditions (exec: `verus/split/kernel/pm/kcall/unlock_mutex.rs`).
  **Description:** On error with `pm_internally_dropped_guard == false`, the model leaves the mutex state unconstrained. In the concrete PM implementation, these error paths occur before guard extraction and should *not* unlock the mutex. The current spec is weaker than the real behavior and could hide regressions.
  **Suggested Fix:** Add an explicit postcondition tying `pm_internally_dropped_guard == false` on error to a predicate that the mutex state is unchanged (or at least not unlocked), and/or relate the flag to the specific PM error sources.

### Low
- **Location:** `spec_is_valid_error_code` (spec: `verus/split/kernel/pm/kcall/unlock_mutex.spec.rs`).
  **Description:** Valid error codes are modeled as `code > 0`, which admits positive values outside the `ErrorCode` enum. This is weaker than the real API (only known errno constants are possible).
  **Suggested Fix:** Define a predicate that matches the `ErrorCode` enumeration (or reuse one from the error module) and require `error_code` to be in that set.

- **Location:** Liveness/notification effects (spec/proof: `unlock_mutex.spec.rs`, `unlock_mutex.proof.rs`).
  **Description:** The original code comment notes that dropping the guard notifies waiting threads; this liveness effect is out of scope in the current model.
  **Suggested Fix:** If liveness is part of the intended contract, add an abstract predicate (e.g., `spec_waiters_notified`) and connect it to `drop_guard_model`, or link to a proved property in the mutex module.

## Positive Observations
- The exec model mirrors the original control flow closely (take guard, drop guard, propagate error) and proves result mapping correctness.
- The model explicitly captures the PM internal “drop on error” path via a ghost flag, which is a good fidelity improvement over a generic “may unlock” predicate.
- Spec/proof separation is clean, and the key functional properties (error propagation, success requires take_guard Ok, guard token chain) are proven.

## Summary
The verification is solid and well-structured, with a faithful control-flow model and good separation of exec/spec/proof. The main gaps are a strengthened `pid/tid` precondition that relies on external dispatcher guarantees and a too-weak description of mutex state on error paths. Tightening these two points and strengthening error-code validity would bring the model closer to full behavioral equivalence.
