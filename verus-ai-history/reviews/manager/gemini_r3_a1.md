# Review: manager (gemini-3-pro-preview)

## Grade: A

## Issues Found

### Medium
- **Location**: `PhysMemoryManagerView`, `pools_are_disjoint`
- **Description**: The verification model assumes `base_addr: 0` for both pools (as noted in comments). This means the critical property that kernel and user memory regions do not overlap cannot be verified. While the allocator logic is correct for indices, the physical address separation relies entirely on unverified external initialization.
- **Suggested Fix**: Update `Kpool` and `Upool` views to track actual base addresses. Pass base addresses to `PhysMemoryManager::new` or read them from the pools to verify `kpool_limit() <= upool_base() || upool_limit() <= kpool_base()`.

### Low
- **Location**: `alloc_many_user_frames`, `alloc_noncontiguous_kernel_frames`
- **Description**: These functions return `Ghost<Seq<int>>` instead of `Vec<Frame>` (or Result thereof). This makes them unusable for executable code, forcing callers to rewrite batch allocations as loops around single allocation.
- **Suggested Fix**: If performance permits, implement the loop inside these functions and return a `Vec` (or a verified array type) to maintain API compatibility. Otherwise, clearly mark as specification-only.

- **Location**: `alloc_kernel_frame`
- **Description**: Removed `clear` parameter. While memory zeroing is orthogonal to allocation safety, this requires callers to manually zero memory if needed, changing the API contract.
- **Suggested Fix**: Add a separate `alloc_zeroed_kernel_frame` or similar if this functionality is relied upon, or document the requirement for callers to clear memory.

## Positive Observations
- **Coverage**: All functions are covered or have justified replacements.
- **Strong Specs**: Specifications are rigorous, tracking allocation counts, frame indices, and ensuring no side effects on the other pool.
- **Provenance**: `alloc_kernel_frame` and `free_kernel_frame` correctly track/check pool ID provenance, preventing freeing a frame to the wrong pool.
- **Documentation**: Excellent documentation explaining design choices and deviations from the original API.

## Summary
The `manager` verification is of high quality. It successfully modularizes the verification by building on `Kpool` and `Upool` specs. The logic is sound, and the specifications capture the essential behavior of a split kernel/user memory manager. The primary limitations are the simplified memory layout model (base=0) and some API divergence to simplify verification, which are well-documented trade-offs.
