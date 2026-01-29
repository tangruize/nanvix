# Review: vmem (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- Coverage gap: Core helpers `lookup_page_table`, `lookup_kernel_page_table`, `copy_to_user_unaligned_unchecked`, and `Drop` are absent in the verified module, so error paths (missing PDE/PT, TLB effects, drop-time unmapping) are unchecked and may hide faults.
- Soundness: `map_kpage`, `load`, and `pgdir` are marked `external_body` with no behavioral specs, leaving kernel mapping, CR3 loading, and directory address correctness unverified in the core module.
- Equivalence: Kernel mappings (page tables and pages) are not modeled at all; verification only tracks user mappings, so kernel-space access control and shared mappings from `clone/new` are unchecked.

### High
- copy_to_user_unaligned: Verification omits physical-bounds checks and panic-on-bad-phys semantics present in `copy_to_user_unaligned_unchecked`; specs allow success even when src/dst are outside physical memory.
- Capacity/coverage mismatch: Verified model caps user mappings at `MAX_USER_PAGES=1024` and returns OutOfMemory, while the implementation uses unbounded linked lists; this can invalidate proofs for realistic workloads.
- Permissions modeling: `AccessPermission` is collapsed to three variants and not tied to PTE bits or caching flags; map/ctrl operations don’t prove that requested permissions/caching are enforced.

### Medium
- kctrl weakening: Verified version does not require the kernel page to exist/mapped before changing permissions, unlike implementation which fails if PDE/PTE is absent; proof permits silently succeeding on unmapped pages.
- is_physical_region constants: Hardcoded `MEMORY_SIZE=0x10000000` instead of reading config, so proofs may not reflect actual platform memory size.
- copy_from_user_unaligned: Verified spec only checks user/kernel region predicates; it does not ensure source page is actually mapped or that the frame lookup succeeds as in the concrete code.

### Low
- Abstraction loss: Caching/TLB flush effects (reload after kernel map) and RWX distinctions (e.g., execute-only) are ignored, so liveness/safety around stale translations are not captured.
- memset semantics: Verified version only checks mapping presence and ignores that hardware call truncates `value` to u8; spec does not state this behavior.

## Positive Observations
- Verified `map`/`unmap` maintain invariants ensuring uniqueness of user virtual addresses, page alignment, and capacity bounds.
- Address/region helpers include overflow checks and enforce disjoint user/kernel ranges.
- Memory copy APIs enforce non-zero sizes and proper user/kernel region separation.

## Summary
The current verification covers only a simplified user-mapping model and leaves major kernel-side behaviors and critical helpers unchecked via omissions and unconstrained `external_body` functions. It weakens core safety conditions (physical bounds, mapping existence, permission fidelity) and hardcodes configuration values, so equivalence and soundness against the concrete OS implementation are not established. Strengthen coverage to include kernel mappings/helpers, replace unconstrained externals with specs or verified bodies, align configuration and capacity with the implementation, and model physical-bound checks and permission semantics to raise confidence.***
