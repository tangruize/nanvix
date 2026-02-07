# Review: condvar (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **Abstraction Gap**: The verified code defines a shadow `Condvar` struct (using a ghost sequence and concrete length) rather than verifying the original `Condvar` struct (which uses `Arc<CondvarInner>` and `RefCell<LinkedList>`). While this is explicitly documented as a "specification model," it means the verification proves the correctness of the queue protocol logic but not that the actual runtime implementation (using `LinkedList`) faithfully implements this protocol.
- **Search Logic Externalized**: The `try_remove_by_pid` and `try_remove_by_tid` functions rely on a caller-provided concrete `has_match` boolean and ghost index. The original implementation computes these internally via `iter().position()`. This shifts the correctness burden of the search logic to the caller of the model, which is currently unverified.

## Positive Observations
- **Exemplary Documentation**: The module-level documentation is outstanding. It clearly defines the verification scope, maps original APIs to model functions, explicitly lists API divergences, and details all trust assumptions (T1-T6). This makes the verification's guarantees and limitations transparent.
- **Protocol Verification**: The `wait` protocol verification is rigorous. Specifically, `lemma_wait_cleanup_restores_state` proves that the "enqueue then remove-on-failure" pattern (modeling `wait` with a failed `sleep`) correctly restores the original queue state, which is a subtle and critical correctness property.
- **Invariant Modeling**: The well-formedness predicate (`wf`) captures key invariants like queue element uniqueness (`spec_all_unique`) and the safety constraint that the kernel process cannot sleep (`spec_no_kernel_pid`).
- **Soundness**: The verification avoids unjustified `assume` or `external_body` blocks in the core logic. The use of ghost state is appropriate for modeling the abstract queue.

## Summary
The `condvar` verification is high-quality work. It successfully navigates the difficulty of verifying `Arc<RefCell<LinkedList>>` by creating a verified model that proves the correctness of the underlying synchronization protocol. The coverage of the state machine transitions—including the complex failure cleanup in `wait()`—is comprehensive. The definitions are sound, the proofs are complete, and the documentation sets a high standard for clarity regarding scope and assumptions.
