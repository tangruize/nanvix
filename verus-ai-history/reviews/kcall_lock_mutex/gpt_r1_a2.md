# Review: kcall_lock_mutex (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `lock_mutex_model` safety contract (exec)
  - **Description:** The unsafe-use preconditions from the original function (caller not the kernel process, no resources held, no ProcessManager reference) are still not encoded as `requires` or linked to PM invariants. The new documentation claims these are call-site responsibilities, but there is no formal hook in this module proving the call sites satisfy them.
  - **Suggested Fix:** Add explicit `requires` predicates (or reference a verified PM invariant) to model the safety contract, and prove call sites establish them.

### Low
- **Location:** `lock_mutex_model` signature (exec)
  - **Description:** `pid`/`tid` remain omitted, so the model is not signature-equivalent to the source and does not prove their irrelevance. This is safe today but risks divergence if future logic uses them.
  - **Suggested Fix:** Include `pid`/`tid` as parameters (even if unused) or add a lemma asserting functional independence from those inputs.

## Positive Observations
- The prior timeout-value-threading gap is fixed: the parsed `TimeoutView` is now ghost-threaded to `mutex_lock_model` and tied to `(timeout_s, timeout_ns)` via `spec_parsed_timeout_for_lock` and `lemma_timeout_value_reaches_lock`.
- The pipeline/error-propagation specs and proofs remain clean and aligned with the original control flow.
- External trust boundaries are still clearly documented and verification passes.

## Summary
The main value-level timeout issue has been addressed, but the unsafe-use contract and pid/tid omission remain unmodeled. Verification is strong for control-flow and error propagation, yet still incomplete on caller-side safety constraints.
