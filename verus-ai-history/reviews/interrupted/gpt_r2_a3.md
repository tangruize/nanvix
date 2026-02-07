# Review: interrupted (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Missing verified `join_cond` still present** (exec/spec/proof): The module continues to omit `InterruptedThread::join_cond` and does not add a boundary model or external stub. The code still explicitly documents the omission, but the sync-facing API remains unmodeled, so the prior coverage gap is not fixed. **Suggested Fix:** Add an opaque `Condvar` boundary model or external stub with at least purity/identity postconditions, or verify it within the sync subsystem.
- **Unverified `thread_state_mut` trust boundary still present** (exec): `thread_state_mut` remains `#[verifier::external]` with no machine-checked postconditions. Documentation was already present, but this does not prevent callers from violating `wf()` or `spec_id()` invariants, so the soundness gap remains. **Suggested Fix:** Refactor to verified setters or replace once `&mut` returns are supported, with enforced postconditions.

### Medium
- None.

### Low
- **ReadyThread boundary still omits `admission_time`** (exec/spec/proof): The scheduling field is still omitted from the boundary model. The explicit out-of-scope note does not eliminate the divergence in behavior. **Suggested Fix:** Model an abstract `admission_time` (unconstrained) or discharge this equivalence gap in `ready.rs` verification.

## Positive Observations
- Core `resume()` transition properties remain well specified and consistent with the implementation.
- Trust-boundary documentation remains explicit about the limitations, aiding auditability.

## Summary
The prior high-severity issues are not actually fixed; they are still documented but remain outside the verification boundary. No new issues were introduced, but verification coverage is still incomplete and the soundness gap persists. Addressing the missing API model and tightening mutation boundaries is required for a clean pass.
