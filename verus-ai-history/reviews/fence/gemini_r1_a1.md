# Review: fence (gemini-3-pro-preview)

## Grade: B+

## Issues Found

### High
- **Ineffective Specification for `wait`**:
    - **Location**: `wait` function in `verus/split/kernel/pm/sync/fence.rs`
    - **Description**: The `wait` function requires `self.spec_is_satisfied()` as a precondition. This makes the function tautological (it requires the postcondition to be true before calling). While this is documented as a limitation of the sequential model, it renders the spec unusable for verifying any client code that actually relies on `wait` to block until completion. A client calling `wait` cannot prove the precondition unless it already knows the fence is signaled, defeating the purpose of waiting.
    - **Suggested Fix**: There is no easy fix within a sequential model. A potential approach is to mark `wait` as `external_body` and provide a trusted specification that models the blocking behavior (e.g., using a liveness property or a token-based concurrency model if Verus extensions allow), or simply acknowledge that `wait` cannot be meaningfully verified in this scope and remove the pretension that it is "verified" beyond a no-op check.

### Low
- **API Divergence in `signal`**:
    - **Location**: `signal` function in `verus/split/kernel/pm/sync/fence.rs`
    - **Description**: The verified `signal` function takes `&mut self` (exclusive reference) and enforces `count < total` (no over-signaling). The original runtime implementation takes `&self` (shared reference via atomics) and allows over-signaling (via `fetch_add`).
    - **Suggested Fix**: This is likely a design choice to enforce a strict protocol. If the runtime behavior of over-signaling is intended to be supported, the spec should be relaxed. If the strict protocol is desired, no fix is needed, but the divergence prevents verifying scenarios where over-signaling might occur benignly. The `&mut self` restriction limits verification to non-concurrent contexts.

## Positive Observations
- **Excellent Documentation**: The "Verification Gap" and "Trust Boundaries" sections in the documentation are exemplary. They clearly articulate exactly what is and isn't verified (sequential state machine vs. concurrent execution), which prevents false confidence in the verification results.
- **Strong Protocol Proofs**: The proof module contains valuable lemmas proving the correctness of the signaling protocol, such as `lemma_signal_commutativity` and `lemma_total_signals_satisfies`. These provide high confidence that the underlying counting logic is correct.
- **Clean Split**: The separation of executable code, specifications, and proofs into `fence.rs`, `fence.spec.rs`, and `fence.proof.rs` is clean and maintainable.
- **Overflow Safety**: The model correctly identifies that the protocol (`count < total` precondition on `signal`) is sufficient to prevent integer overflow on `count`, simplifying the proof obligations.

## Summary
The `fence` verification is a high-quality model of the component's **internal state machine logic**, but it does not verify its **synchronization semantics**. The verification proves that if you signal `total` times, the fence becomes satisfied, and that you cannot signal more than `total` times. However, the `wait` function's specification is vacuous for its primary use case (blocking/waiting), as it requires the caller to prove satisfaction beforehand. The code earns a **B+** because the logic that *is* verified is done well and the limitations are documented with exceptional clarity, even though the scope excludes the component's primary runtime behavior (concurrency).
