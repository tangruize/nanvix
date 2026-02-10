# Review: kcall_signal_cond (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Broadcast semantics not specified (spec/exec)**: `notify_model(cond_addr, broadcast)` has no postconditions tying `broadcast=false` to `notify_first`-style behavior (e.g., awakening at most one thread) or `broadcast=true` to `notify_all`. The verification would still pass if the implementation ignored `broadcast` or always used the same notify variant.
  - **Suggested Fix**: Strengthen `notify_model` with broadcast-dependent postconditions (e.g., `broadcast ==> awakened == num_waiters` and `!broadcast ==> awakened <= 1`), or split into `notify_all_model`/`notify_first_model` and prove the branch selection matches `broadcast`.

### Medium
- **Resource-release guarantees are too weak (spec/proof)**: `spec_cond_ref_released` and `spec_cond_slot_returned` are uninterpreted, and the external-body models do not connect get/put/drop to an abstract PM/condvar state. This makes the proof largely about error propagation and control flow, not about ownership or slot balancing.
  - **Suggested Fix**: Introduce an abstract PM/condvar state with invariants (e.g., slot ownership/refcount), and have `get_cond_model`/`put_cond_model`/`drop_cond_model` update and preserve these invariants. Then prove balanced release.
- **No guarantee of slot return on notify failure (exec/spec)**: The model mirrors the current behavior where `put_cond` is skipped on notify error, so the PM slot may remain unreturned. If system correctness requires cleanup even on notify failure, this is a missing safety property.
  - **Suggested Fix**: Either change the implementation to call `put_cond` on notify error (and update the spec), or formalize/verify a PM cleanup invariant that guarantees eventual slot reclamation.

### Low
- **Liveness properties not proven (spec/proof)**: The model explicitly excludes scheduler/liveness properties such as “waiting threads eventually wake.” If liveness is required for this kcall, it remains unverified.
  - **Suggested Fix**: Add liveness specifications in the condvar/scheduler modules and link them here, or explicitly document the required assumptions.

## Positive Observations
- Full functional coverage: the only function in the source file is modeled and verified.
- Control-flow equivalence is captured (short-circuit error propagation, notify-before-put ordering, and condvar drop on get_cond success).
- Architecture constraints and safety preconditions are explicitly encoded and proven stable.
- The spec/proof split is clean and the documentation clearly identifies trust boundaries and known limitations.

## Summary
The verification correctly models the control-flow and error propagation of `signal_cond`, but it remains shallow around broadcast semantics and PM/condvar state effects. Strengthening the notify specification and introducing state invariants for get/put/drop would materially improve assurance.
