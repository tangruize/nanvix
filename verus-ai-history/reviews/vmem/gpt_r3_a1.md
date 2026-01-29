# Review: vmem (gpt-5.1-codex-max)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- map_kpage / copy_to_user_unaligned_unchecked (external_body): Core mapping and unchecked copy routines are entirely unverified (external_body), so correctness relies on unproved assumptions instead of proofs, leaving kernel-page mapping and unsafe copy semantics unchecked.
- Capacity bound (MAX_USER_PAGES): Model caps user mappings at 65,536 pages (256MB), while USER_BASE..USER_END spans ~2GB (~524k pages). Properties are only proved for this truncated space, so large legitimate address spaces are outside the verified envelope.

### Medium
- copy_to_user_unaligned_unchecked: Postconditions require source physical bounds but omit destination physical bounds, whereas the implementation checks the destination frame is within physical memory. The spec allows states the concrete code would reject/panic on, so refinement is unsound.
- map: Specification does not require the mapped frame to lie within physical memory; implementation relies on FrameAddress invariants and later physical-region checks. Lacking this precondition weakens guarantees about mapped frames used by copy/memset.

### Low
- new/clone simplification: Verified versions drop kernel mapping parameters and error paths, returning empty user mappings unconditionally. Coverage of the constructor semantics (mapping kernel tables, error propagation) is absent, so refinement obligations remain.

## Positive Observations
- Invariant enforces user-space bounds, page alignment, and uniqueness of user mappings.
- copy_from_user_unaligned and copy_to_user_unaligned require regions in the correct address spaces and non-zero size, matching key safety checks from the implementation.
- map/unmap proofs maintain mapping_count consistency and preserve invariants.

## Summary
The verification captures basic user-space mapping invariants and address-space separation but leaves key paths (kernel mapping, unchecked copy) unproved and restricts coverage to a truncated address space. Strengthen specs for physical bounds and frame validity, cover full user-space capacity, and replace external_body stubs with verified contracts or refinement proofs to raise confidence to production quality.
