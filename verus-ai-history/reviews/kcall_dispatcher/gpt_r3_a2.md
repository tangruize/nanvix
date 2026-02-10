# Review: kcall_dispatcher (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `do_kcall_context` / `remote_dispatch_verified` (exec)
  **Description:** Error-code propagation on failure paths is still not part of the postconditions. The new comment claims the property is "verified internally" but it is not exposed in the function contract, so equivalence remains weak: an implementation can satisfy the spec while returning the wrong error code. This was a prior issue and is not fixed.
  **Suggested Fix:** Strengthen the contracts using ghost returns or wrapper functions that return both the outcome and the result, allowing postconditions to assert `result == DispatchResult::error(outcome.error_code)` on each failure path.

- **Location:** `handle_sleep_error_killed` (exec)
  **Description:** The Killed path still does not model the critical side-effect of invoking `ProcessManager::exit(ErrorCode::Interrupted)` before divergence. The new note documents this gap but does not fix it, so the safety-critical termination behavior remains outside the model.
  **Suggested Fix:** Add an external-body wrapper that models the exit side-effect and require it to be invoked before divergence, or introduce ghost state to record termination and prove it is set on this path.

### Low
- **Location:** `dispatcher.spec.rs` + `lemma_kcall_constants_consistency` (spec/proof)
  **Description:** Kcall constants remain manually mirrored from the source enum, so drift can still occur without breaking verification. This was not addressed.
  **Suggested Fix:** Generate constants from the source enum (build step) or enforce a CI check comparing the enum values to the spec constants.

- **Location:** Module-level properties (exec/spec)
  **Description:** Liveness properties for sleepable and remote-dispatch calls are still unspecified and unproven. No new assumptions or lemmas were added.
  **Suggested Fix:** Add explicit progress/liveness assumptions in higher-level verification, or document the liveness scope as intentionally out-of-model.

## Positive Observations
- Verification still passes, and the dispatch routing logic remains faithful to the original match structure.
- The added commentary clarifies intent and trust boundaries, which helps future auditing.

## Summary
The re-review confirms that prior issues were mostly documented rather than fixed: error-code propagation is still absent from contracts and the Killed-path side-effect remains unmodeled. Overall verification is solid for routing logic, but key equivalence and safety properties remain outside the verified surface.
