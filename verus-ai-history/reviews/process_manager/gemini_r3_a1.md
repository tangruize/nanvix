# Review: process_manager (gemini-3-pro-preview)

## Grade: B

## Issues Found

### High
- **Disconnected Model**: The verified `ProcessManagerInner` struct is a shadow model (containing only PIDs and counts) that differs significantly from the original implementation (which contains `LinkedList<Process>`, `ThreadManager`, etc.). The verification proves properties of this abstract model, but there is no formal link or refinement proof connecting the model to the actual executable code. The `LinkedList` manipulations and process object management are effectively unverified.

### Medium
- **Thread-Level Logic Trusted**: The model trusts the correctness of thread-level decisions. For example, `exit_thread` in the model takes a `to_zombie` boolean parameter to determine the state transition. In the real implementation, this decision depends on complex checks of the process's thread list. If the implementation incorrectly calculates this condition, the model's proof provides no protection.
- **Scheduling Policy Unverified**: The `schedule` function in the verified model accepts a `chosen_next` PID as a parameter (Trust Boundary T1). The logic for selecting the next process (e.g., priority, fairness, FIFO) is external to the model and thus unverified.

### Low
- **Type Mismatch**: The model uses `i32` for PIDs, whereas the original code uses a `ProcessIdentifier` wrapper (likely `u32`-based). While `spec_can_create_process` checks `i32::MAX` bounds, semantic mismatches in wrapping behavior or exact type limits could exist.

## Positive Observations
- **Strong Well-Formedness Invariants**: The `wf()` predicate rigorously defines the system state, enforcing pairwise disjointness of queues, strict PID bounds, and consistency between runtime counts and ghost sets.
- **Kernel Safety Proven**: The verification successfully proves the liveness of the kernel process (PID 0), ensuring it is always either running or ready and never in an invalid state.
- **Explicit Trust Boundaries**: The specification file clearly documents what is and isn't verified (T1-T4 boundaries), providing honest and useful context for the review.
- **Clean Split**: The separation of `exec`, `spec`, and `proof` files is clean and follows best practices.

## Summary
The verification of `process_manager` is a high-quality **formal model** of the process lifecycle state machine. It successfully proves that if the system adheres to the modeled transition rules, critical properties like PID uniqueness and kernel liveness hold. However, it does not verify the actual implementation code (linked list operations, thread management, memory contexts). It serves as a design validation rather than an implementation verification. To achieve higher assurance, the model should be connected to the implementation via refinement or by verifying the underlying data structures.
