# Review: kcall_terminate (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Missing running-process rejection in spec/model** (terminate.spec.rs: `spec_terminate_possible`; terminate.rs exec: `process_manager_terminate` external_body).
  - **Description:** The original `ProcessManager::terminate` returns `InvalidArgument` when the target PID is the running process. The spec explicitly omits this, so the model allows `Success` for a running PID when it should return error. This is an observable behavior mismatch for the kcall.
  - **Suggested Fix:** Extend `ProcessManagerStateView` to include the running PID (or a predicate for “is_running”), and strengthen `process_manager_terminate` postconditions to return `InvalidArgument` when `pid == running_pid`. Propagate this into `spec_terminate_possible` and adjust proofs accordingly.

- **Success-state removal is too strong / likely inaccurate** (terminate.rs exec: `process_manager_terminate` ensures; terminate.proof.rs: `lemma_pid_removed_on_success`, `lemma_double_terminate_impossible`).
  - **Description:** The model assumes a successful terminate removes the PID from the process set and makes double-terminate impossible. In the real code, a ready process can be terminated and then resumed back into the ready queue (`RunnableProcess::terminate` → `InterruptedProcess::resume`), so the PID can remain in PM structures. This makes the spec stronger than the implementation and risks proving properties that the real system does not guarantee.
  - **Suggested Fix:** Refine the PM state model to represent process lifecycle states (ready/suspended/interrupted/zombie) and update postconditions to reflect actual transitions. If the intended property is “not terminatable again,” add an explicit predicate for terminatability instead of removal.

### Medium
- **Unproven external_body for core PM behavior** (terminate.rs exec: `process_manager_terminate`).
  - **Description:** `process_manager_terminate` is an `external_body` with strong postconditions, but there is no verified linkage to an actual PM model in the Verus tree. This is a trust assumption for a core kernel operation and weakens soundness.
  - **Suggested Fix:** Replace the `external_body` with a verified model from the PM module (or add a proof that the PM model satisfies these postconditions), or explicitly document and minimize the trusted surface by weakening postconditions to only those required by the kcall proof.

### Low
- None.

## Positive Observations
- The exec/spec/proof split is clean and well-documented, with explicit trust boundaries and pipeline reasoning.
- Error propagation and error-code preservation are modeled precisely, matching the original control flow.
- The verification proves key short-circuit and exhaustiveness properties, and the verification command passes.

## Summary
The verification captures the kcall’s control-flow and error propagation well, but it omits a real error case (running PID) and assumes stronger PM state changes than the implementation provides. Tightening the PM model and eliminating or justifying the external_body assumptions would significantly improve equivalence and soundness.
