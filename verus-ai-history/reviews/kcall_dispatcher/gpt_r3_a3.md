# Review: kcall_dispatcher (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `do_kcall_context` / `remote_dispatch_verified` (exec)
  **Description:** Error-code propagation is still not part of the function contracts. The added lemma about `DispatchResult::error` does not tie the *returned* error code to the specific subsystem outcome on each failure path. An implementation could still satisfy the spec while returning a different error code. This issue remains unfixed.
  **Suggested Fix:** Add ghost outputs or wrapper functions that return both the subsystem outcome and the `DispatchResult`, and state in postconditions that on failure `result.value == outcome.error_code` (and similarly for scoreboard and pid/tid retrieval failures).

### Low
- **Location:** `dispatcher.spec.rs` + `lemma_kcall_constants_consistency` (spec/proof)
  **Description:** Kcall constants are still manually mirrored from the source enum; drift can occur without verification failure.
  **Suggested Fix:** Generate constants from the source enum (build step) or add CI checks comparing enum values to spec constants.

- **Location:** Module-level properties (exec/spec)
  **Description:** Liveness properties for sleepable and remote-dispatch calls remain unspecified and unproven.
  **Suggested Fix:** Add explicit liveness assumptions or progress lemmas in a higher-level model, or document the liveness scope as intentionally out-of-model.

## Positive Observations
- The Killed-path side effect is now modeled: `handle_sleep_error_killed` calls `pm_exit_interrupted()` before divergence, addressing the prior equivalence gap.
- Verification still passes and routing logic remains faithful to the original dispatcher.

## Summary
The prover fixed the Killed-path side-effect modeling, but error-code propagation is still not enforced by contracts, and the constant-mirroring and liveness gaps remain. Overall verification is improved yet not fully complete or equivalence-tight.
