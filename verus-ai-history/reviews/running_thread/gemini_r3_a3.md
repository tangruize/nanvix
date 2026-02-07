# Review: running_thread (gemini_r3_a3)

## Grade: A-

## Status
*   **Passed**: YES
*   **Previous Issues Fixed**: Yes (Mitigated/Documented as per previous cycle)

## Critical Re-Evaluation

I have re-examined the code and proofs with a skeptical eye, specifically looking for regressions or unaddressed edge cases.

### 1. Verification of Fixes/Mitigations
*   **External Mutability**: The `thread_state_mut` function remains `#[verifier::external]`. As noted previously, this is a necessary escape hatch due to current tool limitations regarding `&mut T` return types. The documentation added in the previous cycle ("Trust Boundary") correctly identifies the risks and obligations. The presence of `put_mutex_guard` and `take_mutex_guard` provides the verified path for the most critical state mutations.
*   **Omitted Synchronization**: The omission of `join_cond` is documented. This is a valid scoping decision for this specific module, which focuses on thread state mechanics rather than inter-thread synchronization primitives.
*   **API Strengthening**: The strengthened preconditions on mutex operations (requiring the mutex to be held/not held) correctly model the safety requirements of the kernel, even if they diverge slightly from the "defensive" style of the original implementation.

### 2. Soundness Check
*   The proofs in `running.proof.rs` rely heavily on automated solving (empty bodies). Given the simple nature of the properties (mostly field preservation during struct transformation), this is appropriate and sound.
*   The `lemma_acquire_then_release_restores_mutex_state` proof correctly handles the non-trivial set logic.

### 3. Remaining Risks
*   **Manual Verification Burden**: The correctness of the entire system relies on callers of `thread_state_mut` manually adhering to the `wf()` and identity preservation obligations. This is the primary reason the grade remains A- rather than A.
*   **Cross-Module Integrity**: The `CROSS-MODULE-CHECK` comments are text-only. If `sleeping.rs` or `ready.rs` changes their `from_state` logic, this module's boundary models could become stale without a compiler error.

## Conclusion
The module is verified to a high standard within the constraints of the tools. The specifications are clear, the logical separation is sound, and the known limitations are well-documented. No new issues were found.
