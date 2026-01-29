# Review: manager (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- Unmap leak and behavioral mismatch: `unmap_upage` drops the returned frame address without freeing it to the user pool, whereas the original frees the `UserFrame`. This can exhaust user frames and breaks semantic equivalence. Suggested Fix: model and return the frame to the pool (or extend `vmem.unmap` spec to return a `UserFrame`) and prove pool counts are restored.
- Cloning loses mappings: `new_vmem` ensures `mapping_count == 0`, while the original `Vmem::clone` preserves mappings. This changes semantics and misses obligations about cloned address spaces. Suggested Fix: model clone to preserve existing mappings (and associated invariants) or explicitly justify deviation with a proof of equivalence.

### High
- Allocation accounting/spec gap: `alloc_upage` lacks postconditions tying upool free count decrement and frame provenance to the mapped page; specs allow capacity to be unchanged after allocation. Suggested Fix: strengthen ensures to relate `upool` free count and pool id to the newly mapped frame.
- Missing coverage for multi-page and ELF paths: `alloc_upages`, `alloc_kpages`, and `load_elf` are absent from the verified module, leaving major functionality unchecked. Suggested Fix: add verified counterparts (or stubs with precise specs) for these functions.
- Page-table allocation behavior not modeled: original user-page mapping allocates kernel frames/page tables via `page_table_allocator`, but the verified version calls `vmem.map` without modeling kernel frame consumption. Suggested Fix: include kpool usage and prove capacity constraints for page-table allocation.

### Medium
- Clear-zeroing semantics dropped: original `alloc_upage`/`alloc_kpage` support `clear` to zero pages; verified code omits this behavior and its safety guarantees. Suggested Fix: model the `clear` parameter and prove zeroing or document and justify exclusion.
- Global singleton/init semantics unverified: `init`, `get`, and `get_mut` (global state and synchronization assumptions) are not modeled, so initialization safety and exclusivity are unchecked. Suggested Fix: add specs capturing single-initialization and access conditions or separate a verified wrapper with documented assumptions.
- Manager invariants do not relate pool counts to vmem mappings, so no global accounting ties allocations/unmaps to pool capacity. Suggested Fix: extend invariants/specs to connect mapping count changes with pool free/used counts.

### Low
- No specification for user-range busy check in `alloc_upages` (FIXME in original) is addressed; verification misses this pending safety property. Suggested Fix: incorporate and prove the range-not-busy condition when multi-page allocation is modeled.

## Positive Observations
- Verification passes with no `assume` or `external_body` in the module.
- Core preconditions enforce page alignment, user-address range, mapping existence for unmap, and capacity checks for pools and vmem.
- Pool invariants are preserved across operations, and mapping counts are tracked in user operations.

## Summary
Significant functionality (multi-page alloc/free, ELF loading, page-table allocations, global init) is unverified, and key semantic differences exist (leaking frames on unmap, losing mappings on clone, missing clear semantics). Strengthen specs to tie allocations to pool accounting, model the missing behaviors, and align clone/unmap semantics with the original to achieve coverage and equivalence.
