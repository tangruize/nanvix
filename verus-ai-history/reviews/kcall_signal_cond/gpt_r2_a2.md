# Review: kcall_signal_cond (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `spec_broadcast_semantics` (signal_cond.spec.rs, spec)
  - **Description:** The non-broadcast case is still too weak. It now bounds `awakened` by `spec_num_waiters`, but it still permits `awakened == 0` on success when `spec_num_waiters(cond_addr) > 0`. The concrete `notify_first` returns `Ok(1)` when there is at least one waiter (and returns `Err` on wakeup failure), so a successful call with waiters cannot yield 0. This leaves a gap that could mask regressions in the non-broadcast path.
  - **Suggested Fix:** Strengthen the spec to require `spec_num_waiters(cond_addr) > 0 ==> awakened == 1` (or an equivalent relation), while keeping the `awakened == 0` case for `spec_num_waiters == 0`. Update any dependent lemmas accordingly.

- **Location:** `signal_cond_model` ensures and `lemma_notify_error_skips_put_cond` (signal_cond.rs exec / signal_cond.proof.rs)
  - **Description:** The model still skips `put_cond` on notify error and provides no invariant guaranteeing the PM slot is reclaimed afterward. This remains a correctness gap for resource reclamation, as the verification does not establish that the condvar slot is eventually released on this path.
  - **Suggested Fix:** Either adjust the implementation/model to call `put_cond` on notify error, or add a PM-level invariant (and spec token) that proves eventual reclamation when only the Condvar ref is dropped.

### Low
- None.

## Positive Observations
- The non-broadcast spec was improved with an explicit `awakened <= spec_num_waiters` bound, tightening the success case.
- Coverage, exec/spec/proof split, and short-circuit error semantics remain well-aligned with the original code.

## Summary
One prior issue was partially addressed (added waiter bound), but the non-broadcast success condition is still weaker than the concrete behavior. The resource-reclamation gap on notify error remains unchanged. Overall verification is solid but not yet complete on these correctness points.
