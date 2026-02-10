# Review: kcall_lock_mutex (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Guard-release predicate is too strong and conflicts with success semantics.**
  - **Location:** `spec_guard_ownership_released` (lock_mutex.spec.rs), `put_mutex_guard_model` and `lock_mutex_model` postconditions (lock_mutex.rs).
  - **Description:** `spec_guard_ownership_released` is documented as “mutex lock released,” yet it is asserted unconditionally for *all* outcomes of `put_mutex_guard_model`, including success. In the real code, a successful `put_mutex_guard` stores the guard and keeps the mutex locked; it does **not** release the lock. This makes the spec stronger than the implementation and can lead to unsound downstream proofs that assume the mutex is unlocked after successful lock.
  - **Suggested Fix:** Refine the predicate to distinguish success vs error (e.g., `spec_guard_transferred_or_released(outcome, addr)`), or move the “lock released” guarantee to the `PutGuardError` path only. Update docs so the predicate does not claim unlock on success.

### Medium
- **Safety preconditions still not enforced in exec model.**
  - **Location:** `spec_lock_mutex_safety_preconditions` (lock_mutex.spec.rs), `lock_mutex_model` (lock_mutex.rs, exec).
  - **Description:** The model still omits `pid`/`tid` and does not add a wrapper or requires clause enforcing `spec_lock_mutex_safety_preconditions`. The unsafe-call contract remains unverified at this module boundary.
  - **Suggested Fix:** Add a `lock_mutex_model_with_context(pid, tid, ...)` wrapper with `requires spec_lock_mutex_safety_preconditions(pid, tid)` and prove it delegates to `lock_mutex_model`, or enforce these preconditions in the verified kcall dispatcher.

### Low
- **Architecture guard still not tied to actual target width.**
  - **Location:** `USIZE_BITS`, `lemma_architecture_guard` (lock_mutex.spec.rs / lock_mutex.proof.rs).
  - **Description:** The guard remains a hardcoded constant; no verification-time check connects it to the real `usize` width.
  - **Suggested Fix:** Link to `size_of::<usize>() * 8` or a Verus-level target-width assertion that fails on non-32-bit targets.

## Positive Observations
- The pipeline and error-propagation proofs remain clear and verify successfully.
- The new guard-related spec hooks show intent to model resource release, even though the current predicate needs refinement.

## Summary
The update adds a guard-release predicate, but it is currently too strong and contradicts success-path semantics, introducing a new soundness risk. The caller safety contract is still not enforced, and the architecture-width guard remains a hardcoded assumption. Verification of the pipeline logic is solid, but the model is not yet fully sound for lock ownership semantics.
