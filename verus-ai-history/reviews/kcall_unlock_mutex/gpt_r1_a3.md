# Review: kcall_unlock_mutex (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `take_mutex_guard_model` ghost flag (`pm_internally_dropped_guard`) and `unlock_mutex_model` error postconditions (exec).
  **Description:** The new ghost flag refines the previous “may unlock” predicate, but it remains unconstrained on error: there is no postcondition tying `pm_internally_dropped_guard` to specific error reasons or to whether the guard was actually extracted. This leaves the equivalence with the PM implementation still partially unproven and allows inconsistent states (e.g., error with flag true even when ownership check failed).
  **Suggested Fix:** Split the error outcome into distinct variants (e.g., `ErrorBeforeGuard`, `ErrorAfterGuard`) or add a ghost predicate that links the flag to a concrete precondition (like `spec_thread_owns_mutex`) so the PM module can prove exactly when the flag is true.

### Low
- **Location:** `spec_is_valid_error_code` (spec).
  **Description:** The predicate still accepts any positive integer as a valid error code, which is weaker than the actual `ErrorCode` enum.
  **Suggested Fix:** Replace it with a membership predicate over real `ErrorCode` variants or reuse the error library’s Verus spec.
- **Location:** Liveness/notification effect (spec/exec documentation).
  **Description:** The observable thread-notification effect of dropping the guard remains out of scope; no spec predicate captures it.
  **Suggested Fix:** Add or reference a predicate in the mutex module capturing wakeup/notification, or explicitly state why the kcall contract excludes it.

## Positive Observations
- The ghost-flag refactor makes error-path reasoning more explicit and avoids the earlier overly weak “may unlock” predicate.
- Success-path ownership and guard-drop properties are still properly asserted at the kcall boundary.
- Exec/spec/proof separation remains clean, with good documentation of trust boundaries.

## Summary
The prover improved the error-path modeling by introducing an explicit ghost flag, but without a constraint linking that flag to concrete PM error conditions the specification remains too permissive for full equivalence. Low-level spec weaknesses (error-code validity and notification semantics) also remain. Further tightening of the error-path contract would move this toward a complete and sound verification.
