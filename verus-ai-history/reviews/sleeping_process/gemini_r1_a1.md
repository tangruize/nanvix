# Review: sleeping_process (gemini-3-pro-preview)

## Grade: C

## Issues Found

### Critical
- **Implementation Logic Replaced by Model**: The verified `exec` code replaces the actual thread storage (`NonEmptyVecDeque<SleepingThread>`, `Option<NonEmptyVecDeque<ZombieThread>>`) with `Ghost<Seq<int>>` and a `u64` counter. The actual runtime logic for managing these collections (insertion, removal, iteration) is completely removed.
    - **Description**: The verification proves the correctness of an abstract state machine but does not verify that the original Rust code correctly implements this machine. For example, a bug in the original `wakeup` that removed the wrong thread or failed to remove it would not be detected, because the verified `wakeup` function uses ghost operations on the model rather than `VecDeque` operations.
    - **Suggested Fix**: To verify the implementation, the code should use a verified or trusted wrapper for `NonEmptyVecDeque` (e.g., `Vec<T>`) and store actual thread objects (or trusted handles). The `exec` code should perform the actual list manipulations (e.g., `remove`, `push_back`) and prove they satisfy the model.

### High
- **Runtime Search Logic Missing**: `find_thread` and `find_thread_mut` are reduced to `Ghost` stubs that perform no runtime work.
    - **Description**: The original code iterates through the lists to find a thread. The verified code returns a `Ghost<Option<int>>` immediately. This means the search algorithm itself is unverified, and the verified file cannot be used as a replacement for the original (it would break runtime callers expecting a `ThreadRef`).
    - **Suggested Fix**: Implement the search logic (e.g., using a loop or iterator) in the verified `exec` code and prove it matches `spec_find_thread`.

### Medium
- **Oracle-Driven Control Flow**: `wakeup` and `wakeup_alarm` rely on oracle parameters (`found`, `has_expired`) to dictate control flow.
    - **Description**: The decision logic (checking if a thread exists, checking if an alarm is expired) is abstracted away. While the `requires` clauses constrain these oracles to be consistent with the ghost state, the actual calculation of these values (e.g., the time comparison `now >= alarm`) is not verified.
    - **Suggested Fix**: For `wakeup`, if the list were physical, the return value of the search would determine `found`. For `wakeup_alarm`, pass `now` and model the alarm check in the spec (if possible) to prove the partition is correct based on time, rather than just accepting a partition oracle.

### Low
- **Signature Mismatch**: Verified functions use `Ghost` types in signatures where original functions use concrete types.
    - **Description**: `wakeup` takes `Ghost<int>` instead of `ThreadIdentifier`. `find_thread` returns `Ghost`. `RunnableProcess` is a ghost wrapper. This confirms the code is a model, not a drop-in verified replacement.
    - **Suggested Fix**: Use concrete types (e.g., `u64` or a wrapper for `ThreadIdentifier`) in public signatures to allow the verified code to potentially link with the rest of the kernel.

## Positive Observations
- **Strong Model Invariants**: The specification correctly captures the essential invariants of the process state: non-empty sleeping threads, disjointness of lists, and uniqueness of thread IDs.
- **Clean Partition Modeling**: The use of oracles in `wakeup_alarm` to model the partitioning of threads (expired vs. remaining) effectively captures the structural requirements (conservation, stability) of the operation, even if the time logic is elided.
- **Good Split**: The separation of specifications (`sleeping.spec.rs`), proofs (`sleeping.proof.rs`), and executable model (`sleeping.rs`) is clean and readable.

## Summary
The verification of `sleeping_process` is currently a **high-level model verification**, not an implementation verification. It successfully proves that the abstract state machine design is sound (threads aren't lost, transitions are valid). However, because it abstracts away the physical data structures (`VecDeque`) and their manipulation, it provides minimal assurance about the correctness of the actual `src/kernel/.../sleeping.rs` code. It cannot detect bugs in the collection handling or search logic. To achieve high-assurance verification, the physical data structures must be brought into the verified scope.
