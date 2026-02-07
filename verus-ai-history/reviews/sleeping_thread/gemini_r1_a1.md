# Review: sleeping_thread (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- None.

### Low
- **Function Omission**: `join_cond()` is present in the original source but omitted from the verified model.
  - **Location**: `src/kernel/src/pm/thread/sleeping.rs` vs `verus/split/kernel/pm/thread/sleeping.rs`
  - **Description**: The function returns a `Condvar`. The verified code comments explain this is intentional because `Condvar` is an opaque sync primitive that Verus cannot model.
  - **Suggested Fix**: Keep as is, the documentation justifies the omission.

- **External Function**: `thread_state_mut()` is marked `#[verifier::external]`.
  - **Location**: `verus/split/kernel/pm/thread/sleeping.rs`
  - **Description**: Verus does not yet support `&mut T` return types. This creates a trust boundary where callers must manually uphold invariants.
  - **Suggested Fix**: This is a known tool limitation. The comments correctly identify the obligations for callers.

## Positive Observations
- **Strong Specification**: The `wf()` predicate correctly enforces well-formedness of the underlying state and the alarm.
- **State Transition Verification**: The `wakeup()` and `interrupt()` transitions are rigorously verified to preserve thread identity, mutex accounting, and drop safety.
- **Cross-Module Obligations**: The code explicitly documents "CROSS-MODULE-CHECK" obligations for the boundary models (`ReadyThread`, `InterruptedThread`), which is excellent for maintaining integrity in a split verification setup.
- **Clean Split**: The separation of executable code, specifications, and proofs into `.rs`, `.spec.rs`, and `.proof.rs` is clean and follows best practices.
- **Comprehensive Lemmas**: `sleeping.proof.rs` provides a good set of lemmas covering construction, identity correctness, and state transitions.

## Summary
The verification of `sleeping_thread` is high quality. It covers the core state transitions of a sleeping thread (waking up and interruption) with strong guarantees about identity and resource accounting preservation. The deviations from the original code (`join_cond` omission and `thread_state_mut` external attribute) are well-justified by current tool limitations and are properly documented. The use of boundary models for `ReadyThread` and `InterruptedThread` with explicit cross-check reminders is a robust approach for modular verification.
