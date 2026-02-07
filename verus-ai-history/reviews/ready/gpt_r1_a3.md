# Review: ready (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **ReadyThread::thread_state_mut (exec)**: Still `#[verifier::external]` returning `&mut ThreadState` with no machine-checked postconditions. Documentation does not prevent arbitrary mutation; callers can still violate `wf()`/identity invariants, so the soundness hole remains.
- **ReadyThread::run (exec/spec)**: Still omits the `*mut ContextInformation` return. The model continues to ignore the context pointer and has no postcondition tying it to `state.context_mut()`, so verified semantics diverge from the real scheduler transition.

### Medium
- **ReadyThread::join_cond (missing)**: Still omitted entirely, leaving synchronization behavior and API coverage unverified.

### Low
- **RunningThread/ZombieThread boundary models**: Still rely on cross-module assumptions without a linkage proof in this module.

## Positive Observations
- No regressions in admission-time constraints or forwarding-method specs.

## Summary
The updated files do not address the prior high/medium issues; the key soundness and coverage gaps persist. Verification remains incomplete relative to the concrete kernel APIs.
