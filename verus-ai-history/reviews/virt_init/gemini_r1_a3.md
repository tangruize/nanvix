# Review: virt_init (gemini-3-pro-preview)

## Grade: B+

## Issues Found

### High
- **Model vs. Implementation Gap**: The verification is still a "Verified Model" that runs parallel to the code rather than verifying the actual implementation. It uses `Vec<usize>` and `InitResult` instead of the kernel's actual `Result<LinkedList<...>, Error>` and `PageTable` structures. While `init_checked` models the runtime behavior correctly, it is still a shadow implementation.

### Medium
- **External Body Abstractions**: `page_table_map_page` and `get_mmio_paddr` are `external_body`, meaning the safety of the underlying pointer operations and hardware interactions is assumed rather than verified.

### Low
- **Data Structure Simplification**: `MemRegion` is simplified compared to `TruncatedMemoryRegion`.

## Positive Observations
- **Input Validation Verified**: The prover has introduced `init_checked` and `validate_regions`, which successfully addresses the previous "Assumption of Validity" issue. The `init_checked` function has NO preconditions on region ordering/overlap, and correctly returns `OverlapError` or `Ok` based on a verified runtime check. This proves that the runtime check is sufficient to guard the initialization logic.
- **Side Effects Modeled**: The `page_table_map_page` function models the side effects of mapping pages, connecting the logical verification to the system state updates.
- **Robust Specification**: The postconditions for `init_checked` (on the `Ok` path) are extremely strong, guaranteeing alignment, ordering, coverage, and correctness of physical address mappings.
- **Clean Composition**: The separation of `validate_regions` (check) and `init` (algorithm) allows for clear reasoning about safety and correctness independently, composed safely in `init_checked`.

## Summary
The verification has significantly improved with the addition of `init_checked`, which correctly models the runtime input validation that was previously missing. This addresses the primary safety concern (that invalid inputs might be processed unsafely). The remaining gap is that this is a verified model *of* the kernel logic, not the kernel implementation itself. The verified code is structurally identical to the real code but uses simplified types (`Vec` vs `LinkedList`). Given the complexity of the kernel's memory management types, this is a reasonable abstraction trade-off. The logic, safety, and correctness of the initialization algorithm are now fully verified. The grade is raised to **B+**.
