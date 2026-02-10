# Review: kcall_signal_cond (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `spec_broadcast_semantics` (signal_cond.spec.rs, spec)
  - **Description:** The non-broadcast case is now exact, but the broadcast case still permits `awakened == 0` on success even when `spec_num_waiters(cond_addr) > 0`. In the concrete `notify_all`, if there are waiters and all wakeups fail, the function returns `Err`, so a successful call with waiters cannot yield 0. The spec remains weaker than the implementation for broadcast success and could mask regressions.
  - **Suggested Fix:** Strengthen the broadcast branch to require `spec_num_waiters(cond_addr) > 0 ==> awakened >= 1` (or `awakened == 0 ==> spec_num_waiters(cond_addr) == 0`). Update any dependent lemmas accordingly.

- **Location:** `signal_cond_model` ensures and `lemma_notify_error_skips_put_cond` (signal_cond.rs exec / signal_cond.proof.rs)
  - **Description:** The model still skips `put_cond` on notify error and provides no invariant guaranteeing the PM slot is reclaimed afterward. This remains a correctness gap for resource reclamation, as the verification does not establish that the condvar slot is eventually released on this path.
  - **Suggested Fix:** Either adjust the implementation/model to call `put_cond` on notify error, or add a PM-level invariant (and spec token) that proves eventual reclamation when only the Condvar ref is dropped.

### Low
- None.

## Positive Observations
- The non-broadcast semantics were strengthened to enforce `awakened == 1` when waiters exist, matching the concrete `notify_first` behavior.
- Exec/spec/proof separation and short-circuit error propagation remain faithful to the original control flow.

## Summary
The non-broadcast semantics issue is fixed, but broadcast success is still underspecified and the notify-error reclamation gap persists. These remaining weaknesses prevent full semantic alignment with the concrete kernel behavior.
