# Review: ustack (gemini-3-pro-preview)

## Grade: A+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- None.

### Low
- **Redundant Constants**: `PAGE_SIZE` and `USER_STACK_SIZE` are redefined in `verus/ustack.rs` because Verus cannot easily import constants from non-verified crates. While they currently match the source (`src/libs/config/src/lib.rs` and `constants.rs`), they could drift out of sync.
  - *Fix*: Ensure CI pipelines verify these values match, or (long-term) move configuration to a shared module accessible by both.
- **Missing Debug Verification**: The `fmt::Debug` implementation is not verified. This is standard practice but technically a gap.
  - *Fix*: None required; low value to verify debug formatting.

## Positive Observations
- **API Equivalence Model**: The verified module explicitly models the relationship between the original `PageAligned<VirtualAddress>` type and the verified `usize` + predicates approach. The lemmas `lemma_page_aligned_equivalence` and `lemma_postcondition_models_page_aligned` provide a strong formal link to the original code structure.
- **Documentation Fixes**: The verification uncovered and fixed a contradiction in the original documentation regarding `base` vs `top` addresses (stack growth direction). The verified documentation is precise and implementation-consistent.
- **Strong Invariants**: The `inv()` predicate captures essential properties like page alignment, size consistency, and page contiguity (`pages_are_contiguous`), which goes beyond simple bounds checking.
- **Useful Extensions**: The verified module adds helpful methods (`contains`, `page_index`, `has_room`) that are logically sound and useful for clients.
- **Liveness Proofs**: The `new` function proves construction success under valid inputs, replacing the original's type-system infallibility with a proven runtime check.

## Summary
The verification of `ustack` is exemplary. It provides full functional coverage of the original code while strengthening the formal guarantees. The decision to model `PageAligned` types via predicates is well-justified and formally proven equivalent. The inclusion of auxiliary properties (like page contiguity) and the correction of original documentation errors demonstrate a deep engagement with the correctness of the component. The only minor issue is the duplicate constant definition, which is a known limitation of the current toolchain structure.
