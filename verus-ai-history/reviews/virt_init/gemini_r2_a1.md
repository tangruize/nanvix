# Review: virt_init (gemini-3-pro-preview)

## Grade: B

## Issues Found

### Critical
- **Missing Side Effects (Allocation):** The verified `init` function returns only a `Vec<usize>` of page table base addresses. The original code allocates heap memory (`Box::new(...)`) and constructs `PageTable<PageTableStorage>` objects containing the actual page table entries. The verified code is a model of the *address calculation logic* but does not perform the actual *page table construction*. It is not functionally equivalent to the original code in terms of memory effects and data structure creation.
- **Signature Mismatch:** The verified `init` function signature (`Vec<MemRegion> -> ...`) differs significantly from the original (`LinkedList<TruncatedMemoryRegion> -> ...`). The return types are also incompatible (`Vec<usize>` vs `LinkedList<(PageTableAddress, PageTable<...>>`). This prevents the verified code from being a drop-in replacement.

### High
- **Faithful Modeling of Potential Bug:** The verified code faithfully models what appears to be a bug in the original code regarding MMIO mappings. In the original `init` loop (lines 205-214), `paddr` for MMIO regions is always calculated from `region.start()`, regardless of the current `raw_vaddr`. This means every page in an MMIO region is mapped to the *same physical frame* (the first one). The verification captures this (`spec_init_paddr` depends on `region_start` for MMIO), proving equivalence, but the underlying behavior is likely incorrect.

### Medium
- **Hardcoded Permissions:** The verification hardcodes the permissions (`present=true`, `writable=true`, `user=false`) in `spec_has_init_permissions`. While this matches the original code, the original has a `FIXME: do not be so open about permissions`. The spec should ideally allow for flexible permissions if the code is improved, or at least acknowledge this limitation explicitly in comments.

### Low
- **Loop Bound Modeling:** The original code breaks the loop if `raw_vaddr == MEMORY_SIZE - PAGE_SIZE`. The verified code replaces this with a precondition that all regions fit within `INIT_MEMORY_SIZE`. While sound (assuming the precondition holds), it's a slight divergence in control flow logic.

## Positive Observations
- **Strong Safety Proofs:** The verification successfully proves that sorted, non-overlapping input regions result in strictly increasing, non-overlapping page table bases (`lemma_sorted_addrs_sorted_pgtab_bases`).
- **Comprehensive Coverage:** The verification tracks every single page mapping via ghost state (`Seq<PageMapping>`) and proves that exactly the expected number of pages are processed (`spec_total_pages`).
- **Identity Mapping Proof:** The proofs correctly verify that non-MMIO memory is identity-mapped, which is a critical property for the kernel initialization.
- **Clean Split:** The separation of execution code, specifications, and proofs is well-organized and follows the project's verification patterns.

## Summary
The verification of `virt_init` is a strong model of the address calculation logic, proving key safety properties like "no overlap" and "correct identity mapping". However, it falls short of being a full verification of the implementation because it abstracts away the actual page table memory allocation and data structure construction. It essentially verifies that "if we were to build page tables, these would be the correct addresses," rather than "we correctly built these page tables."

Critically, the verification highlights a likely bug in the original MMIO mapping logic where all pages in a region map to the same physical frame. The verified code is not yet a drop-in replacement due to signature mismatches and missing heap allocation logic.
