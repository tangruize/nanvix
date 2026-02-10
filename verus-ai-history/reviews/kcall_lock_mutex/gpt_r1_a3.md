# Review: kcall_lock_mutex (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `spec_lock_mutex_safety_preconditions` + `lock_mutex_model` (spec/exec)
  - **Description:** The unsafe-use contract is now formalized as uninterpreted predicates, but it is still not enforced as `requires` on `lock_mutex_model`, nor is there a proof that call sites establish it. This leaves the safety obligations purely documentary within this module.
  - **Suggested Fix:** Add `requires spec_lock_mutex_safety_preconditions(pid, tid)` to the verified entry point (or a wrapper) and prove in the PM/kcall dispatch layer that the predicate holds for each invocation.

### Low
- None.

## Positive Observations
- The pid/tid issue is resolved: a lemma now explicitly states result independence from pid/tid, guarding against future signature drift.
- Timeout value threading remains correct and is tied to `(timeout_s, timeout_ns)` via `spec_parsed_timeout_for_lock` and `lemma_timeout_value_reaches_lock`.
- The pipeline/error-propagation specs and proofs remain aligned with the original control flow, and verification passes.

## Summary
Most prior issues are fixed and the model is strong on functional behavior, but the caller safety contract is still not enforced in the verified interface. Once the preconditions are tied to a checked call site, the verification would be effectively complete.
