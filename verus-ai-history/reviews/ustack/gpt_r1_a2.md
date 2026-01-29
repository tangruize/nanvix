# Review: ustack (gpt-5.1-codex-max)

## Grade: B-

## Issues Found

### High
- **API parity still broken (PageAligned & base/top semantics):** The verified version keeps `usize` for `base()`/`top()` and documents base as lowest/top as highest, while the shipped kernel uses `PageAligned<VirtualAddress>` and documents the inverse (base highest, top lowest). No wrapper or equivalence proof is provided, so callers proven against the kernel API/semantics are not covered. The prior request to retain or mirror the original `PageAligned` interface (or prove equivalence) remains unaddressed.
- **Configuration linkage still missing:** Constants (`PAGE_SIZE`, `USER_STACK_SIZE`, `USER_STACK_PAGES`) remain locally duplicated with comments about matching kernel values but no linkage or assertion against `config::memory_layout`/`arch` definitions. If the kernel configuration changes, the proof can pass while diverging from the built kernel. The prior request to import or assert equality to the canonical constants is not implemented.

### Medium
- None.

### Low
- None.

## Positive Observations
- Invariant proof coverage (alignment, size, ordering, contiguity, overflow) remains intact.
- Helper methods (`contains`, `page_index`, `initial_sp`, `has_room`) are fully specified and verified.

## Summary
The submission did not resolve the two outstanding high-severity items: API/type/semantic parity with the kernel and linkage to canonical configuration constants. Without those, the verified module can still diverge from the shipped interface or configuration. Implement the requested parity fixes (or formal equivalence) and tie proofs to the kernel constants to complete the verification story.
