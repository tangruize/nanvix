# Review: virt_init (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **init still diverges from original input handling and error semantics** (exec `init`, `mod.rs`): the verified function still accepts only pre-sorted, non-overlapping regions and returns `Vec<usize>` instead of `Result<LinkedList<...>, Error>`, so merge+sort and error paths (alignment/overlap/MMIO failures) are not modeled. Documentation claiming a pre-processing step does not establish equivalence for unsorted/invalid inputs. **Suggested fix:** verify the merge+sort step and keep a `Result` return with specs for each error condition, or provide a refinement proof from the original API.
- **Mapping side effects remain unmodeled** (exec `init`, `mod.rs` / spec): the new ghost `mappings` sequence only records `(vaddr, paddr, is_mmio)` and is not tied to page-table contents or permission/caching flags; no postcondition exposes the mapping correctness. Thus “map() is called with correct paddr/flags for every page” is still unproven. **Suggested fix:** model a ghost page-table map (vaddr → (paddr, flags)) updated per iteration and add postconditions that the map matches intended semantics.
- **Memory-boundary behavior still strengthened** (exec `init`, `mod.rs`): the precondition `regions[i].spec_end() <= INIT_MEMORY_SIZE` replaces the original early-break behavior at `MEMORY_SIZE - PAGE_SIZE`, so partial mapping for oversized regions is not captured. **Suggested fix:** model the break condition or specify partial mapping semantics instead of strengthening inputs.

### Medium
- **External bodies still assume infallible MMIO translation** (exec `get_mmio_paddr`, `mod.rs`): the new alignment postcondition is still an unproven assumption; the original `from_mmio_address` can fail and this is not modeled. `PageTableStorage::deref_len/deref_mut_len` also remain `external_body` without justification beyond length. **Suggested fix:** model MMIO translation as `Result` with explicit preconditions, or justify the hardware invariants and error impossibility in specs.
- **Coverage gap for `manager` remains** (module list / documentation, `mod.rs`): the module is still excluded and only documented as out-of-scope, which does not satisfy the stated coverage criterion for the original module. **Suggested fix:** add a verified model for `manager` or explicitly scope the verification artifact and adjust the coverage claim.

### Low
- None.

## Positive Observations
- Added ghost `mappings` tracker improves internal evidence of page coverage and identity mapping for non-MMIO pages.
- MMIO alignment is now explicitly stated, clarifying intended hardware invariants.
- Spec/proof separation remains clean and readable.

## Summary
Some documentation and ghost tracking were added, but the major semantic gaps remain: missing preprocessing/error handling, unmodeled mapping side effects, and strengthened memory-boundary behavior. Verification is still partial and not semantically equivalent to the original for all inputs.
