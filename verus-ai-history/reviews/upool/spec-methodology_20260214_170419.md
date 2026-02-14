# Review: upool Spec Methodology (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### High

- **UpoolView has `pub` fields exposing internals.** `UpoolView.allocator_view` (type `FrameAllocatorView`) and `UpoolView.base_addr` (type `int`) are `pub` fields. Per the guidelines (Step 1), the View type should hide internal fields not important to users. `allocator_view` directly exposes the underlying `FrameAllocatorView`, leaking the implementation detail that `Upool` wraps a `FrameAllocator`. Users can access `self@.allocator_view` and bypass the abstraction layer. The `UpoolView` already provides accessor spec functions (`capacity()`, `is_allocated()`, etc.) that properly abstract over the internals — the fields themselves should be private, with only these accessors exposed. (upool.spec.rs:155-161, upool.rs:155-161)

- **UserFrame.addr field is `pub`, violating encapsulation.** The `addr` field in `UserFrame` is declared `pub` (upool.rs:104). The code comment at line 98-100 acknowledges this is for Verus spec access, but the guidelines say member fields should be private and accessed via getter/setter methods. This is partially mitigated by the spec functions (`spec_address`, `spec_frame_number`, etc.) in the spec file, but callers can still directly access `uframe.addr` in exec code, bypassing `address()`.

### Medium

- **Public spec functions on `Upool` beyond `inv`/`view`.** The guidelines (Step 3) say "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." However, `Upool` has two additional public spec functions: `spec_capacity()` (open, upool.spec.rs:191) and `spec_num_allocated()` (closed, upool.spec.rs:198). These are used in public method specs (e.g., `alloc_many` precondition at upool.rs:334, `alloc`/`free` postconditions). Per the guidelines, these should either be moved to `UpoolView` or made private. `spec_capacity` simply delegates to `self@.capacity()` and could be replaced inline. `spec_num_allocated` is closed and references `self.frame_allocator.spec_num_allocated()` — it should ideally be expressed through the View (e.g., `self@.num_allocated()`).

- **Public method specs use `self.spec_num_allocated()` / `self.spec_capacity()` instead of `self@` equivalents.** Per Step 3, public method specs should only use `inv`, `view` (i.e., `@`), and View-level spec functions — not `Self` spec functions. Lines using `self.spec_num_allocated()` (upool.rs:267, 334, 358, 384, 412, 530, 581) and `self.spec_capacity()` (upool.rs:365) should use `self@.num_allocated()` and `self@.capacity()` respectively.

- **UserFrame spec functions are all `pub open` including those accessing `self.addr` directly.** Functions like `spec_address`, `spec_frame_number`, `spec_is_aligned`, `spec_raw_address` (upool.spec.rs:17-37) are `pub open` and directly reference `self.addr`. Since `UserFrame` doesn't implement `View`, these serve as the abstraction boundary, which is acceptable. However, `UserFrame` has no View type at all — it would be more aligned with the methodology to define a `UserFrameView` (even if trivial), make `view()` closed, and route specs through it. This is a minor design concern given `UserFrame` is essentially a newtype wrapper.

### Low

- **`UpoolView` functions on `impl UpoolView` are all `pub open`.** This is correct per the guidelines (Step 3 final paragraph: "create a `pub open spec fn` in `MyTypeView`"). No issue here — noted for completeness.

- **`view()` on `Upool` is `closed spec fn` but not explicitly `pub`.** At upool.spec.rs:167, the `View` trait implementation uses `closed spec fn view(...)` without `pub`. In Verus, trait impl functions inherit the trait's visibility, so this is effectively public. This is correct behavior but worth noting the convention difference.

## Summary

The upool spec methodology is well-structured and largely follows the guidelines. The View type (`UpoolView`) uses proper abstract types (`int`, `Set<int>` via `FrameAllocatorView`), `view()` is correctly `closed`, and `inv()` is `pub closed spec fn`. Verification passes cleanly with 20 verified, 0 errors, and there are no `assume`, `admit`, or `external_body` annotations.

The main areas for improvement are: (1) `UpoolView` fields should be private rather than `pub` to prevent clients from bypassing the abstraction, (2) `UserFrame.addr` should be private with the existing accessor methods used instead, and (3) public method specs should exclusively use `self@` (View-level) accessors rather than `self.spec_num_allocated()` / `self.spec_capacity()`, with those helper functions either inlined or moved to `UpoolView`.

These are methodology refinements rather than correctness issues — the verified properties are sound and comprehensive, covering no-double-allocation, no-double-free, disjointness, liveness, and count tracking.
