# Review: ready (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **ReadyThread::thread_state_mut (exec)**: Still `#[verifier::external]` returning `&mut ThreadState` with no machine-checked postconditions. Added docs/forwarding methods do not prevent arbitrary mutation; callers can still violate `wf()`/identity invariants. Soundness hole persists.
- **ReadyThread::run (exec/spec)**: Still omits `*mut ContextInformation` from the return. The new `RunResult` and comments do not model or relate a context pointer to `state.context_mut()`, so the verified semantics do not match the original scheduler transition.

### Medium
- **ReadyThread::join_cond (missing)**: Still omitted entirely, so synchronization behavior remains unverified and API coverage incomplete.

### Low
- **RunningThread/ZombieThread boundary models**: Still rely on cross-module assumptions with no linkage proof in this module.

## Positive Observations
- Constructors now ensure `spec_admission_time() >= 0` and `clock_now()` is constrained to non-negative values.
- Added verified forwarding methods (`set_interrupt_reason`, `store_mutex_guard`, `take_mutex_guard`) that preserve invariants and admission time.

## Summary
Admission-time constraints were added, but the major soundness gaps from the prior review remain (unverified mutable access, missing context pointer semantics, and omitted `join_cond`). Verification is still incomplete relative to the real kernel APIs.
