# Review: kcall_lock_mutex (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `spec_is_valid_error_code` (spec), `mutex_lock_model`/`get_mutex_model`/`put_mutex_guard_model` contracts (exec)
  - **Description:** The new error-code predicate restricts all generic errors to a 5-code subset. However, the real `Mutex::lock` path can return `SleepError::Generic(ErrorCode::OperationTimedOut)` when a timeout is already expired in `Condvar::wait`, and `ErrorCode` in the kernel has many more variants. This makes the model too strong and breaks semantic equivalence for legitimate error paths.
  - **Suggested Fix:** Widen `spec_is_valid_error_code` to include all error codes that can arise from `get_mutex`, `put_mutex_guard`, and `Mutex::lock` (at least `OperationTimedOut`), or model the full `ErrorCode` enum in the Verus split and constrain against that.

### Medium
- **Location:** `spec_lock_mutex_safety_preconditions` (spec), `lock_mutex_model` (exec)
  - **Description:** The caller safety predicates are still not required by `lock_mutex_model` and no wrapper function enforces them. The docs explicitly say they are “not added as requires,” so the unsafe contract remains unverified at call sites.
  - **Suggested Fix:** Add these predicates as `requires` on a wrapper model used by kcall dispatch, or prove them in the PM module and thread the proof into `lock_mutex_model`.

### Low
- **Location:** `mutex_lock_model` (exec) / overall spec
  - **Description:** Liveness/progress behavior (e.g., finite timeout eventually timing out) is still not specified; the model permits any lock outcome consistent with the minimal timeout constraint.
  - **Suggested Fix:** Add liveness assumptions or reference a separate mutex-module liveness proof if required for kcall-level guarantees.

## Positive Observations
- The guard ownership chain is now tied to the mutex address via `Ghost<Option<u32>>`, which fixes the previous ownership-token weakness.
- The pipeline modeling and short-circuit proofs remain clean and aligned with the original control flow.

## Summary
The guard-token fix is real and improves ownership soundness, but the new error-code restriction is too strong and invalidates equivalence for legitimate timeout errors. The unsafe caller preconditions are still not enforced. Overall, the verification is closer to complete but still has important soundness gaps.
