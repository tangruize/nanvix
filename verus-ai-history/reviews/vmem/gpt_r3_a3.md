# Review: vmem (gpt-5.1-codex-max)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Unverified externals persist**: `map_kpage`, `pgdir`, `load`, and `copy_to_user_unaligned_unchecked` remain `external_body` with no refinement evidence. Kernel-page mapping semantics and unchecked copy safety are still assumed, not proved.
- **Capacity truncation unchanged**: `MAX_USER_PAGES` stays at 65,536 (256 MB) while USER_BASE..USER_END spans ~2 GB (~524k pages). The new lemma `max_user_pages_sufficient` only shows the cap ≥ `MEMORY_SIZE`, not that the full user space is modeled. Large legitimate user spaces remain outside the verified envelope.

### Medium
- **Destination physical bounds still missing**: `copy_to_user_unaligned_unchecked` postconditions cover only source physical bounds; destination physical bounds are omitted, justified by allocator invariants and panic behavior. This remains a refinement gap versus the implementation, which checks destination frame bounds at runtime.
- **map lacks frame-bound precondition**: The spec still does not require `frame_addr` < `MEMORY_SIZE`. Rationale points to allocator guarantees, but no linked proof ties `FrameAddress` to physical bounds, leaving mapped frames potentially out-of-range in the spec.

### Low
- **Constructor/clone simplification remains**: `new`/`clone` still drop kernel mapping parameters and error paths, always returning empty user mappings. Kernel table setup and failure modes remain unmodeled for refinement.

## Positive Observations
- Invariants still enforce user-space bounds, page alignment, and uniqueness of user mappings.
- Copy helpers (`copy_from_user_unaligned`, `copy_to_user_unaligned`) retain region checks and non-zero size preconditions.
- Mapping operations continue to preserve `mapping_count` and the invariant.

## Summary
No previously reported issues were fixed. High-risk externals are still unverified, the user-space capacity remains truncated, and medium-level gaps (missing destination physical bounds in unchecked copy; missing frame-bound precondition in `map`) persist. Constructors continue to abstract away kernel setup and errors. Verification coverage is effectively unchanged; production confidence still requires addressing these gaps or supplying refinement proofs.
