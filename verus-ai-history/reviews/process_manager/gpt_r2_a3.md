# Review: process_manager (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Capability semantics still unmodeled** (exec: `process_manager.rs` `capctl`, `capctl_error_noop`): The update adds a rationale but still models `capctl` as a queue-level no-op without any abstract capability state. This remains weaker than the real behavior (capability bits change), so security-relevant semantics are unverified. **Suggested Fix:** Add a ghost/spec capability map and specify set/clear effects plus their relation to `outer_has_capability`.
- **Thread-state branch selection still unconstrained** (exec: `process_manager.rs` `sleep_thread_running`, `exit_thread_running`, `exit_thread_to_suspended`, `exit_thread_to_zombie`, wrappers): There are still no predicates tying a branch choice to actual thread state (runnable vs sleeping vs zombie). The model can pick any transition. **Suggested Fix:** Introduce abstract thread-state predicates and require them on the corresponding transitions.

### Low
- **Scheduling order/fairness remains abstracted** (spec/exec): Ready queue is still a `Set<int>` and `chosen_next` is unconstrained, so earliest-admission/fairness is not captured. **Suggested Fix:** If fairness is required, model ready as `Seq<int>` or add an ordering predicate.

## Positive Observations
- The previously missing outer post-message error-path stub (`outer_post_message_not_found`) is now present, covering receiver-not-found and borrow-failure paths.
- `post_message` retains the strengthened precondition and success-path spec, and verification still passes cleanly.
- No `assume` or `external_body` constructs were introduced.

## Summary
The prover fixed the outer post-message error-path coverage and preserved sound queue invariants, but capability updates and thread-state-dependent branching are still unmodeled. The verification is stronger than before but not yet complete for the full semantic behavior of these operations.
