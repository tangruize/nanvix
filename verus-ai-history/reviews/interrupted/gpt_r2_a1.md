# Review: interrupted (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Missing verified `join_cond`** (exec/spec/proof: omitted): The original `InterruptedThread::join_cond` exists in `src/kernel/src/pm/thread/interrupted.rs` but has no verified counterpart in the split module. This violates coverage and weakens equivalence because the sync-facing API is entirely unmodeled. **Suggested Fix:** Add a boundary model for `Condvar` (opaque view) and a verified/external `join_cond` with at least identity/purity postconditions; or provide a verified stub that returns an abstract condvar handle with a spec view.
- **Unverified `thread_state_mut` trust boundary** (exec: `InterruptedThread::thread_state_mut`): The function is marked `#[verifier::external]` with no machine-checked postconditions, allowing callers to mutate `ThreadState` arbitrarily and potentially violate `wf()` or `spec_id()` invariants. This is a soundness gap in a core module. **Suggested Fix:** When Verus supports `&mut` returns, replace with a verified function with postconditions preserving `wf()` and identity, or refactor callers to use verified setters with explicit postconditions today.

### Medium
- None.

### Low
- **ReadyThread boundary omits `admission_time`** (exec/spec/proof: `ReadyThread::from_state`): The verified boundary model ignores the scheduling field set in the real implementation, so equivalence is partial for scheduling behavior. **Suggested Fix:** Add an abstract `admission_time` field/view and specify it as unconstrained or monotonic, or explicitly state in the spec that scheduling fields are outside scope and must be verified in `ready.rs`.

## Positive Observations
- `resume()` is well-specified: it proves interrupt reason propagation, identity preservation, and well-formedness, along with mutex, drop-safety, and stack ownership invariants.
- The spec cleanly models the `InterruptReason` variants and enforces validity via `wf()`.
- The spec/proof split is clear and the trust boundary for omitted/unsupported features is documented.

## Summary
Coverage is close but incomplete due to the missing `join_cond` model, and the external `thread_state_mut` leaves a significant soundness gap. Core safety properties around `resume()` are well captured, but scheduling equivalence is explicitly out of scope. Address the missing API and tighten the trust boundary to raise confidence.
