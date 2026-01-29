# Review: manager (gpt-5.1-codex-max)

## Grade: D

## Issues Found

### Critical
- **Coverage gap – missing `free_kernel_frame` verification**: The original manager exposes `free_kernel_frame`, but the verified module omits it entirely; kernel-frame deallocation correctness (double-free prevention, provenance, capacity preservation) is unverified. **Suggested Fix:** Add a verified `free_kernel_frame` mirroring the kernel pool’s free operation with provenance, allocation-status, and count-decrement guarantees.

### High
- **API/behavior mismatch – omitted `clear` argument in kernel allocation**: Original `alloc_kernel_frame(clear: bool)` exposes zeroing semantics that may fail; the verified `alloc_kernel_frame()` drops the parameter and treats allocation as pure, so zeroing failures or effects are unchecked. **Suggested Fix:** Reintroduce a `clear` flag (or separate verified clear) with specs covering failure modes and post-state when clearing fails/succeeds.
- **API/behavior mismatch – contiguous kernel allocations**: Original `alloc_many_kernel_frames(clear, count)` promises contiguous ranges and returns `Vec<KernelFrame>` with error handling; the verified version calls `alloc_noncontiguous` and returns ghost indices with a precondition guaranteeing success, eliminating the error path and contiguity guarantee. **Suggested Fix:** Provide a verified contiguous allocator with result type `Result<Seq<int>, Error>`, modeling failure when capacity is insufficient and proving contiguity of returned indices.
- **API/behavior mismatch – user batch allocation**: Original `alloc_many_user_frames` returns `Result<Vec<UserFrame>, Error>`; the verified version returns ghost indices under a precondition that enough frames exist, removing the runtime error path and executable frame returns. **Suggested Fix:** Verify an executable batch allocator (or a loop wrapper) that returns frames or an error, matching original semantics.

### Medium
- **Unenforced pool disjointness invariant**: The verified `inv` only requires kpool/upool invariants; disjointness is left to callers and not captured in the invariant, so isolation between kernel/user regions is unchecked. **Suggested Fix:** Extend pool views to carry base/limit and include disjointness in `inv` (or as a constructor postcondition) to prove isolation once addresses are known.

### Low
- **Liveness asymmetry in batch allocators**: The single-frame allocators model success/failure based on free frames, but batch allocators assume success via preconditions, so liveness (success when capacity allows, failure otherwise) is not specified. **Suggested Fix:** Add ensures/ensures for success on sufficient capacity and explicit Err on insufficiency, aligning with single-frame liveness guarantees.

## Positive Observations
- Single-frame allocators/free functions include strong state-preservation and distinctness properties, with provenance checks for kernel frames.
- Invariants for underlying pools are preserved, and allocation count tracking is specified for single-frame operations.

## Summary
Significant coverage and semantic gaps remain: kernel deallocation is unverified, clearing semantics and contiguous allocations are dropped, and batch APIs lose their error models and contiguity guarantees. Disjointness between kernel/user pools is not enforced in the invariant. Address these issues to align the verified module with the original behavior and key isolation properties.
