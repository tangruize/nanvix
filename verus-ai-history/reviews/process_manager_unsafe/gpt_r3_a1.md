# Review: process_manager_unsafe (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `ProcessManagerUnsafeState::wf` (spec, `process_manager_unsafe.spec.rs`).
  **Description:** The state invariant does not relate `current_tid` to `current_pid` (no “current_tid ∈ threads(current_pid)” constraint). This allows verified executions where the running thread is not part of the running process, which undermines safety for context switching and address-space selection. The model explicitly defers this (T9), but it is an essential correctness property for an OS scheduler.
  **Suggested Fix:** Extend `ProcessManagerInner` with a ghost map from PID→Set<TID> and add an invariant `current_tid ∈ threads(current_pid)`; propagate it through inner operations and require it in `switch/sleep/exit` preconditions.

### Medium
- **Location:** `try_recv_*` (exec, `process_manager_unsafe.rs`).
  **Description:** The original `try_recv` returns `Result<Option<Message>, Error>` and can fail on `try_borrow_mut()`. The model only provides `try_recv_some` and `try_recv_none` success cases, with no error path or unified spec for the original signature. This weakens coverage and omits error propagation behavior.
  **Suggested Fix:** Add a unified `try_recv` model that returns a Result-like pair and include a `try_recv_error` path for borrow failure; connect it to the existing some/none branches.

- **Location:** `try_recv_some/try_recv_none` (exec/spec, `process_manager_unsafe.rs`).
  **Description:** Per-thread message delivery is not modeled; the spec only decrements `number_buffered_messages` and uses a ghost `tid` without tying it to a queue or message content. This omits the key safety property that the returned message belongs to the requested thread.
  **Suggested Fix:** Introduce ghost per-thread message queues in `ProcessManagerInner` (e.g., `Map<TID, Seq<Message>>`) and prove that `try_recv` returns the head of the correct queue and updates that queue.

- **Location:** `join_thread` (exec/proof, `process_manager_unsafe.rs`).
  **Description:** The returned `exit_status` is an unconstrained ghost input; the model does not link it to the target zombie thread’s stored status. This makes the spec too weak for correctness of `join_thread`’s return value.
  **Suggested Fix:** Track zombie thread exit statuses in a ghost map and require `exit_status` to match the stored value for the joined TID.

### Low
- **Location:** `join_thread` loop (proof, `process_manager_unsafe.proof.rs`).
  **Description:** Only a loop-invariant/partial-correctness argument is provided; termination depends on eventual `notify_all` and scheduling fairness (liveness is a trust boundary). This leaves a key liveness property unproven.
  **Suggested Fix:** Add a higher-level liveness assumption or a temporal specification that the target thread eventually exits and signals, and document it as a required scheduling/fairness contract.

## Positive Observations
- All top-level unsafe wrapper functions are modeled and the split between exec/spec/proof is clean.
- The `switch` model carefully captures the stale-atomic PID comparison and quantum reset logic.
- Divergence of `exit/exit_thread` is handled with a ghost flag, preventing post-exit reasoning.

## Summary
The verification is well-structured and matches the control-flow of the unsafe wrapper, but it omits several essential correctness properties (TID↔PID membership, message delivery, join status correctness, and liveness). Addressing these would significantly strengthen the guarantees and improve semantic equivalence.
