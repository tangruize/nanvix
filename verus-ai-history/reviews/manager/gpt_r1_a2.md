# Review: manager (gpt-5.1-codex-max re-review)

## Grade: C

## Re-Review Findings

### Resolved
- **Kernel free coverage gap addressed**: `free_kernel_frame` is now present with provenance, double-free prevention, count decrement, and pool-state preservation ensures. This closes the previous critical omission.

### Remaining / Not Fixed
- **API/behavior mismatch – omitted `clear` argument in kernel allocation (High)**: `alloc_kernel_frame` still omits the `clear: bool` semantics from the original API; zeroing success/failure and post-state effects remain unmodeled. Documentation notes the omission but does not restore or verify the behavior.
- **API/behavior mismatch – contiguous kernel allocations (High)**: `alloc_many_kernel_frames` returns ghost indices, assumes success via precondition, and uses non-contiguous `alloc_noncontiguous`; contiguity guarantees and failure/error paths from the original API are still absent.
- **API/behavior mismatch – user batch allocation (High)**: `alloc_many_user_frames` returns ghost indices under a capacity precondition, removing runtime error paths and executable frame returns (`Result<Vec<_>, Error>` in the original). Behavior remains narrower than the source API.
- **Unenforced pool disjointness invariant (Medium)**: `inv` still omits disjointness; `pools_are_disjoint` is spec-only and cannot be proven with the current base/limit modeling. Isolation between kernel/user regions is not enforced.
- **Liveness asymmetry in batch allocators (Low)**: Batch allocators still assume success via preconditions and do not model failure when insufficient capacity, unlike single-frame allocators that specify both success and failure conditions.

### New Issues
- None observed.

## Summary
The critical omission of `free_kernel_frame` is fixed with proper provenance and state guarantees. However, the major API/semantic gaps flagged earlier (clear semantics, contiguous and batch allocation behavior, disjointness enforcement, and batch liveness) remain unresolved. Verification coverage is improved but still diverges materially from the original API for batch and clearing behaviors.
