# Review: kredzone (gpt-5.1-codex-max)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Function `store` / `load`**: Specifications only guarantee bounds checking; they do not relate returned value to stored data. As a result, the core API lacks a verified read-after-write or non-interference guarantee—the key functional property is left to trusted assumptions rather than being encoded in postconditions. Suggested Fix: Strengthen the postconditions (or add wrapper specs) so that, under the stated trust assumptions, a successful `store` followed by `load` at the same index is guaranteed to yield the stored value (e.g., by using ghost state and explicitly referencing the last store effect in the ensures).

### Medium
- **Function `load_with_ghost`**: Uses an `assume` to equate the returned value with the ghost state (trust assumption T2) without a formal bridge to the concrete `raw_load`/assembly implementation. This weakens soundness: the property is asserted rather than proved or tied to a verified interface contract. Suggested Fix: Replace the `assume` with a lemma that references a trusted/axiomatized volatile-read contract, or factor the assumption into a small, isolated trusted function with a documented refinement obligation to the assembly, making the trust boundary explicit and minimal.
- **Function `create_initial_ghost`**: Assumes kredzone is zero-initialized (T4) but provides no proof obligation tying this to kernel initialization or start.S; callers could construct inconsistent ghost state if the assumption breaks. Suggested Fix: Require an explicit initialization routine (e.g., verified loop of `store(i, 0)`) before exposing the ghost, or add a proof/axiom tied to the loader/linker contract that justifies zero-initialization.

### Low
- **Constant `ENTRY_SIZE` fallback**: For non-32/64-bit targets, ENTRY_SIZE defaults to 8 with only a warning, which may diverge from `mem::size_of::<usize>()` and the actual kredzone layout. While Nanvix targets x86-32, this fallback can make proofs vacuously consistent yet mismatch real layouts on unusual hosts. Suggested Fix: Replace the fallback with a compile-time error or an explicit `assume` guarded by a proof obligation documenting unsupported targets.

## Positive Observations
- Complete coverage of the original API (`store`, `load`) with verified bounds checks and clear abstraction of trust boundaries.
- Abstract model (`KernelRedZoneView`) captures key algebraic properties (read-after-write, non-interference, commutativity) with supporting lemmas and ghost wrappers.
- Clear documentation of environmental trust assumptions (T1–T4) and separation between verified logic and trusted volatile accesses.

## Summary
Bounds safety is fully verified and the abstract model is well-specified, but functional correctness of the concrete `store`/`load` operations depends on unverified assumptions and an `assume` bridging ghost state to volatile memory. Addressing the high-priority gap requires tying the wrappers’ postconditions to a minimal, explicit trusted contract (or eliminating the `assume`) so read-after-write is backed by a justified refinement to the assembly implementation; medium issues cover zero-init and architectural fallback assumptions. Overall assurance is good for bounds but incomplete for end-to-end functional behavior.
