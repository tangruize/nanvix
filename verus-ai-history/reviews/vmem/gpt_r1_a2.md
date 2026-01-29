# Review: vmem (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- Coverage gap persists: key helpers `lookup_page_table`, `lookup_kernel_page_table`, `copy_to_user_unaligned_unchecked`, and `Drop` remain absent from the verified module, so error paths (missing PDE/PT, TLB effects, drop-time unmapping) are still unverified.
- Unconstrained externals unchanged: `map_kpage`, `load`, and `pgdir` remain `external_body` with no behavioral specs, leaving kernel mapping effects, CR3 loading, and directory address correctness unchecked.
- Kernel mappings still not modeled: verification tracks only user mappings, so kernel-space permissions/sharing from `new`/`clone` and kernel mapping invariants are unproved.

### High
- Permissions remain abstract: `AccessPermission` is decoupled from PTE bits/caching flags and map/ctrl operations do not prove requested permissions or caching are enforced.
- Capacity mismatch unchanged: fixed `MAX_USER_PAGES=1024` with OutOfMemory diverges from unbounded list implementation, so proofs do not cover realistic workloads.
- Copy to/from user ignore mapping existence: specs accept success without showing destination/user pages exist (copy_to_user) or source mappings exist (copy_from_user); page-by-page frame lookup and related failures in the concrete code remain unmodeled despite added region checks.

### Medium
- Kernel permission change weakening: `kctrl` still succeeds without checking the kernel page is mapped, unlike the concrete implementation that fails when PDE/PTE is absent.
- Hardcoded physical size: `is_physical_region` keeps `MEMORY_SIZE=0x10000000` baked in instead of reading configuration, so proofs may mismatch actual platform memory.
- Copy_from_user physical coverage incomplete: source physical bounds and per-page checks are not modeled; only destination physical bounds are constrained.

### Low
- Hardware effects abstraction unchanged: caching/TLB flush effects, execute-only distinctions, and reload behavior remain unmodeled.

## Positive Observations
- Address/region helpers retain overflow checks and user/kernel separation, and copy_to_user now includes a physical-bounds check on the source region.
- memset docs now note the u8 truncation behavior of the underlying implementation.

## Summary
Most prior critical gaps remain: kernel helpers are unverified or external, kernel mappings are unmodeled, and permission/capacity mismatches persist. Copy operations gained a source physical check and memset documentation improved, but mapping existence and configuration fidelity still diverge from the concrete implementation. Verification remains incomplete; strengthen kernel coverage, align configuration/capacity with the implementation, and model mapping existence and permission enforcement to reach soundness.
