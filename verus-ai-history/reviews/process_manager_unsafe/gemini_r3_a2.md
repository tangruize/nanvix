# Review: process_manager_unsafe (gemini-3-pro-preview)

## Grade: B+

## Issues Found

### Medium (Unresolved)
- **Location:** `ProcessManagerUnsafeState` methods (`exit`, `exit_thread`, `sleep`, `giveup`, `join_thread`, `switch`) in `process_manager_unsafe.rs`
- **Description:** The model functions continue to accept `new_inner` as an independent argument, disconnected from the semantics of the operation being performed. For example, `exit` does not take an `ExitStatus` parameter, nor does it enforce that `new_inner` is the result of a process exit. It merely enforces that *if* the system transitions to `new_inner`, the globals are updated consistently. This means the functional correctness of `ProcessManager::exit` (that it actually terminates the process) is not verified, only the consistency of the global state updates.
- **Status:** **Rejected / Ignored.** The prover did not address this issue and did not provide a justification for retaining the loose specification.
- **Recommendation:** If full functional correctness is a goal, this must be addressed. If the scope is strictly limited to "maintenance of global invariants during arbitrary inner transitions", this limitation should be explicitly documented as a deviation from full correctness.

## Positive Observations
- **Low Issue Fixed:** The `spec_tid_valid` specification was updated to remove the unnecessary `< i32::MAX` constraint, matching the type definition.
- **Verification Passing:** The code continues to verify successfully.
- **Documentation:** The documentation explaining the "Scope and Limitations" is helpful, though it should be more explicit that "operation semantics" are not fully verified at this layer.

## Summary
The verification of `process_manager_unsafe` is sound regarding safety invariants (well-formedness of globals), but weak regarding functional correctness. The model ensures that *any* transition provided by the caller results in a consistent global state, but it does not verify that `exit` actually performs an exit, `sleep` actually sleeps, etc., because the logic linking the operation to the inner state transition is missing from the model constraints. The previous feedback on this point was not addressed. The fix for the TID bound issue was verified.
