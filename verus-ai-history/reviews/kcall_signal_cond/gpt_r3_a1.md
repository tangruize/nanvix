# Review: kcall_signal_cond (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `spec_put_cond_completed`, `spec_cond_ref_released`, `spec_condvar_acquired` (spec) and external bodies in `signal_cond.rs`.
  **Description:** The verification treats PM/condvar state effects as uninterpreted tokens, so it does not establish concrete invariants about condvar map ownership or reference counts. This makes the proof too weak to rule out resource leaks or stale entries in the PM map, which are key safety properties for a kernel kcall.
  **Suggested Fix:** Link these predicates to PM/condvar state models (or import their invariants) and prove that get/drop/put update those invariants; alternatively, add a cross-module lemma that `spec_put_cond_completed` implies the condvar slot is reclaimed or otherwise tracked.

### Low
- **Location:** `lemma_notify_error_skips_put_cond` (proof) / `signal_cond_model` (exec).
  **Description:** The model explicitly mirrors the current behavior where `notify` errors skip `put_cond`, but it does not prove that skipping `put_cond` is safe (e.g., eventual reclamation). If cleanup on error is required, this is a correctness gap that is currently only documented.
  **Suggested Fix:** Either change the implementation to call `put_cond` on notify error and update the model, or add a PM-level invariant proving eventual reclamation on error paths.

## Positive Observations
- Coverage is complete: the only function in the original source (`signal_cond`) has a verified exec model with corresponding spec and proof files.
- The spec accurately models the short-circuit pipeline and preserves error codes and awakened counts on success.
- Broadcast semantics are captured with a precise success-path bound and are propagated from the notify trust boundary to the final result.
- The split between exec/spec/proof is clean and well-documented, with explicit trust boundaries.

## Summary
The verification is structurally sound and closely mirrors the original control flow, with strong guarantees about error propagation and broadcast behavior. The main gap is that key resource-management invariants are abstracted away, so the proof cannot rule out PM/condvar state leaks or stale entries, especially on notify-error paths. Strengthening the PM/condvar invariants or proving eventual reclamation would close the remaining safety gap.
