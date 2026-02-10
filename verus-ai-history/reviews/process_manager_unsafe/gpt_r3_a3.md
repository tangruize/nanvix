# Review: process_manager_unsafe (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `ProcessManagerUnsafeState::wf` (spec, `process_manager_unsafe.spec.rs`).
  **Description:** The invariant still does not relate `current_tid` to `current_pid` (no “current_tid ∈ threads(current_pid)” constraint). The spec explicitly retains T9 as a trust boundary, so executions where the running thread is not part of the running process remain admissible. This is an essential scheduler safety property.
  **Suggested Fix:** Extend `ProcessManagerInner` with a ghost PID→Set<TID> map and add an invariant tying `current_tid` to the current PID; propagate through inner operations and require it in `switch/sleep/exit` preconditions.

### Medium
- **Location:** `try_recv_*` (exec/spec, `process_manager_unsafe.rs`).
  **Description:** Per-thread message delivery is still not modeled. The spec only decrements `number_buffered_messages` and treats `tid` as a ghost tag, without proving the returned message belongs to that thread.
  **Suggested Fix:** Add ghost per-thread message queues in `ProcessManagerInner` and prove `try_recv` returns and removes the correct per-thread message.

- **Location:** `join_thread` (exec, `process_manager_unsafe.rs`).
  **Description:** The exit status returned from the harvest path remains an unconstrained ghost input; the T15 commentary explicitly defers correctness rather than proving it. This keeps the spec too weak for return-value correctness.
  **Suggested Fix:** Track zombie-thread exit statuses in the inner model and require `exit_status` to match the stored value for the joined TID.

### Low
- **Location:** `join_thread` loop (proof, `process_manager_unsafe.proof.rs`).
  **Description:** Liveness/termination is still a trust boundary; only partial correctness is shown.
  **Suggested Fix:** Add a higher-level fairness/notification assumption or temporal spec documenting the required liveness contract.

## Positive Observations
- The try-recv error path and unified `try_recv` entry point remain correctly modeled.
- No `assume` or `external_body` introduced; verification still passes.

## Summary
The updated files do not materially address the outstanding semantic gaps from the previous review. Verification remains incomplete for key safety properties (TID↔PID membership, per-thread message delivery, join exit-status correctness) and liveness.
