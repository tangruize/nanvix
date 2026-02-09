# Review: sleeping_process (gemini-3-pro-preview)

## Grade: B

## Issues Found

### High
- **Implementation Logic Unverified (Model-Only Verification)**: The verified `SleepingProcess` struct uses `Ghost<Seq<int>>` to track threads instead of the actual `NonEmptyVecDeque<SleepingThread>`. Consequently, the actual data structure manipulations (push, pop, remove, iteration) in the original source are not verified. The verification proves the correctness of the abstract state machine, not the executable implementation.
- **`wakeup_alarm` Partitioning Logic Unverified**: The complex partitioning logic in `wakeup_alarm` (iterating through threads, checking timestamps, splitting into two lists) is replaced by oracle parameters in the verified model. The correctness of the partition (i.e., that it actually respects the time comparison) is assumed via the oracle, not proven.

### Low
- **Unverified PID Immutability**: `state_mut` returns a mutable reference to the process state but assumes (via postcondition) that the PID does not change. Since `state_mut` is `external_body`, the caller is responsible for upholding this, but there is no mechanism to enforce it if the underlying `ProcessState` allows modification.
- **`find_thread` Returns Ghost Indices**: The verified `find_thread` returns `Ghost<Option<int>>` (indicating which list the thread is in) rather than a reference to the thread. This limits the utility of the spec for callers needing the actual thread reference.

## Positive Observations
- **Strong Abstract Invariants**: The model enforces rigorous invariants for thread management, including uniqueness (no duplicates) and disjointness between sleeping and zombie lists.
- **Clear State Machine Model**: The transitions between `SleepingProcess`, `RunnableProcess`, and `InterruptedProcess` are clearly modeled and proven to preserve process identity and thread conservation.
- **Excellent Documentation**: The code is well-documented, clearly explaining the verification strategy, the trust boundaries (oracles), and the mapping between the model and the original code.
- **Clean Split Structure**: The separation of executable model (`sleeping.rs`), specifications (`sleeping.spec.rs`), and proofs (`sleeping.proof.rs`) is clean and follows best practices.

## Summary
The verification of `sleeping_process` is a high-quality abstract model of the component's state machine. It successfully proves that *if* the underlying list operations behave as modeled (preserving elements, respecting partitions), then the process state transitions are safe and correct. However, because it abstracts away the actual `VecDeque` storage and replaces algorithmic logic (like `wakeup_alarm`'s iteration) with oracles, it stops short of verifying the executable implementation. It provides strong confidence in the design protocol but no guarantees about the implementation's adherence to that protocol (e.g., bug-free list manipulation). To achieve an "A" grade, the verification would need to link the abstract model to the actual `VecDeque` implementation.
