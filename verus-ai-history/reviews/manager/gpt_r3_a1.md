# Review: manager (gpt-5.1-codex-max)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location: alloc_kernel_frame / alloc_contiguous_kernel_frames**  
  **Description:** The verified API removes the `clear: bool` parameter and does not specify or prove frame zeroing semantics that exist in the original allocator. This is a behavioral gap and omits a safety-relevant property (returning cleared frames) expected by callers.  
  **Suggested Fix:** Reintroduce a `clear` flag (or a separate `clear_frame` routine) with a specification that guarantees zeroed frames when requested, and prove it preserves pool invariants.
- **Location: alloc_contiguous_kernel_frames / alloc_noncontiguous_kernel_frames**  
  **Description:** The verified contiguous allocation returns only a start index and the non-contiguous variant returns ghost indices, while the original `alloc_many_kernel_frames` returns a `Vec<KernelFrame>` preserving frame objects and provenance. The change drops executable batch allocation semantics and provenance checks for all returned frames, so coverage and equivalence are missing.  
  **Suggested Fix:** Provide a verified `alloc_many_kernel_frames(clear, count) -> Result<Vec<KernelFrame>, Error>` that matches the original behavior (including clearing) and prove distinctness, validity, and provenance for each frame.
- **Location: alloc_many_user_frames**  
  **Description:** Verified version returns `Ghost<Seq<int>>` with a precondition requiring enough free frames, eliminating the error path present in the original `Result<Vec<UserFrame>, Error>`. This weakens observable behavior and prevents callers from handling exhaustion via `Err`.  
  **Suggested Fix:** Verify an executable `alloc_many_user_frames(n) -> Result<Vec<UserFrame>, Error>` that (a) preserves pool invariants, (b) returns an error when insufficient space, and (c) proves distinctness and validity of all returned frames.

### Medium
- **Location: PhysMemoryManager invariants (inv) / pools_are_disjoint**  
  **Description:** Pool disjointness is only a spec helper and not part of the manager invariant or constructor guarantees. The original system relies on kernel/user pool separation; without enforcing or assuming disjointness, aliasing between pools remains unchecked in the verified model.  
  **Suggested Fix:** Add a constructor precondition/ensures capturing non-overlap (once base addresses are modeled), or strengthen the invariant to include disjointness when provided by pool metadata.
- **Location: free_user_frame semantics**  
  **Description:** Verified `free_user_frame` requires the frame to be allocated and guarantees `Ok(())`, while the original function returns `Result<(), Error>` and can signal misuse (e.g., double free) via `Err`. The proof omits the error path, so observable behavior is not equivalent.  
  **Suggested Fix:** Model the error cases: permit `Err` when the frame is not allocated or invalid, and prove success otherwise, aligning with the original API contract.
- **Location: Coverage of RAII-based kernel frame frees**  
  **Description:** The original design relies on `Drop` for kernel frames; the verified code introduces explicit `free_kernel_frame` but does not model or prove the drop-based deallocation path, leaving original behavior unverified.  
  **Suggested Fix:** Model `Drop` for `KernelFrame` (or equivalently specify/prove an auto-free path) to ensure the original RAII semantics are covered, or adjust the original to explicit frees and verify both call sites.

### Low
- None.

## Positive Observations
- Core pool invariants are preserved and propagated through manager operations; allocations track allocation counts and frame validity (indices and alignment).
- Liveness for single-frame allocations is captured (succeeds when free frames exist, fails otherwise) while leaving the other pool unchanged.
- Provenance is enforced for kernel frames on allocation and free, preventing cross-pool misuse.

## Summary
The verification provides solid per-pool safety guarantees and liveness for single-frame operations but diverges from the original API in several key behaviors (clearing semantics, batch allocations, error paths, and RAII frees). Strengthening the specs to match observable behavior—especially batch allocations and clear/drop semantics—and enforcing pool disjointness will improve coverage and equivalence. Addressing these gaps should bring the verification to parity with the original manager functionality.
