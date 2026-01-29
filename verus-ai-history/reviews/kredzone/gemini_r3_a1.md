# Review: kredzone (gemini-3-pro-preview)

## Grade: B+

## Issues Found

### High
- **Soundness Hole in Ghost State Wrappers**: The `create_initial_ghost` function is a `proof` function that creates a `KernelRedZoneGhost` from scratch. Since it can be called multiple times, it is possible to create multiple independent ghost states (`g1`, `g2`) that both claim to model the single global `kredzone`.
    - **Scenario**: A user could update `kredzone` via `store_with_ghost(..., &mut g1)` (updating real memory and `g1`), leaving `g2` stale. A subsequent call to `load_with_ghost(..., &g2)` would then `assume` the return value matches the stale `g2`, contradicting reality. This allows verifying incorrect code.
    - **Suggested Fix**: `create_initial_ghost` should consume a unique, non-duplicable token (e.g., a singleton `InstanceAuth` created only at kernel boot) to ensure only one ghost model exists. Alternatively, mark the wrappers as `unsafe` or clearly document that the caller must enforce uniqueness (though this weakens the verification guarantee).

### Medium
- **Unjustified `assume` in `load_with_ghost`**: The function contains `assume(res.unwrap() == spec_load_result(ghost.view, index as int))`. While this is intended to bridge the gap between the volatile read and the ghost model (Trust Assumption T2), it is only valid if the ghost model is synchronized. As noted above, this synchronization is not enforced by the type system.
    - **Suggested Fix**: This `assume` is the mechanism of the soundness hole. It should be removed or conditioned on a stronger proof of synchronization (like the unique token mentioned above).

### Low
- **Structural Divergence**: The verified `store` and `load` functions differ from the original source. The original uses `unsafe` blocks directly, while the verified version calls `external_body` wrappers (`raw_store`/`raw_load`).
    - **Impact**: While functionally equivalent for verification purposes, we are technically verifying a modified AST.
    - **Suggested Fix**: Keep as is (standard pattern for Verus), but ensure the `raw_store`/`raw_load` bodies in the verified file (commented out) are kept in sync with the original `unsafe` blocks.

## Positive Observations
- **Verified Bounds Safety**: The most critical property—bounds checking—is fully verified in `store` and `load`. The specs `ensures result.is_ok() ==> spec_is_valid_index(index)` provide strong guarantees.
- **Clear Trust Boundaries**: The documentation explicitly lists Trust Assumptions (T1-T4) and explains why `external_body` is used (volatile operations on extern statics).
- **Abstract Model**: The `KernelRedZoneView` and associated lemmas provide a solid mathematical foundation for reasoning about the component, assuming the single-instance invariant is respected.

## Summary
The verification of `kredzone` is **sound regarding bounds safety** for the base `store` and `load` functions. This is the primary safety property for this component.

However, the **ghost state wrapper layer (`*_with_ghost`) has a soundness issue** allowing the creation of multiple aliased ghost states that can diverge from reality. While the intention (documented in T3/T4) is that these are used linearly in a single-threaded context, the verification specs do not enforce this uniqueness, making the `assume` in `load_with_ghost` potentially unsound in a broader context.

For a kernel primitive, the bounds check verification is the most valuable part and is done correctly. The wrapper layer issues are significant but only affect those attempting to reason about the *content* of the red zone via Verus, not the safety of accessing it.
