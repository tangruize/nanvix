# Review: ready (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **ReadyThread::thread_state_mut (exec)**: Implemented as `#[verifier::external]` with no machine-checked postconditions, so callers can arbitrarily mutate state and violate `wf()`/identity invariants. This is a soundness hole in a core module. **Suggested Fix:** Add a verified wrapper API for required mutations and restrict use of `thread_state_mut`, or model it with a spec/ensures via a trusted lemma and audit all call sites.
- **ReadyThread::run (exec/spec)**: The verified `run()` omits the `*mut ContextInformation` return and does not specify that the returned context pointer corresponds to `state.context_mut()`. This breaks semantic equivalence with the original scheduler transition. **Suggested Fix:** Model the context pointer (e.g., as an abstract token or ghost handle) and add a postcondition tying it to the pre-state context.

### Medium
- **ReadyThread::join_cond (missing, exec/spec/proof)**: The original method is omitted entirely, so coverage is incomplete and the synchronization behavior is unverified. **Suggested Fix:** Add a boundary model for `Condvar` and a verified `join_cond()` accessor (even if opaque) to preserve API coverage.
- **ReadyThread::new/from_state (spec)**: The specs do not constrain `admission_time` at all; `wf()` ignores it, and the constructors lack a postcondition relating it to `clock_now()` or even `>= 0`. This is too weak to support any time-based reasoning. **Suggested Fix:** Add a postcondition like `result.spec_admission_time() >= 0` or `== clock_now()` (captured via a local variable) and include any required invariant in `wf()` if time properties are needed.

### Low
- **RunningThread/ZombieThread boundary models (spec/exec)**: Correctness relies on cross-module assumptions that `from_state` preserves the stated properties, but no linkage proof is present here. **Suggested Fix:** When verifying `running.rs` and `zombie.rs`, prove that their real `from_state` implementations imply the boundary specs used here.

## Positive Observations
- Core transitions (`new`, `from_state`, `run`, `terminate`) preserve identity, interrupt state, mutex accounting, and drop safety, which match the intended safety properties.
- The model explicitly documents trust boundaries and aligns `ErrorCode::Interrupted` with a spec constant.
- Proof file provides lemmas for key state transitions and composition, improving auditability.

## Summary
Verification captures many safety-relevant properties of ready-to-running/zombie transitions, but there are meaningful gaps in coverage and semantic equivalence (notably `join_cond`, context pointer handling, and the unverified `thread_state_mut`). Tightening admission-time specs and reducing trusted escape hatches would improve soundness and utility for scheduler-level correctness.
