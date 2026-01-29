# Review: vmem (gpt-5.1-codex-max)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Unverified externals unchanged**: `map_kpage` and `copy_to_user_unaligned_unchecked` remain `external_body` with no proof obligations or refinement evidence. Kernel-page mapping semantics and unchecked copy safety are still assumed rather than verified.
- **Capacity truncation persists**: `MAX_USER_PAGES` is still 65,536 (256 MB) while USER_BASE..USER_END spans ~2 GB (~524k pages). New lemma `max_user_pages_sufficient` merely shows the cap exceeds `MEMORY_SIZE`, not that the full user space is modeled. Large legitimate address spaces remain outside the verified envelope.

### Medium
- **Destination physical bounds still omitted**: `copy_to_user_unaligned_unchecked` keeps a postcondition for source physical bounds only; destination bounds are excluded with a rationale about allocator invariants and panics. This leaves a refinement gap versus the implementation, which checks destination frame bounds at runtime.
- **map lacks frame bounds precondition**: No precondition requires `frame_addr` within physical memory; the added rationale defers to allocator invariants but provides no linked proof. Mapped frames could be out-of-range in the spec, weakening downstream guarantees.

### Low
- **Constructor/clone simplification unchanged**: `new`/`clone` still drop kernel mapping parameters and error paths, always returning empty user mappings. The constructor semantics for kernel tables and error propagation remain uncovered for refinement.

## Positive Observations
- Invariants still enforce user-space bounds, page alignment, and uniqueness of user mappings.
- Copy helpers (`copy_from_user_unaligned`, `copy_to_user_unaligned`) maintain region/bounds checks and non-zero size preconditions.
- Mapping operations preserve `mapping_count` and invariant as before.

## Summary
No previously reported issues were fixed. High-risk externals stay unverified, the user-space capacity is still truncated, and the medium-level specification gaps (missing destination physical bounds, missing frame-bound precondition on `map`) remain. Constructors continue to abstract away kernel setup. Verification coverage is unchanged; production confidence still requires addressing these gaps or supplying refinement proofs.
