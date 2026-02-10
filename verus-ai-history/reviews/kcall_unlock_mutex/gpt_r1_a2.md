# Review: kcall_unlock_mutex (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `take_mutex_guard_model` / `unlock_mutex_model` error postconditions (exec, `verus/split/kernel/pm/kcall/unlock_mutex.rs`), `spec_mutex_may_be_unlocked_on_error` (spec).
  **Description:** The previous mismatch about error-after-guard was addressed by adding `spec_mutex_may_be_unlocked_on_error`, but this predicate is unconstrained and always asserted on any error. This does not precisely capture which error paths actually unlock the mutex and leaves the equivalence to the PM implementation ambiguous (clients could accidentally treat it as a concrete state fact).
  **Suggested Fix:** Model the PM error path with a ghost flag (e.g., `GuardExtractedBeforeError`) or split error outcomes so the spec can state exactly when the mutex is unlocked. Alternatively, prove a PM invariant that `put_mutex` cannot fail after guard extraction and replace the “may” predicate with a concrete postcondition.

### Low
- **Location:** `spec_is_valid_error_code` (spec, `verus/split/kernel/pm/kcall/unlock_mutex.spec.rs`).
  **Description:** The predicate still accepts any positive integer as a valid error code, which is weaker than the actual `ErrorCode` enum and permits invalid values.
  **Suggested Fix:** Tie the predicate to the concrete `ErrorCode` set (reuse the error library’s Verus spec predicate or define a membership predicate over all variants).
- **Location:** `spec_guard_dropped_and_mutex_unlocked` / documentation (spec/exec).
  **Description:** The liveness/notification effect noted in the original code comment (threads being notified when the guard is dropped) remains out of scope and is not captured in any spec predicate.
  **Suggested Fix:** Add a notification/wakeup predicate (or link to the mutex module’s proof) to reflect this observable behavior, or explicitly justify why it is excluded from the kcall’s correctness contract.

## Positive Observations
- The success-path ownership property is now propagated to the kcall boundary (`spec_thread_owns_mutex` on success), fixing a previous weakness.
- The error-path narrative now acknowledges PM-internal guard drops, avoiding the earlier incorrect “no unlock on error” assumption.
- Exec/spec/proof separation remains clean and well documented.

## Summary
The rework improves the success-path guarantees and avoids the earlier overly strong error-path assumption. However, the new “may be unlocked” predicate is too imprecise to fully justify equivalence with PM error behavior, and low-level spec weaknesses (error-code validity and notification effects) remain. Tightening the error-path modeling would be the next step to reach an A-grade verification.
