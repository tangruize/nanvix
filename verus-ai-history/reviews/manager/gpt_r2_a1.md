# Review: manager (gpt-5.1-codex-max)

## Grade: B-

## Issues Found

### Critical
- Coverage gap: `alloc_kernel_frame(clear)` and `alloc_many_kernel_frames(clear, count)` drop the `clear` behavior entirely, so verified API is not semantically equivalent to the original that can request zeroed frames.
- Coverage gap: Batch allocators return only ghost indices (`Ghost<Seq<int>>`) instead of concrete `Vec<Frame>`, leaving the executable behavior of batched allocations unverified.

### High
- `alloc_many_user_frames` / `alloc_many_kernel_frames`: Specs require the caller to pre-prove sufficient free frames and then force `Ok`, eliminating the error path that exists in the original `Result`-returning APIs; this weakens coverage of failure behavior and diverges from runtime semantics.
- Missing coverage: The original exposes only `free_user_frame` (kernel frames are freed through pool APIs/RAII), but the verified module introduces `free_kernel_frame` and `alloc_contiguous_kernel_frames` (not in the source) while not verifying the original kernel-frame free pathway, breaking equivalence/coverage.

### Medium
- Pool disjointness is only an optional view-level predicate with no invariant or constructor guarantee; the verified manager does not ensure kernel/user regions are non-overlapping, leaving a key isolation property unproven for this core module.
- `free_user_frame` is specified with preconditions that the frame is allocated, so the potential error path on invalid input is unverified, whereas the original returns `Result` that can signal errors; specs are too strong and miss robustness checks.

### Low
- Documentation states “No Drop semantics” and introduces explicit frees, but the original relies on the pools’ existing RAII/Drop conventions; this mismatch is not reflected in the specs and may hide behavioral differences in how frames are released.

## Positive Observations
- No `assume` or `external_body` usage in the core module; proof is fully internal.
- Invariants require both pools to maintain their own invariants, and specs preserve capacities and per-pool independence for single-frame alloc/free operations.
- Allocation specs track alignment, index validity, and distinctness, preventing double-alloc and double-free under the stated preconditions.

## Summary
The verification passes but is not equivalent to the original implementation: it removes the `clear` behavior, replaces executable batch allocation with ghost-only results, omits error-path coverage for batched and free operations, and adds APIs absent from the source while leaving kernel/user disjointness unenforced. To align with the kernel code, restore `clear` semantics, verify concrete batch allocations (including failure cases), cover the original kernel-frame free path, and strengthen invariants to guarantee pool disjointness (or document and enforce it at construction). Until these gaps are addressed, the verification should be considered partial and not a full correctness guarantee for the original manager. 
