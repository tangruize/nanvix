# Review: kcall_terminate (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `process_manager_terminate` external_body (terminate.rs, exec) and `spec_terminate_possible` (terminate.spec.rs)
  **Description:** The refined model still does not state that a PID in the terminatable set (ready/suspended) will succeed. The contract only says success implies terminatable membership, but the converse is absent, so the spec allows spurious errors even when the real implementation would return Ok for ready/suspended processes. This leaves a liveness/functional completeness gap and weakens equivalence.
  **Suggested Fix:** Strengthen `process_manager_terminate` with a postcondition that if `pid` is in `terminatable_set` (and not running/kernel by `spec_pm_wf`), then the result is `TmOk` with no error; then add the corresponding lemma to propagate success through the kcall.

### Low
- **Location:** `try_from_process_identifier` and `axiom_kernel_pid_is_valid` (terminate.rs, exec/proof)
  **Description:** Soundness still depends on trusted `external_body`/axiom assumptions without enforced cross-module proof composition in this verification run. The trust chain is documented, but the current verification command does not check the providing modules, so these assumptions are not mechanically discharged here.
  **Suggested Fix:** Integrate or reference the PID and PM module verification in the build (or add a proof import) so these assumptions are validated end-to-end.

## Positive Observations
- The previous medium issue about lifecycle-state conflation is fixed: the new `terminatable_set` and postconditions rule out success for interrupted/zombie PIDs.
- The model now explicitly encodes terminatable vs non-terminatable states and refines `spec_pm_wf` accordingly.
- Coverage and split quality remain good, and verification passes.

## Summary
The fix properly addresses the over-approximation of success by introducing a terminatable subset and stronger error conditions. However, the spec is still too weak to guarantee success for terminatable PIDs, leaving a functional completeness gap. Trusted cross-module assumptions remain without enforced discharge in this module’s verification run.
