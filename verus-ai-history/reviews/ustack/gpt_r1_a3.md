# Review: ustack (gpt-5.1-codex-max)

## Grade: B-

## Issues Found

### High
- **API/type parity not fixed (PageAligned + semantics):** The verified module still exposes `usize` for `base()`/`top()` and keeps the “base lowest / top highest” semantics. The kernel API remains `PageAligned<VirtualAddress>` with the opposite documented semantics. The new “equivalence” lemmas only restate the alignment predicate and do not provide an adapter or proof of behavioral equivalence to the shipped API. Callers proven against the kernel API are still uncovered.
- **Configuration linkage still absent:** Constants (`PAGE_SIZE`, `USER_STACK_SIZE`, `USER_STACK_PAGES`) are still duplicated locally. The new lemma only proves self-consistency; there is still no linkage or assertion to `config::memory_layout`/`arch` values. Proofs can diverge from the built kernel if configuration changes.

### Medium
- None.

### Low
- None.

## Positive Observations
- Proof coverage of alignment, ordering, contiguity, and overflow remains intact.
- Added notes about the documentation inconsistency and a small equivalence model, though it does not yet bridge to the kernel API.

## Summary
The two high-severity items from the prior review remain unresolved: the verified API still diverges from the kernel’s `PageAligned` interface/semantics, and the constants remain locally duplicated without binding to the canonical configuration. These gaps leave the verification out of sync with the shipped interface and build-time configuration. Implement an actual API bridge/equivalence and tie constants to the kernel definitions (or assert equality in CI/proofs) to complete the verification.
