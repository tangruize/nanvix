# Review: sleeping_thread (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### Medium
- **Unverified Mutable Access**: `thread_state_mut` is marked `#[verifier::external]`. This function returns a mutable reference to the internal state (`&mut ThreadState`), which allows callers to modify the state without respect to `SleepingThread` invariants (e.g., identity preservation). While this is due to current Verus limitations regarding `&mut T` return types, it creates a hole where invariants could be violated by unverified code.
    - **Location**: `impl SleepingThread` in `sleeping.rs` (exec).
    - **Fix**: Identify the specific fields of `ThreadState` that clients need to modify and provide verified setter methods for them on `SleepingThread`. This would allow `thread_state_mut` to be removed or deprecated.

### Low
- **Omitted Function**: `join_cond()` is omitted from the verification model. While documented as an "opaque sync primitive," its absence means any properties related to condition variable identity or safety are lost.
    - **Location**: `sleeping.rs` (original vs verified).
    - **Fix**: Add a phantom field or abstract model for `Condvar` to track its identity, ensuring it isn't lost or swapped during state transitions.
- **Cross-Module duplication**: `ReadyThread` and `InterruptedThread` are defined locally as boundary models. This creates a maintenance risk where the local definitions might diverge from the actual implementations in `ready.rs` and `interrupted.rs`.
    - **Location**: `sleeping.rs` (exec) and `sleeping.spec.rs` (spec).
    - **Fix**: Ensure the `CROSS-MODULE-CHECK` obligations documented in the code are tracked in a central verification roadmap or CI check.

## Positive Observations
- **Strong Invariant Preservation**: The proofs meticulously verify that key properties—thread identity, mutex accounting, drop safety, and well-formedness—are preserved across all state transitions (`wakeup`, `interrupt`).
- **Clear Boundary Documentation**: The code contains excellent documentation regarding trust boundaries and cross-module obligations (e.g., `CROSS-MODULE-CHECK` comments), making it clear what assumptions are made about external modules.
- **Complete Spec Coverage**: All functional methods (`from_state`, `wakeup`, `interrupt`, `id`, `alarm`) have comprehensive specifications that capture their behavior precisely.
- **Clean Split**: The separation into `exec`, `spec`, and `proof` files is clean and follows project conventions.

## Summary
The verification of `sleeping_thread` is high quality, providing strong guarantees about state transitions and invariant preservation. The coverage is nearly complete, with the only significant exception being `thread_state_mut` due to language limitations. The use of boundary models for `ReadyThread` and `InterruptedThread` is handled well with clear documentation of the required cross-module checks. The grade is an A-, primarily deducting for the `thread_state_mut` escape hatch which weakens the encapsulation of the verified type.
