# Review: kcall_lock_mutex (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `spec_lock_mutex_safety_preconditions` (spec), `lock_mutex_model` (exec)
  - **Description:** The caller safety predicates are still not required by `lock_mutex_model` and no wrapper function enforces them; the documentation still states they are not added as `requires`. This leaves the unsafe contract unverified at call sites.
  - **Suggested Fix:** Add a verified wrapper with `requires spec_lock_mutex_safety_preconditions(pid, tid)` and prove it at the kcall dispatch layer, or add these predicates directly as preconditions on `lock_mutex_model`.

### Low
- **Location:** `spec_is_valid_error_code` (spec)
  - **Description:** The predicate now accepts any non-zero `int`, which allows negative codes even though the comment states error codes are positive. This weakens the model more than necessary and makes the predicate inconsistent with its documentation.
  - **Suggested Fix:** Strengthen to `code > 0` (or to a predicate aligned with the ErrorCode enum) and keep the documentation consistent.
- **Location:** `mutex_lock_model` (exec) / overall spec
  - **Description:** Liveness/progress behavior (e.g., finite timeout eventually timing out) is still not specified; the model permits any lock outcome consistent with the minimal timeout constraint.
  - **Suggested Fix:** Add liveness assumptions or connect to a mutex-module liveness proof if kcall-level progress guarantees are required.

## Positive Observations
- The previous error-code over-constraint was fixed; the model no longer blocks legitimate timeout-related errors.
- The guard ownership chain remains tied to the mutex address via a guard token that must match the mutex.

## Summary
The error-code restriction issue is fixed, but the unsafe caller preconditions still are not enforced and liveness remains unmodeled. The model is closer to soundness, yet it still omits key usage constraints needed for a complete verification story.
