# Review: mutex (gemini-3-pro-preview)

## Grade: B-

## Issues Found

### High
- **Sequential Model Divergence**: The verification models `Mutex` using `&mut self` and a plain `bool`, whereas the original implementation uses `&self`, `Arc`, and `AtomicBool`. This means the verification proves state machine correctness in a sequential context but fails to model the actual concurrency, atomicity, or memory ordering that are central to a mutex's correctness.
- **Unverified Blocking Logic**: The verified `lock()` function requires `spec_is_unlocked()` as a precondition, effectively assuming the lock is free. This completely bypasses the core logic of the original `lock()` function: the loop and `Condvar::wait` call that handle contention. The blocking behavior is unverified.

### Medium
- **RAII/Drop Divergence**: The verified code replaces the `MutexGuard` RAII pattern with a manual `unlock()` function requiring a tracked `MutexToken`. While this is a standard Verus pattern for linear resources, it represents a significant structural divergence from the ergonomic, safe Rust API used in the kernel.

### Low
- **Missing API Surface**: The `reference_count()` method and `fmt::Debug` implementation for `MutexGuard` are missing from the verified model.

## Positive Observations
- **Clean Split**: The separation into `mutex.rs` (exec), `mutex.spec.rs` (spec), and `mutex.proof.rs` (proof) is clean and follows project conventions.
- **Strong State Machine Proofs**: The sequential state machine is rigorously verified, including properties like "lock-then-unlock restores state" and "tokens are instance-bound" (via ghost IDs).
- **Soundness**: The core module avoids `external_body` and unjustified `assume`, ensuring the sequential model is sound within its own definitions.
- **Well-Formedness Invariants**: The `wf()` predicate correctly enforces the biconditional relationship between the locked state and token issuance.

## Summary
The current verification provides a solid proof of the mutex *protocol* (locking a free mutex produces a token; unlocking consumes it) but does not verify the *implementation* (atomics, sleeping, weak references). The model simplifies the problem by requiring `&mut self` and assuming no contention, which excludes the most critical and complex aspects of a kernel mutex. It is a good starting point but essentially verifies a "RefCell-like" object rather than a concurrent mutex.
