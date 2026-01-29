# Review: manager (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- Coverage gap: Missing `init`, `get`, `get_mut`, `alloc_upages`, `alloc_kpages`, and `load_elf` from the original module. Verified code omits global initialization/accessors, multi-page allocators, kernel multi-page allocator, and ELF loading, so substantial functionality is unverified.
- Semantic mismatch: `unmap_upage` drops the frame address and never returns it to the pool, whereas the original frees the `UserFrame`. This leaks user frames and can exhaust the pool despite mappings being removed.

### High
- Missing page-table allocation requirements: `alloc_upage`’s spec only checks user-pool and vmem capacity; the original may allocate page tables from the kernel pool via a closure. Kernel pool capacity and provenance for page-table frames are neither modeled nor required.
- No bulk allocation properties: The original `alloc_upages`/`alloc_kpages` ensure contiguous multi-page allocation and mapping; the verified model lacks specs/proofs for multi-page operations, leaving these behaviors unchecked.

### Medium
- Missing zeroing semantics: Original `alloc_upage` optionally clears the new page; verified version omits the `clear` flag and corresponding postcondition, so data-initialization behavior is unverified.
- Global singleton not modeled: The static `MEMORY_MANAGER` with `init/get/get_mut` (including double-init panic) is unverified; this leaves initialization and access-synchronization responsibilities unspecified.

### Low
- ELF loading unverified: `load_elf` is absent; while noted as out of scope, this leaves the executable loading path (which drives user mappings) without any specification or proof.

## Positive Observations
- Core pool invariants are tracked and required for manager operations.
- Mapping operations require user-space alignment, mapping capacity, and preserve vmem invariants.
- Allocation/unmap/control functions are free of unchecked `assume`/`external_body`; verification passes cleanly.

## Summary
The current verification captures basic single-page allocation/control invariants but omits several core behaviors of the original manager: global initialization, multi-page allocation, kernel-page allocation, and ELF loading. The most serious issue is the unmap path leaking user frames, breaking equivalence and enabling pool exhaustion. Extending the model to include frame return on unmap, page-table allocation from the kernel pool, multi-page operations, and covering the omitted APIs would raise confidence to production-ready levels.
