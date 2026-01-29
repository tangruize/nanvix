# Review: ustack (gpt-5.1-codex-max)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Configuration Linkage Still Manual** – Constants `PAGE_SIZE`, `USER_STACK_SIZE`, and `USER_STACK_PAGES` remain duplicated with only comments suggesting CI checks. There is still no enforced linkage to the kernel definitions (e.g., config::memory_layout, arch), so a config drift would leave proofs unsound while code compiles. *Suggested Fix:* Add an automated check/CI step (or code generation) that compares these constants against the authoritative kernel values.

### Low
- None (previous concern about type-level alignment is addressed via `PageAlignedAddr`/aligned accessors).

## Positive Observations
- The prior API-equivalence gap is closed with the new `PageAlignedAddr` wrapper and aligned accessors mirroring the original `PageAligned<VirtualAddress>` API.
- No `assume`/`external_body` remain; proofs cover constructors, accessors, and helpers with explicit invariants for alignment, contiguity, and overflow avoidance.
- Documentation now clearly states base/top semantics consistent with the implementation and records the CI linkage requirement.

## Summary
The verification is now closer to the runtime API, addressing the type-level alignment concern. The remaining gap is the still-unenforced linkage of duplicated constants to kernel configuration; automating that check would close the last substantive risk.
