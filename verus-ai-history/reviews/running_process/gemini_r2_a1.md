# Review: running_process (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **Model vs Implementation Gap**: The verification is performed on a model (`Ghost<Seq<int>>` + counters) rather than the actual `NonEmptyVecDeque` implementation. While this correctly verifies the state machine logic, it relies on the assumption that the `u64` counters in the real code stay in sync with the collection lengths.
- **Oracle Parameters**: `wakeup` and `try_join_thread` use oracle parameters (`found`, `tag`) with preconditions linking them to ghost state. This is sound for the model but requires the actual implementation to perform the corresponding checks (which it does via `remove_if`), which is not formally linked in this verification.

## Positive Observations
- **High-Fidelity State Machine Model**: The model faithfully captures the complex transitions of `RunningProcess`, including the correct movement of threads between ready, interrupted, sleeping, and zombie lists during `schedule`, `sleep`, `exit`, and `exit_thread`.
- **Bug Discovery & Fix**: The verification documentation explicitly notes a bug found in `exit_thread` (regarding `self.zombie.take()` vs `Some(zombie_threads)`), which has been fixed in the original source. This demonstrates high value.
- **Strong Invariants**: The proofs establish conservation of total thread count and preservation of process/thread IDs across all transitions.
- **Clean Split**: The separation into `exec` (model), `spec`, and `proof` files is clean and follows best practices. The `spec` file provides clear Views and logical models for all operations.

## Summary
The verification of `running_process` is excellent. Although it uses a model-based approach rather than verifying the raw implementation directly (likely due to `NonEmptyVecDeque` complexity), the model is detailed and accurate. It successfully validates the critical OS state transitions and has already proven its worth by identifying a logic bug in `exit_thread`. The specifications clearly define the intended behavior of process scheduling and termination.
