# Review: kcall_wait_cond (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `wait_cond.rs` (exec, module docs)
  **Description:** Liveness is still explicitly out of scope; there is no proof or assumption that `cond.wait` or `mutex.lock(None)` eventually returns (under fairness/timeout assumptions). For a wait primitive, eventual return is a key property.
  **Suggested Fix:** Add a liveness lemma (or reference an existing scheduler/condvar liveness proof) and thread it as an explicit assumption in this module’s contract.

### Low
- None.

## Positive Observations
- No regressions in the exec/spec/proof split; equivalence and error precedence remain faithfully modeled.
- Architectural and resource-release fixes from the prior round remain intact.
- Verification still passes after the updates.

## Summary
The only remaining gap is liveness, which is still deferred without an explicit assumption. Otherwise, the verification is sound and unchanged from the prior round.
