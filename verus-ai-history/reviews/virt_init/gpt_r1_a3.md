# Review: virt_init (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Input handling and error semantics still diverge from original** (exec `init`, `mod.rs`): the verified entry still assumes pre-sorted, non-overlapping, aligned regions and returns `(Vec<usize>, Ghost<...>)` instead of `Result<LinkedList<...>, Error>`. Merge+sort and all error paths (overlap/alignment/invalid MMIO) remain unmodeled, so equivalence for unsorted or invalid inputs is not established. **Suggested fix:** verify the merge+sort step and keep a `Result` return with specs for each error condition, or provide a refinement proof from the original API.
- **Mapping side effects/flags remain unmodeled** (exec `init`, `mod.rs`): the new ghost `PageMapping` records (vaddr,paddr,region_start,is_mmio) but still does not model `map()` side effects or permission/caching flags, nor link to concrete page-table contents. Correctness of the actual page tables is still unproven. **Suggested fix:** model a ghost page-table map (vaddr → (paddr, flags)) updated per iteration and add postconditions that the map matches the original `map()` parameters.
- **Memory-boundary semantics still strengthened** (exec `init`, `mod.rs`): the precondition `regions[i].spec_end() <= INIT_MEMORY_SIZE` replaces the original early-break behavior at `MEMORY_SIZE - PAGE_SIZE`, so partial mapping for oversized regions is not captured. **Suggested fix:** model the break condition or specify partial mapping semantics instead of strengthening inputs.

### Medium
- **MMIO safety precondition still missing** (exec `get_mmio_paddr`, `mod.rs`): the code now claims MMIO translation is infallible if regions are page-aligned, but the original `from_mmio_address` is `unsafe` with a validity precondition (no checks). The spec still lacks a requirement that `region_start` is a valid MMIO address. **Suggested fix:** add an explicit precondition for MMIO validity or model the unsafe requirement in the spec.
- **Coverage gap for `manager` remains** (module list / documentation, `mod.rs`): the `manager` sub-module is still out-of-scope, which violates the coverage criterion for the original `virt` module. **Suggested fix:** add a verified model for `manager` or explicitly narrow the verification scope and adjust coverage claims.

### Low
- None.

## Positive Observations
- The ghost `PageMapping` return and postconditions improve traceability of per-page paddr computation and alignment.
- Spec/proof separation remains clean and readable.

## Summary
The latest changes improve mapping traceability but do not fix the core semantic gaps: preprocessing/error handling is still omitted, mapping side effects are not modeled, and memory-boundary behavior is strengthened. Verification remains partial and not semantically equivalent to the original for all inputs.
