# Review: manager (claude-opus-4.5)

## Grade: A-

## Issues Found

### Critical
- None

### High
- **Location**: `alloc_kernel_frame` / API difference
  - **Description**: The verified `alloc_kernel_frame()` omits the `clear: bool` parameter present in the original API (`alloc_kernel_frame(clear: bool)`). While the documentation explains this is intentional (memory zeroing is orthogonal to allocation safety), this is a semantic difference that could affect callers relying on initialization guarantees.
  - **Suggested Fix**: Either add a spec-level `clear` parameter that doesn't affect verification but preserves API compatibility, or add a verified `clear_frame()` helper function as suggested in the documentation.

- **Location**: `alloc_many_user_frames` / Return type difference
  - **Description**: Original returns `Result<Vec<UserFrame>, Error>` while verified returns `Ghost<Seq<int>>` (not even wrapped in Result). This changes the error handling semantics - the original can fail, but the verified version requires a precondition ensuring enough frames exist. This shifts the burden from runtime error handling to caller proof obligations.
  - **Suggested Fix**: Consider wrapping in `Result` to match original fallibility semantics, even if preconditions prevent actual failures.

### Medium
- **Location**: `free_kernel_frame` / Missing from original manager.rs
  - **Description**: The original `PhysMemoryManager` does not have an explicit `free_kernel_frame()` method. Instead, kernel frame deallocation is handled via RAII (`Drop` trait on `KernelFrame`). The verified version adds an explicit `free_kernel_frame()` method. While this is a reasonable modeling choice for verification, it's technically an API addition.
  - **Suggested Fix**: Document this as an intentional modeling difference. The verification correctly captures the deallocation semantics that `Drop` would provide.

- **Location**: `alloc_many_kernel_frames` / Semantics difference
  - **Description**: Original `alloc_many_kernel_frames(clear, count)` allocates a **contiguous** range (calls `alloc_many` on kpool which does contiguous search). The verified `alloc_many_kernel_frames(count)` allocates **non-contiguous** frames (`alloc_noncontiguous`). The verified version adds a separate `alloc_contiguous_kernel_frames` for contiguous semantics.
  - **Suggested Fix**: Either rename `alloc_many_kernel_frames` to `alloc_noncontiguous_kernel_frames` to avoid confusion, or make `alloc_many_kernel_frames` delegate to contiguous allocation to match original semantics.

- **Location**: `pools_are_disjoint()` / Unverifiable property
  - **Description**: The `pools_are_disjoint()` spec function is defined but acknowledged to be unverifiable because both pools use `base_addr: 0` as placeholder. This is a critical isolation property that cannot currently be proven.
  - **Suggested Fix**: Track actual base addresses in pool constructors or add an `assume` with clear documentation that pool disjointness is established at system initialization.

### Low
- **Location**: `PhysMemoryManagerView` / Documentation completeness
  - **Description**: The view struct exposes many spec functions but doesn't have a validity/well-formedness invariant at the view level (e.g., capacity >= 0, pools don't overlap at spec level).
  - **Suggested Fix**: Add a `wf()` (well-formed) spec function on `PhysMemoryManagerView` that captures basic well-formedness properties.

- **Location**: Test module
  - **Description**: The proof tests (`test_user_alloc_kpool_unchanged`, `test_kernel_alloc_upool_unchanged`, `test_user_alloc_free_cycle`) are relatively simple state-based checks. They verify that the specs imply expected properties but don't exercise edge cases like allocation at capacity or concurrent operations.
  - **Suggested Fix**: Add more proof tests for edge cases: empty pool behavior, full pool behavior, alloc-alloc-free-alloc cycles.

## Positive Observations

1. **No `assume` or `external_body` in manager.rs**: The core module is fully verified without trusted assumptions, which is excellent for soundness.

2. **Comprehensive specifications**: Each function has detailed pre/postconditions covering:
   - Invariant preservation
   - Pool isolation (kernel ops don't affect user pool and vice versa)
   - Liveness properties (allocation succeeds iff free frames exist)
   - Frame validity (alignment, bounds)
   - Count tracking (num_allocated changes correctly)

3. **Provenance tracking**: The `pool_id` on `KernelFrame` prevents cross-pool aliasing bugs - frames can only be freed to their originating pool.

4. **Pool independence proven**: Specs explicitly prove that operations on one pool leave the other unchanged (`self@.kpool_view == old(self)@.kpool_view` and vice versa).

5. **No double allocation/free**: Inherited from underlying pool verifications, the specs ensure frames are only allocated if free and only freed if allocated.

6. **Well-documented design decisions**: The module header extensively documents intentional differences from the original API with rationale.

7. **Verification passes**: All 11 verification conditions pass, indicating the implementation satisfies the specifications.

8. **Compositional verification**: The manager properly composes verified `Kpool` and `Upool` modules, maintaining their invariants.

## Summary

The verification is well-executed with strong specifications covering the essential correctness properties of a physical memory manager: no double allocation, no double free, pool isolation, liveness, and invariant preservation. The code passes verification with no trusted assumptions in the core module.

**Strengths**:
- Clean compositional design
- Strong isolation properties between kernel and user pools
- Provenance tracking for kernel frames
- No soundness holes in the core module

**Areas for Improvement**:
- API semantic differences (contiguous vs non-contiguous, clear parameter, return types) should be reconciled or more prominently documented
- Pool disjointness property should be tracked with actual base addresses
- Additional edge-case proof tests would strengthen confidence

The grade of A- reflects excellent verification quality with some API equivalence gaps that should be addressed for complete fidelity to the original implementation.
