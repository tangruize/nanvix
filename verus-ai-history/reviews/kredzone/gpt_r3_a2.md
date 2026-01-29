# Review: kredzone (gpt-5.1-codex-max)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **store / load specifications remain purely bounds-only**: Postconditions still only assert index validity; there is no verified linkage between a successful `store` and subsequent `load` returning the stored value or preserving non-interference. The concrete API’s core functional correctness remains entirely trusted, leaving read-after-write and isolation unverified. This was previously flagged and is still unresolved.

### Medium
- **`load_with_ghost` still uses unchecked `assume` for T2**: The wrapper retains the `assume(res.unwrap() == spec_load_result(...))` bridge with no trusted axiom or refinement contract to the volatile implementation. The trust boundary is not minimized or formalized, so the soundness gap called out earlier remains.
- **`create_initial_ghost` still relies on unproven T4**: The ghost initializer continues to assume zeroed kredzone without a proof obligation or enforced call to `init_kredzone()`, so callers can construct inconsistent ghost state if the loader/linker guarantee fails. No mitigation or refinement was added.

### Low
- **`ENTRY_SIZE` fallback on unsupported targets**: The fallback continues to default to 8 bytes with only a warning, potentially mismatching `usize` layout on non-32/64-bit hosts. This unfixed assumption can make proofs vacuously consistent off-target.

## Positive Observations
- No new regressions observed relative to the prior review; documentation of trust assumptions remains clear.

## Summary
Previous concerns were not addressed: functional correctness of `store`/`load` is still unverified beyond bounds, the ghost wrapper still relies on an unchecked assume, zero-init remains an unproven environmental assumption, and the architecture fallback is unchanged. The verification remains incomplete for functional behavior despite intact bounds safety. Overall assurance unchanged from prior review.
