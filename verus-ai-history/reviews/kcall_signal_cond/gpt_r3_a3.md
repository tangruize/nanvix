# Review: kcall_signal_cond (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `spec_put_cond_completed`, `spec_cond_ref_released`, `spec_condvar_acquired` (spec) and external bodies in `signal_cond.rs`.
  **Description:** These predicates remain uninterpreted abstract tokens, and there is still no evidence of concrete PM/condvar state invariants (e.g., map ownership or refcount changes). The proof therefore cannot rule out stale entries or resource leaks, which are key kernel safety properties.
  **Suggested Fix:** Link these predicates to concrete PM/condvar state models or import their invariants, and prove that get/drop/put maintain those invariants (or add a cross-module lemma stating what `spec_put_cond_completed` guarantees).

### Low
- **Location:** `lemma_notify_error_skips_put_cond` (proof) / `signal_cond_model` (exec).
  **Description:** The notify-error path still skips `put_cond` and there is no proof that this is safe (eventual reclamation or non-leak). The issue is documented but not resolved.
  **Suggested Fix:** Either change the implementation to call `put_cond` on notify error and update the model, or add a PM-level invariant showing eventual reclamation on error paths.

## Positive Observations
- Coverage remains complete with matching exec/spec/proof artifacts.
- Error propagation and broadcast semantics are still precisely modeled and proven.
- The exec/spec/proof split remains clean and well-documented.

## Summary
The previous issues were not actually fixed: resource-management predicates are still abstract and the notify-error path still lacks a safety argument. No new regressions were introduced, but verification is still incomplete for key kernel safety properties around condvar ownership/reclamation. Strengthening PM/condvar invariants is still required for a sound proof.
