# Review: kcall_dispatcher (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `do_kcall_context` / `remote_dispatch_verified` (exec)
  **Description:** The specs do not capture error-code propagation on failure paths (pid/tid retrieval failure or scoreboard access/dispatch failure). Postconditions only require `result.wf()` (and a few success-value constraints), so the verified model allows arbitrary error codes even when the original returns the specific `Error.code` from ProcessManager/ScoreBoard. This weakens equivalence and makes it possible for an implementation to satisfy the spec while returning incorrect error codes.
  **Suggested Fix:** Strengthen postconditions to relate the result to the corresponding subsystem outcome. For example, introduce ghost bindings (or return outcomes in a tuple) so the postcondition can state: if `pm_get_pid` fails then `result == DispatchResult::error(pid_outcome.error_code)`, and similarly for `pm_get_tid`, `scoreboard_get_mut`, and `scoreboard_dispatch_call` error paths.

- **Location:** `handle_sleep_error_killed` (exec)
  **Description:** The divergent Killed path is modeled as `external_body` with `ensures false` but does not model or require the side-effect of calling `ProcessManager::exit(ErrorCode::Interrupted)` that the original code performs. This leaves the most safety-critical behavior (forced termination) outside the model, so equivalence is incomplete for that path.
  **Suggested Fix:** Add an explicit external-body wrapper (or ghost state update) for `ProcessManager::exit` and require it to be invoked before divergence; alternatively, model a termination flag in the state and prove it is set on the Killed path.

### Low
- **Location:** `dispatcher.spec.rs` + `lemma_kcall_constants_consistency` (spec/proof)
  **Description:** Kcall number constants are manually mirrored from `KcallNumber` and are not mechanically linked. If the enum changes, verification can silently become stale while still passing, which is an equivalence risk.
  **Suggested Fix:** Generate these constants from the source enum (build script or include), or add a CI check that diffs the enum values against this spec.

- **Location:** Module-level properties (exec/spec)
  **Description:** No liveness properties are specified or proved for sleepable or remote-dispatch calls (e.g., eventual return, no deadlock). This is acknowledged by external-body boundaries but leaves liveness unverified.
  **Suggested Fix:** Add abstract liveness assumptions or progress lemmas for the scheduler/scoreboard in higher-level verification, or explicitly document the liveness scope as out-of-model.

## Positive Observations
- All original functions (`do_kcall`, `handle_sleep_error`) are represented and verified; the match-based dispatch structure closely mirrors the source.
- Classification, local/remote partitioning, and sleep-error routing are well specified with supporting lemmas.
- Trust boundaries are clearly documented, and the ABI encoding bridge (`do_kcall_encoded`) makes the representation gap explicit.

## Summary
The verification is structurally strong and mirrors the dispatch logic well, but several specs are too weak on failure-path error-code propagation and the Killed path side-effect, leaving equivalence gaps. Tightening these postconditions and modeling termination would significantly strengthen correctness without large changes to the proof structure.
