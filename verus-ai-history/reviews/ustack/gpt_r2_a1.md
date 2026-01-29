# Review: ustack (gpt-5.1-codex-max)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location: Constants (PAGE_SIZE, USER_STACK_SIZE, USER_STACK_PAGES)** – The verification duplicates kernel configuration constants instead of importing them, and there is no proof or automated check tying these values to `config::memory_layout`/`arch` definitions. A configuration drift would make the proof irrelevant while code changes still compile. *Suggested Fix:* Add a CI/assertion step that compares the duplicated constants against the kernel config (or generate them from the source definitions) so proofs stay synchronized with builds.

### Low
- **Location: API Equivalence (base/top return types)** – Verified functions return raw `usize` with postconditions instead of `PageAligned<VirtualAddress>`, so the type-level guarantee from the kernel API is modeled only in specs and not in the executable type system. Downstream code in the verified module is safe via invariants, but callers outside Verus lose the static wrapper the original provides. *Suggested Fix:* Mirror the `PageAligned` wrapper in the verified code (or expose an abstract newtype with alignment invariant) to preserve the original API’s type-level contract.

## Positive Observations
- All original functions (`new`, `size`, `base`, `top`) are covered with proofs of alignment, size constancy, and overflow avoidance; additional helpers (contains, page_index, has_room, initial_sp) strengthen safety reasoning.
- Invariants establish contiguity, alignment, positive size, and top computation; verification passes with no `assume`/`external_body` in core logic.
- Documentation clarifies the base/top semantic inconsistency present in the original comments and aligns proofs with the implementation behavior.

## Summary
Verification is strong on local memory-safety properties (alignment, contiguity, overflow checks) and covers the original API surface with added useful helpers. The main risk is reliance on manually duplicated constants; a mismatch with kernel configuration would silently invalidate proofs. Addressing the constant-linkage gap and restoring a type-level alignment wrapper would bring the verification to parity with the runtime API.
