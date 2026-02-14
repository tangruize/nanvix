# Review: vmem Exec Consistency (claude-opus-4.6)

## Grade: A-

## Verification Status

**PASS**: 30 verified, 0 errors. All specifications, proofs, and exec functions verify successfully.

## Issues Found

### Critical

- None.

### Major

- None.

### Minor

1. **`memset` validation divergence**: The original `memset` does NOT perform explicit `is_user_addr` or alignment checks — it relies on `find_user_frame(dst)` which implicitly requires `dst` to be page-aligned (via `PageAligned<VirtualAddress>` type enforcement) and in user space (only user page tables are searched). The verus version adds explicit `is_user_addr` and `vaddr % PAGE_SIZE != 0` checks before scanning the mapping array. This is **stricter but sound** — the verus version rejects the same inputs as the original, just via explicit checks rather than type-system enforcement. No correctness issue, but worth noting for refinement proofs.

2. **`copy_from_user_unaligned` omits dry-run pattern**: The original performs `copy_from_user_unaligned_impl(true, ...)` then `copy_from_user_unaligned_impl(false, ...)`, where the inner loop calls `find_user_frame()` for each page. The verus version replaces this with a flat validation (size, user_region, kernel_region) and returns `Ok(())`. The precondition `spec_user_region_is_mapped` captures the per-page mapping requirement, which is sound. However, the original's dry-run also validates each page's `find_user_frame` succeeds, checking frame existence per-page; the verus precondition uses `spec_page_is_mapped` which checks vaddr alignment to page boundary but not the exact page table lookup semantics. This is an acceptable abstraction but limits what the postcondition guarantees — in particular, no postcondition about physical bounds of source frames.

3. **`unmap` swap-with-last strategy**: The verus `unmap` uses swap-with-last removal, which changes array ordering. This is a valid implementation for an unordered collection but differs from the original's linked-list removal (which preserves insertion order). The spec uses existential quantifiers over the array, so ordering is irrelevant for correctness. Sound.

4. **`clone` returns `Self` not `Result<Self, Error>`**: The original can fail (via `physical_address()` during page directory setup). The verus version is infallible. This means the verus model cannot express failure modes of cloning. The consistency report documents this. For refinement, an `external_body` wrapper with the original signature would be needed.

### Observations

1. **Constants correctly match**: `USER_BASE = 0x40000000`, `USER_END = 0xC0000000`, `MEMORY_SIZE = 0x10000000` match `config::memory_layout` and `config::kernel::MEMORY_SIZE` respectively.

2. **Uniqueness invariant is strong**: The vmem invariant enforces `vaddr` uniqueness across all valid mappings (`i != j ==> mappings[i].vaddr != mappings[j].vaddr`). This mirrors the original's implicit uniqueness (page tables cannot map the same virtual address twice).

3. **`external_body` functions are well-scoped**: `load`, `pgdir`, `map_kpage`, `kctrl`, `copy_to_user_unaligned_unchecked` are all correctly marked external with appropriate pre/postconditions. The contracts capture the essential safety properties without over-promising about hardware effects.

4. **`uctrl` models validation but not permission update**: The verus version checks user_addr, alignment, and mapping existence, then returns `Ok(())` without tracking permission changes. This is consistent with the documented scope limitation (per-page permissions not tracked). The postcondition `mapping_count == old(self)@.mapping_count` correctly states that uctrl doesn't change the mapping count.

5. **Module documentation is thorough**: The 156-line module header clearly documents all abstraction decisions, scope limitations, and relationship to the original. This is exemplary for a verification model.

## MISMATCH Resolution Assessment

All 19 MISMATCH items from the AST diff were documented as equivalences rather than code-fixed. This is the correct approach given the fundamental modeling decision (array vs. linked-list, usize vs. typed wrappers). The three categories of differences — type simplifications, external bodies, and missing functions — are all well-justified.

## MISSING Function Assessment

The 3 missing functions (`drop`, `lookup_page_table`, `lookup_kernel_page_table`) are correctly omitted:
- `drop`: Resource cleanup, out of scope for memory safety verification.
- `lookup_page_table`/`lookup_kernel_page_table`: Internal helpers for linked-list traversal, abstracted by the array model.

## Summary

The exec consistency fix report correctly identifies that all 22 inconsistencies are intentional structural differences arising from the verification model's design. No code changes were needed because the model faithfully captures the original's safety-critical logic (address validation, bounds checking, uniqueness, mapping lifecycle) while abstracting away hardware details and complex data structures. The verification passes with 30 verified items. The minor issues are all sound abstractions that would only matter for refinement proofs connecting this model to the actual implementation. The documentation quality is high, making the model's limitations transparent.
