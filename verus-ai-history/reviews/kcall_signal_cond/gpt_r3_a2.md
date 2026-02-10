# Review: kcall_signal_cond (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `spec_put_cond_completed`, `spec_cond_ref_released`, `spec_condvar_acquired` (spec) and external bodies in `signal_cond.rs`.
  **Description:** The predicates remain uninterpreted and are still used as abstract tokens. There is still no evidence that PM/condvar state invariants (e.g., map ownership or refcount changes) are established or preserved, so the proof cannot rule out stale entries or resource leaks.
  **Suggested Fix:** Connect these predicates to concrete PM/condvar state models or import their invariants, and prove that get/drop/put maintain those invariants (or add a cross-module lemma stating what `spec_put_cond_completed` guarantees).

### Low
- **Location:** `lemma_notify_error_skips_put_cond` (proof) / `signal_cond_model` (exec).
  **Description:** The model still short-circuits on notify error without calling `put_cond`, and the proof only documents this behavior. There is no proof of safety for this path (eventual reclamation or non-leak), so the original gap remains.
  **Suggested Fix:** Either change the implementation to call `put_cond` on notify error and update the model, or add a PM-level invariant showing that the slot is eventually reclaimed on error paths.

## Positive Observations
- Coverage remains complete: the only original function (`signal_cond`) has exec/spec/proof counterparts.
- The pipeline, error propagation, and broadcast semantics are still precisely modeled and proven.
- The exec/spec/proof split is clean and consistent with prior version.

## Summary
The previous issues were not actually fixed: the core resource-management predicates are still abstract and the notify-error path still lacks a safety argument. No new regressions were introduced, but the verification remains incomplete for key kernel safety properties around condvar ownership/reclamation. Strengthening PM/condvar invariants is still required for a sound proof.
