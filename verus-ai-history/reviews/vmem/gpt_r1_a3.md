# Review: vmem (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- Core helpers still unverified: `lookup_page_table`, `lookup_kernel_page_table`, `copy_to_user_unaligned_unchecked`, and `Drop` remain absent, so PDE/PTE miss paths, TLB effects, and drop-time cleanup are unchecked.
- Kernel-side coverage missing: `map_kpage`, `load`, and `pgdir` stay `external_body` without behavioral specs, and kernel mappings are still not modeled, leaving kernel permissions/sharing and CR3 correctness unproved.

### High
- Permission fidelity unresolved: `AccessPermission` remains decoupled from PTE/caching bits; map/ctrl do not prove requested permissions are enforced.
- Capacity mismatch unchanged: fixed `MAX_USER_PAGES=1024` diverges from unbounded lists in the implementation, so proofs omit realistic workloads.
- Copy preconditions are insufficient: new `spec_user_region_is_mapped` only checks page-aligned offsets; for unaligned regions it is vacuous, so copy_to/from_user can assume mappings without evidence for the first/last pages.

### Medium
- Kernel permission change weakening persists: `kctrl` can succeed without the kernel page being mapped, unlike the concrete implementation that errors on missing PDE/PTE.
- Hardcoded physical size remains: `MEMORY_SIZE=0x10000000` is baked in, not read from configuration, risking mismatch with the actual platform.

### Low
- Hardware effects still abstracted: TLB/cache effects, execute-only distinctions, and reload behavior remain unmodeled.

## Positive Observations
- copy_to_user_unaligned and copy_from_user_unaligned now require mapped user pages (though the predicate is too weak for unaligned regions), improving alignment with the concrete panic-on-missing-frame behavior.
- Existing strengths retained: address/region overflow checks, user/kernel separation, and acknowledgment of memset’s u8 truncation.

## Summary
Some progress was made by adding mapping-existence preconditions to copy routines, but the predicate is too weak for unaligned regions, and major gaps persist: kernel helpers are still unchecked/external, kernel mappings are unmodeled, permissions and capacity remain misaligned with the implementation, and configuration is hardcoded. Verification remains incomplete and not sound relative to the concrete OS behavior; kernel coverage, permission fidelity, capacity alignment, and stronger mapping/physical checks are still required.
