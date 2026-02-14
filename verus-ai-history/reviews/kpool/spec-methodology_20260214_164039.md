# Review: kpool Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **`KernelFrame` lacks a View type**: `KernelFrame` does not implement the `View` trait and has no `KernelFrameView` abstraction type. The guidelines (Step 1) specify that types should have a `MyTypeView` with abstract types and a `pub closed spec fn view()`. Instead, `KernelFrame` uses closed spec accessors (`spec_address()`, `spec_frame_number()`, `spec_pool_id()`, etc.) directly on the concrete type. While this is a reasonable lightweight approach for a simple wrapper struct with only two fields, it deviates from the documented methodology. Public method specs on `KernelFrame` (e.g., `address()`, `pool_id()`, `base()`) reference `self.spec_address()` and `self.spec_pool_id()` — private closed spec functions rather than view-based `self@.field` accessors.

- **`Kpool` public method specs use `self.spec_num_allocated()` and `self.spec_capacity()`**: Per the guidelines (Step 3), public method specifications should not call `Self` spec functions other than `inv` and `view`. Multiple public methods (`alloc`, `alloc_range`, `alloc_contiguous`, `alloc_noncontiguous`, `free`, `free_range`, `free_contiguous`) reference `self.spec_num_allocated()` and/or `self.spec_capacity()` in their ensures/requires clauses. `spec_num_allocated()` is `pub closed spec fn` on `Kpool` that accesses `self.frame_allocator.spec_num_allocated()` — an implementation detail. `spec_capacity()` is `pub open spec fn` that delegates to `self@.capacity()`, which is fine but adds indirection. These should use `self@.num_allocated()` and `self@.capacity()` instead.

### Medium
- **`KpoolView` exposes `allocator_view: FrameAllocatorView` as a public field**: The view type's `allocator_view` field leaks the underlying `FrameAllocator` abstraction into the public interface. The guidelines state that views should "hide internal fields that aren't important to users." A cleaner approach would be to inline the relevant properties (e.g., `allocated_frames: Set<int>`, `capacity: int`) directly into `KpoolView`, making it self-contained. Currently, `KpoolView` methods like `capacity()`, `is_allocated()`, `num_free()`, etc., are all thin wrappers delegating to `allocator_view`, which partially mitigates this but doesn't fully encapsulate.

- **`Kpool::new()` public spec references `frame_allocator` parameter's internal spec functions**: The ensures clause of `new()` at line 284 uses `frame_allocator.spec_num_allocated()` and `frame_allocator@.is_allocated(i)` — these reference the parameter's implementation-level specs. While acceptable for a constructor that takes an implementation type, it could be cleaner if expressed in terms of the resulting view.

- **`base_addr` hardcoded to 0 in `view()`**: The `Kpool.view()` function (kpool.spec.rs:164) hardcodes `base_addr: 0` for the `KpoolView`. While documented as "Base address is abstract (default 0)", this means `KpoolView::base()` always returns 0 and `frame_addr()` produces addresses relative to 0. This is noted but may limit expressiveness if base address tracking becomes important.

### Low
- **`view()` missing `pub` keyword**: The `View` trait implementation (kpool.spec.rs:160) declares `closed spec fn view(&self)` without the `pub` keyword. Per the guidelines, it should be `pub closed spec fn view()`. In Verus, trait implementations inherit the trait's visibility, so this works correctly, but explicitly adding `pub` would better match the documented convention.

- **KpoolView methods are all `pub open spec fn`**: This is correct per the guidelines (Step 3) — common subexpressions on the view type should be `pub open spec fn`. No issue here; noting for completeness.

## Summary

The kpool spec methodology is well-structured and follows most of the guidelines. Verification passes cleanly (33 verified, 0 errors) with no `assume`, `admit`, or `external_body` remaining. The `Kpool` type has a proper `KpoolView` abstraction with abstract types (`int`, `Set<int>`), a `pub closed spec fn inv()`, and comprehensive public method specs that use `self@.field` for most properties. The proof tests are thorough, covering allocation, deallocation, round-trips, count tracking, and invariant preservation.

The main gaps are: (1) `KernelFrame` lacks a `View` implementation, using closed spec accessors instead of the view pattern; (2) several public method specs reference `self.spec_num_allocated()` — a closed spec on the concrete type — rather than expressing count tracking through the view; and (3) `KpoolView` exposes the internal `FrameAllocatorView` as a public field rather than inlining the abstract state. These are methodology deviations that don't affect correctness but reduce abstraction encapsulation.
