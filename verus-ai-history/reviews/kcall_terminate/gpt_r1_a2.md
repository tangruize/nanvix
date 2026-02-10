# Review: kcall_terminate (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Missing invariant linking `running_pid` to the process set** (terminate.spec.rs: `ProcessManagerStateView`, `spec_is_running_process`; terminate.rs exec: `process_manager_terminate` ensures).
  - **Description:** `ProcessManagerStateView` adds `running_pid: Option<nat>` but does not state that `running_pid` (when `Some`) must be contained in `process_set`. This allows a state where `spec_is_running_process(pm_pre, pid)` is true while `!spec_pm_has_process(pm_pre, pid)` is also true. In such a case, the `process_manager_terminate` postconditions simultaneously require `InvalidArgument` (running PID) and `NoSuchProcess` (non-existent PID), making the contract inconsistent. This is a soundness risk.
  - **Suggested Fix:** Add a well-formedness predicate (or strengthen the type) enforcing `running_pid.is_Some() ==> process_set.contains(running_pid)`, and require it for `process_manager_terminate`/`terminate_model` (or exclude running PID from the non-existent-PID implication).

### Medium
- **`process_manager_terminate` remains an unlinked `external_body` trust boundary** (terminate.rs exec: `process_manager_terminate`).
  - **Description:** The postconditions are still assumed without a formal refinement to the verified process manager model (`verus/split/kernel/pm/process/manager/*`). The updated comments claim the PM module “covers the implementation side,” but there is no proof that the `ProcessManagerStateView` used here corresponds to the PM model’s invariants or that those guarantees are actually derived.
  - **Suggested Fix:** Either (a) connect `ProcessManagerStateView` to the PM model with a refinement lemma and use a verified wrapper instead of `external_body`, or (b) explicitly weaken the postconditions to the minimum required by this kcall proof and keep the stronger properties in the PM module.

### Low
- None.

## Positive Observations
- Running-process rejection is now modeled (spec predicate, external_body postconditions, and proof lemma).
- The previous over-strong “PID removal on success” property is no longer claimed; docs now state it is out of scope.
- Exec/spec/proof split remains clean and the verification run passes.

## Summary
The major behavioral mismatch from the previous review was fixed, and the removal assumption was dropped. However, the new running-PID modeling introduces a missing invariant that can make the external_body contract inconsistent, and the PM termination behavior is still trusted without a formal link to the verified PM model. Addressing these will be necessary before considering the verification sound.
