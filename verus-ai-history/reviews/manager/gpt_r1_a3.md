# Review: manager (gpt-5.1-codex-max re-review)

## Grade: B-

## Re-Review Findings

### Resolved
- **Contiguous kernel allocation gap**: Added `alloc_contiguous_kernel_frames` returning `Result<usize, Error>` with contiguity, validity, allocation-state change, and failure leaves state unchanged. This addresses the prior mismatch where only non-contiguous ghost allocation existed.

### Remaining / Not Fixed
- **API/behavior mismatch – omitted `clear` argument in kernel allocation (High)**: `alloc_kernel_frame` still omits the `clear: bool` semantics from the original API; zeroing success/failure and post-state effects remain unmodeled. Documentation notes the omission but does not restore or verify the behavior.
- **API/behavior mismatch – user batch allocation (High)**: `alloc_many_user_frames` returns ghost indices under a capacity precondition, removing runtime error paths and executable frame returns (`Result<Vec<_>, Error>` in the original). Behavior remains narrower than the source API.
- **Unenforced pool disjointness invariant (Medium)**: `inv` still omits disjointness; `pools_are_disjoint` is spec-only and cannot be proven with the current base/limit modeling. Isolation between kernel/user regions is not enforced.
- **Liveness asymmetry in batch allocators (Low)**: Batch allocators (user and non-contiguous kernel) assume success via preconditions and do not model failure when insufficient capacity, unlike single-frame allocators that specify both success and failure conditions.

### New Issues
- None observed.

## Summary
Contiguous kernel allocation is now modeled with proper contiguity and error handling. The major gaps that remain are: missing clear semantics for kernel allocation, user batch allocation still ghost-only without error modeling, lack of enforced pool disjointness, and batch allocator liveness asymmetry. Verification is improved but still diverges from the original API for clearing and batch behaviors.
