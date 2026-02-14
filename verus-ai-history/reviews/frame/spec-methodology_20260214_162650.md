# Review: frame Spec Methodology (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical
- None.

### High
- **`spec_num_allocated` is a third public spec fn beyond `inv` and `view`** (frame.spec.rs:147–149). The guidelines say "Don't write any further `pub` specification functions in `impl MyType` beyond `inv` and `view`." `spec_num_allocated` is `pub closed spec fn` on `FrameAllocator` and is used extensively in public method postconditions. While its internals are hidden (`closed`), its existence as a third public spec fn departs from the methodology. Consider moving this to `FrameAllocatorView` as a `pub open spec fn` (e.g., computed from `allocated_frames.len()`), or keeping it private and using the existing `FrameAllocatorView::num_allocated()` in public method specs instead. This would also eliminate the subtle discrepancy between the bitmap-based `spec_num_allocated` and the set-based `num_allocated` on the view.

### Medium
- **`new()` preconditions reference `bitmap.inv()` and `bitmap.is_bit_set(i)`** (frame.rs:73–77). While `bitmap` is an input parameter (not `self`), the spec exposes `Bitmap`'s internal specification functions to callers. This is acceptable for constructors but worth noting — callers must know `Bitmap`'s spec interface to use `FrameAllocator::new()`. The `from_raw_storage()` constructor is cleaner in this regard, using only `RawArray`'s view.

### Low
- **`FrameAllocatorView` fields are `pub`** (frame.rs:46–48). The guidelines suggest using `pub open spec fn` helpers on the view type for common expressions. The fields `allocated_frames` and `capacity` are already public, and several `pub open spec fn` helpers exist on `FrameAllocatorView`. This is fine since the View type is the public abstraction, but if the view representation ever changes, all callers referencing fields directly would need updating. This is a minor design observation, not a violation.

## Checklist

| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | View uses abstract types | ✅ Pass | `Set<int>` for allocated frames, `int` for capacity. No concrete types (Vec, usize, etc.). |
| 2 | `view()` is `pub closed spec fn` | ✅ Pass | Declared as `closed spec fn` in `impl View for FrameAllocator` (frame.spec.rs:108). Effectively `pub` via trait. |
| 3 | `inv()` exists and is `pub closed spec fn` | ✅ Pass | Declared at frame.spec.rs:124 as `pub closed spec fn inv(&self) -> bool`. Comprehensive: covers bitmap invariant, capacity bounds, view consistency, allocated-frames-in-range, bitmap↔view connection, and no-memory-aliasing. |
| 4 | Public method specs avoid `self.field` | ✅ Pass | All public method requires/ensures use `self@.field`, `self.inv()`, `self.spec_num_allocated()`, or `old(self)@.field`. No direct `self.bitmap` in specs. |
| 5 | Public methods require/ensure `inv()` | ✅ Pass | All public methods with `&self`/`&mut self` require `self.inv()` (or `old(self).inv()`) and ensure `self.inv()` (or `result.inv()`). Constructors ensure `result.inv()`. |
| 6 | No remaining assume/admit/external_body | ✅ Pass | No `assume`, `admit`, or `#[verifier::external_body]` in frame files. One comment at line 697 mentions "assume" in prose context only (not a Verus directive). |
| 7 | Verification passes | ✅ Pass | 26 verified, 0 errors. |

## Summary

The frame allocator spec methodology is well-executed. The `FrameAllocatorView` cleanly abstracts the implementation using `Set<int>` and `int`. The `view()` is properly closed, `inv()` is comprehensive and closed, and all public method specs correctly use `self@` rather than `self.field`. The invariant is maintained across all public methods. Verification passes completely with no assumptions or suppressed checks.

The only notable departure is `spec_num_allocated` as a third `pub` spec fn on `FrameAllocator` (beyond `inv` and `view`). While it is `closed` and does not leak implementation details, the methodology recommends limiting public spec fns to just `inv` and `view`, placing additional helpers on the View type instead. The existing `FrameAllocatorView::num_allocated()` could serve this role if the bitmap-based count were connected to it via a lemma. This is a High-severity methodology deviation but not a correctness issue — the specs are sound and the abstraction boundary is maintained.
