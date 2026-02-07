# Review: interrupted (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Low
- **Duplicate Definition (ReadyThread):** The `ReadyThread` struct is defined locally in `interrupted.rs` as a mock for the real `ReadyThread` in `ready.rs`. While necessary for split verification, this creates a maintenance risk if the real `ReadyThread` definition changes (e.g. adding fields).
- **External Function (`thread_state_mut`):** The `thread_state_mut` function is marked `#[verifier::external]` due to Verus limitations with `&mut T` return types. This creates a small soundness gap where callers could theoretically violate invariants, though the function documentation clearly warns about this trust boundary.

## Positive Observations
- **Excellent Documentation:** The file headers and comments provide exceptional clarity on trust boundaries, modeling decisions (like `InterruptReason` abstraction), and limitations.
- **Strong Proofs:** The proofs comprehensively cover state transitions, identity preservation, and well-formedness. The `resume` transition is rigorously verified.
- **Clean Separation:** The split into `exec`, `spec`, and `proof` files is clean and follows the project's verification patterns well.
- **Correct Modeling:** The abstraction of `InterruptReason` to `int` and `Box<ThreadState>` to `ThreadState` is sound and well-justified.

## Summary
The verification of `interrupted.rs` is of high quality. It successfully captures the core logic of the interrupted thread state and its transition back to a ready thread. The use of a local `ReadyThread` mock is a pragmatic choice for split verification, and the trust boundaries are explicitly documented. The verification is sound within the stated scope.
