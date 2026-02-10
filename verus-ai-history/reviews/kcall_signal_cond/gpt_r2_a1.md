# Review: kcall_signal_cond (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `spec_broadcast_semantics` (signal_cond.spec.rs, spec)
  - **Description:** The non-broadcast case only enforces `awakened <= 1` and does not relate the awakened count to the number of waiters. This permits `awakened == 1` even when `spec_num_waiters(cond_addr) == 0`, which is impossible in the concrete `notify_first` implementation (it returns `Ok(0)` when the queue is empty). The spec is therefore weaker than the intended behavior and could mask regressions.
  - **Suggested Fix:** Strengthen the spec to relate `awakened` to `spec_num_waiters` for the non-broadcast case (e.g., `awakened <= spec_num_waiters(cond_addr)` and `spec_num_waiters(cond_addr) > 0 ==> awakened == 1`). Update `lemma_notify_first_awakens_at_most_one` and any dependent proofs accordingly.

- **Location:** `signal_cond_model` ensures and `lemma_notify_error_skips_put_cond` (signal_cond.rs exec / signal_cond.proof.rs)
  - **Description:** On `notify` error, the model (correctly) skips `put_cond`, and the spec provides no token that the PM slot is reclaimed or eventually returned. This leaves a correctness gap for resource reclamation: the verification does not establish any invariant ensuring the condvar slot is freed after notify failure.
  - **Suggested Fix:** If the intended behavior is to reclaim the slot even on notify error, either (a) change the implementation to call `put_cond` in the notify-error path and update the model/spec, or (b) add a PM-level invariant (and corresponding spec token) that guarantees eventual reclamation when only the Condvar ref is dropped.

### Low
- None.

## Positive Observations
- Full coverage: the sole function in the original module (`signal_cond`) has a verified exec model with matching control flow.
- Error propagation, short-circuit ordering, and awakened-count preservation are explicitly specified and proven.
- Clear trust-boundary separation with external_body models and a clean split between exec/spec/proof files.

## Summary
The verification is well-structured and faithfully mirrors the original control flow, including error short-circuiting and broadcast-vs-signal behavior. The main gaps are a too-weak non-broadcast specification and the lack of a reclamation guarantee on notify failure. Strengthening the broadcast semantics spec and addressing resource-reclamation invariants would bring the model closer to essential correctness.
