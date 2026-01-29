# Review: manager (gpt-5.1-codex-max)

## Grade: D

## Issues Found

### Critical
- None.

### High
- **Location: alloc_kernel_frame / alloc_contiguous_kernel_frames**  
  **Status:** Not fixed. The `clear: bool` semantics remain removed; no specification or proof of returning zeroed frames as provided by the original API. Behavioral gap still unaddressed.  
  **Needed:** Reintroduce a cleared-allocation path (flag or helper) and prove zeroed frames while preserving invariants.
- **Location: alloc_contiguous_kernel_frames / alloc_noncontiguous_kernel_frames**  
  **Status:** Not fixed. Batch kernel allocation still returns a start index or ghost indices instead of `Vec<KernelFrame>`; provenance/validity of each frame and executable batch semantics are not proved.  
  **Needed:** Verified `alloc_many_kernel_frames(clear, count) -> Result<Vec<KernelFrame>, Error>` matching original behavior (including clearing), with distinctness/validity proofs.
- **Location: alloc_many_user_frames**  
  **Status:** Not fixed. Still returns `Ghost<Seq<int>>` with a precondition requiring enough space, removing the failure path and executable returns of `Vec<UserFrame>`. Observable behavior diverges from original API.  
  **Needed:** Executable batch user allocation returning `Result<Vec<UserFrame>, Error>` with error on exhaustion and proofs of distinctness/validity.

### Medium
- **Location: PhysMemoryManager inv / pools_are_disjoint**  
  **Status:** Not fixed. Disjointness remains only a doc/spec helper; constructor/invariant do not require or ensure non-overlap. Potential aliasing between pools remains unchecked.  
  **Needed:** Include disjointness in constructor preconditions/ensures (once base addresses are modeled) or strengthen the invariant accordingly.
- **Location: free_user_frame semantics**  
  **Status:** Not fixed. Preconditions forbid misuse and ensures always `Ok(())`; the original API surfaces errors (e.g., double free/invalid) via `Err`. Error path remains unmodeled, so behavior is not equivalent.  
  **Needed:** Model error cases and return `Err` when preconditions fail; prove success otherwise.
- **Location: Kernel frame RAII frees**  
  **Status:** Not fixed. Original design frees via `Drop`; verified code only covers explicit `free_kernel_frame` and does not model drop-based deallocation. Original behavior remains unverified.  
  **Needed:** Specify/prove the RAII/drop path or adjust the API and proofs to cover all deallocation mechanisms used by callers.

### Low
- None.

## Positive Observations
- Single-frame allocations/free still preserve pool invariants, capacities, and provenance checks; liveness for single-frame alloc succeeds/fails based on availability.
- Allocation/free proofs maintain non-interference between pools for single-frame operations.

## Summary
None of the previously reported behavioral gaps were addressed. Clearing semantics, executable batch allocations with error paths, disjointness guarantees, error modeling for frees, and RAII deallocation remain unverified. Verification is still incomplete relative to the original manager API.
