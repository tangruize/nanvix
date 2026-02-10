# Review: kcall_terminate (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **`process_manager_terminate` remains an unlinked `external_body` trust boundary** (terminate.rs exec: `process_manager_terminate`).
  - **Description:** The updated `spec_pm_wf` invariant fixes the earlier inconsistency, but the core PM behavior is still assumed without a refinement proof to the verified process manager model (`verus/split/kernel/pm/process/manager/*`). The comments assert the PM module “covers the implementation side,” yet there is no formal link between `ProcessManagerStateView` here and the PM model’s view/invariants.
  - **Suggested Fix:** Add a refinement lemma mapping the PM model’s state to `ProcessManagerStateView` and derive the `process_manager_terminate` postconditions, or weaken the postconditions to those strictly needed by the kcall proof.

### Low
- **Outdated doc comment about PID removal** (terminate.rs exec: doc comment on `terminate_model`).
  - **Description:** The comment still claims the model proves “PID removal on success,” which is no longer asserted after the recent fix. This is misleading documentation.
  - **Suggested Fix:** Update the comment to reflect the current guarantees (state preservation on error; no PID removal claim).

## Positive Observations
- The previous inconsistency between running PID and process-set membership is fixed via `spec_pm_wf`, and it is enforced as a pre/postcondition.
- Running-process rejection and non-existent-PID errors are now modeled without contradictory contracts.
- Verification passes and the exec/spec/proof split remains clean.

## Summary
The prior high-severity issue is fixed by introducing a well-formedness invariant. The remaining concern is the still-unproven `external_body` for PM termination behavior, plus a minor doc mismatch; addressing these would make the verification closer to fully sound.
