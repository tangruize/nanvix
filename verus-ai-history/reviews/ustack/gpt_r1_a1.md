# Review: ustack (gpt-5.1-codex-max)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Equivalence / API parity (new, base, top):** Verified version changes signatures from `PageAligned<VirtualAddress>` to raw `usize` and reinterprets semantics (base as lowest address, top as highest) with preconditions instead of type-level guarantees. This drops compile-time alignment/upper-bound guarantees and diverges from the original API contract, so callers proved against the original may not be covered. *Suggested Fix:* Keep `PageAligned<VirtualAddress>` for `base()`/`top()` and an infallible `new(PageAligned<VirtualAddress>)`, or add wrappers/specs showing equivalence to the original interface while proving the conversion is safe and semantics unchanged.

### Medium
- **Specification linkage to real constants:** The verified module hardcodes `USER_STACK_SIZE`, `PAGE_SIZE`, and derived `USER_STACK_PAGES` locally instead of referencing `config::memory_layout::USER_STACK_SIZE` and the platform page size. If configuration changes, the proof may pass while diverging from the kernel build. *Suggested Fix:* Import and use the canonical constants (or assert equality) so proofs track the configured sizes.

### Low
- None.

## Positive Observations
- Core functions are covered with explicit invariants: alignment, no overflow, contiguity, and top/base ordering are proved.
- Additional helpers (`contains`, `page_index`, `has_room`) are fully specified and verified for bounds/alignment.
- No `assume` or `external_body` are present in the core logic.

## Summary
The verification proves useful memory-safety properties but diverges from the original API and configuration sources. Aligning the verified interface and constants with the shipped kernel definitions would restore parity and prevent future drift. Once those adjustments are made, the module should provide strong, configuration-faithful guarantees.
