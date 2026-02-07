# Review: interrupted (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Missing verified `join_cond` remains** (exec/spec/proof: still omitted): The module still omits `InterruptedThread::join_cond` and provides no verified boundary model or external stub. The new documentation explains the omission, but the sync-facing API remains unmodeled, leaving an observable behavior gap. **Suggested Fix:** Add an opaque `Condvar` boundary model (or external stub) with at least purity/identity postconditions, or explicitly scope and verify this in the sync subsystem.
- **Unverified `thread_state_mut` trust boundary remains** (exec: `InterruptedThread::thread_state_mut`): The function is still `#[verifier::external]` and returns `&mut ThreadState` without any machine-checked postconditions. Added comments do not enforce preservation of `wf()` or `spec_id()`, so callers can still violate invariants. **Suggested Fix:** Replace with verified setters or a verified wrapper when Verus supports `&mut` returns; until then, refactor call sites to avoid unrestricted mutation.

### Medium
- None.

### Low
- **ReadyThread boundary still omits `admission_time`** (exec/spec/proof): The omission is now documented as out of scope, but scheduling semantics still diverge from the real implementation. **Suggested Fix:** Either model an abstract `admission_time` with unconstrained value or explicitly discharge this gap in `ready.rs` verification.

## Positive Observations
- Trust-boundary documentation is more explicit about `join_cond` and `thread_state_mut` limitations and call sites.
- The `resume()` transition continues to carry strong invariants (interrupt reason propagation, identity preservation, and well-formedness).

## Summary
The two high-severity issues from the prior review are not substantively fixed; they are documented but still outside the proof boundary. As a result, verification coverage remains incomplete and the soundness gap persists. Addressing the missing API model and tightening the mutation boundary are required for a clean pass.
