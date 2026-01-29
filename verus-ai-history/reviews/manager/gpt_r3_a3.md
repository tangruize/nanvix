# Review: manager (gpt-5.1-codex-max)

## Grade: D

## Issues Found

### Critical
- None.

### High
- **Location: alloc_kernel_frame / alloc_contiguous_kernel_frames**  
  **Status:** Not fixed. The `clear: bool` semantics remain absent; no specification or proof of returning zeroed frames as the original API allows. Behavioral gap persists.  
  **Needed:** Provide a cleared-allocation path (flag or helper) with a proof that frames are zeroed while preserving pool invariants.
- **Location: alloc_contiguous_kernel_frames / alloc_noncontiguous_kernel_frames**  
  **Status:** Not fixed. Batch kernel allocation still returns a start index or ghost indices instead of `Vec<KernelFrame>`; provenance/validity for each frame in an executable batch is unproved.  
  **Needed:** A verified `alloc_many_kernel_frames(clear, count) -> Result<Vec<KernelFrame>, Error>` matching original semantics (including clearing), with distinctness/validity proofs.
- **Location: alloc_many_user_frames**  
  **Status:** Not fixed. Still returns `Ghost<Seq<int>>` with a precondition requiring sufficient space, eliminating the failure path and executable `Vec<UserFrame>` return. Observable behavior diverges from the original API.  
  **Needed:** Executable batch user allocation returning `Result<Vec<UserFrame>, Error>` that errors on exhaustion and proves distinctness/validity.

### Medium
- **Location: PhysMemoryManager inv / pools_are_disjoint**  
  **Status:** Not fixed. Disjointness remains only a doc/spec helper; constructor/invariant do not require or ensure non-overlap, so cross-pool aliasing is unchecked.  
  **Needed:** Include disjointness in constructor preconditions/ensures (once base addresses are modeled) or strengthen the invariant accordingly.
- **Location: free_user_frame semantics**  
  **Status:** Not fixed. Preconditions forbid misuse and ensures always `Ok(())`; original API can signal errors (e.g., double free/invalid) via `Err`. Error path is still unmodeled.  
  **Needed:** Model error cases and return `Err` when preconditions fail; prove success otherwise.
- **Location: Kernel frame RAII frees**  
  **Status:** Not fixed. Original design frees via `Drop`; verified code only covers explicit `free_kernel_frame` and does not model drop-based deallocation. Original behavior remains unverified.  
  **Needed:** Specify/prove the RAII/drop path or adapt the API/proofs to cover all deallocation mechanisms used by callers.

### Low
- None.

## Positive Observations
- Single-frame alloc/free still preserve pool invariants, capacities, and provenance checks; liveness for single-frame allocations is captured, and cross-pool non-interference holds for those operations.

## Summary
No previously reported gaps were addressed. Clearing semantics, executable batch allocations with error handling, disjointness guarantees, free error modeling, and RAII deallocation remain unverified. Verification remains incomplete relative to the original manager API.
