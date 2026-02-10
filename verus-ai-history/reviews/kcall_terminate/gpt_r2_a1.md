# Review: kcall_terminate (gpt-5.2-codex)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `ProcessManagerStateView` / `spec_pm_has_process` (terminate.spec.rs), `process_manager_terminate` external_body (terminate.rs, exec)
  **Description:** The model’s `process_set` conflates all lifecycle states (ready/suspended/interrupted/zombie). As a result, the external_body contract permits `TmOk` for any PID in `process_set` (including interrupted/zombie), while the real implementation returns `NoSuchProcess` for those states. This is an equivalence gap and makes the spec too weak for callers that rely on termination failure for non-terminatable states.
  **Suggested Fix:** Refine the PM state view to distinguish terminatable vs non-terminatable states (e.g., separate sets for ready/suspended vs interrupted/zombie), or strengthen `process_manager_terminate` postconditions to forbid success unless the PID is in a “terminatable” subset.

### Low
- **Location:** `try_from_process_identifier` and `axiom_kernel_pid_is_valid` (terminate.rs, exec/proof)
  **Description:** Core correctness relies on trusted external_body/axiom assumptions for PID parsing and kernel PID validity. These are documented but not proven in this module, so soundness depends on external verification that is not enforced by this proof context.
  **Suggested Fix:** Add an explicit linkage (e.g., import verified lemmas from the PID module or a proof obligation that `spec_is_valid_pid(KERNEL_PID())` holds) or integrate these proofs into the verification build to ensure the assumptions are discharged.

## Positive Observations
- Coverage is complete: the only original function (`terminate`) is modeled and verified.
- The pipeline spec cleanly captures parse/terminate short-circuiting and error-code preservation.
- State invariants (`spec_pm_wf`) and error-path frame properties are explicit and used to avoid inconsistent postconditions.
- Trust boundaries are clearly documented, and proof/spec/exec separation is clean.
- Verification command passed with no errors.

## Summary
The verification is thorough and well-structured, with good error-propagation and safety guarantees. The main weakness is the coarse PM state abstraction that can over-approximate success cases; refining state modeling or tightening contracts would improve equivalence. Overall, this is a strong verification with a small but meaningful semantic gap.
