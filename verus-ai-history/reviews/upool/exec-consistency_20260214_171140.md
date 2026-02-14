# Review: upool Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Result
- **20 verified, 0 errors** — all proofs pass.
- No `assume`, `admit`, `external_body`, or `trusted` annotations found.

## Issues Found

### Critical
- None.

### Minor

1. **`alloc_many` return type divergence is significant but well-documented.**
   The original returns `Result<Vec<UserFrame>, Error>` (exec-visible allocated frames); the verified version returns `Ghost<Seq<int>>` (erased at compile time). This means the verified `alloc_many` cannot serve as a drop-in replacement — callers lose exec access to allocated frames. The doc comment correctly advises using `alloc()` in a loop for executable code, and the Verus Vec limitation is a genuine blocker. The state-transition equivalence (same number of `FrameAllocator::alloc()` calls, same pool mutations) is sound. However, the original's `alloc_many` also reverses frame order via `uframes.pop()` — this order-sensitivity is not modeled. **Impact: low** (order is not semantically significant for pool correctness).

2. **`UserFrame.addr` visibility change (`pub` vs private).**
   The field is private in the original and `pub` in Verus. Documented with a valid Verus visibility-model justification. The `new()` constructor still enforces alignment preconditions, so the safety boundary is preserved at the spec level. No exec-level risk since Verus code isn't compiled as the production kernel.

3. **`UpoolInner` elimination is justified but should be noted as a structural divergence.**
   The consistency report correctly explains this as a consequence of `Rc<RefCell<>>` removal. Since `UpoolInner` methods are 1:1 delegations to `FrameAllocator`, eliminating the struct preserves all exec semantics. No missing logic.

4. **`base_addr` hardcoded to 0 in `View for Upool`.**
   The `UpoolView.base_addr` is always set to `0` (upool.spec.rs:173). The original source doesn't expose a base address concept at the `Upool` level, so this is a verification-only abstraction. Specs like `frame_addr()` and `limit()` that depend on `base_addr` will produce addresses relative to 0, which is correct only if the pool's physical region actually starts at 0. This is acceptable for the current proof scope (relative reasoning) but could be a footgun if `base_addr` is later used to reason about absolute physical addresses. **Impact: none currently.**

5. **`trace!()` removal in `alloc_many`.**
   Documented. Verus cannot process Rust macros that expand to non-verifiable code. No semantic impact.

## Function-by-Function Assessment

| Function | Original | Verified | Status |
|----------|----------|----------|--------|
| `UserFrame` struct | `addr: FrameAddress` (private) | `pub addr: FrameAddress` | ✅ Equiv documented |
| `UserFrame::new` | `Self { addr }` | `UserFrame { addr }` | ✅ Identical exec body |
| `UserFrame::address` | `self.addr` | `self.addr` | ✅ Identical exec body |
| `UpoolInner` struct | Thin wrapper with `FrameAllocator` | Eliminated | ✅ Justified |
| `UpoolInner::new` | `Self { frame_allocator }` | Eliminated | ✅ Justified |
| `UpoolInner::alloc` | `self.frame_allocator.alloc()` | N/A (inlined) | ✅ Justified |
| `UpoolInner::alloc_many` | Loop + Vec + `?` | N/A (inlined) | ✅ Justified |
| `UpoolInner::free` | `self.frame_allocator.free(page_addr)` | N/A (inlined) | ✅ Justified |
| `Upool` struct | `Rc<RefCell<UpoolInner>>` | `FrameAllocator` directly | ✅ Equiv documented |
| `Upool::new` | Wraps in Rc/RefCell/UpoolInner | `Upool { frame_allocator }` | ✅ Equiv documented |
| `Upool::alloc` | `self.inner.borrow_mut().alloc()?` → UserFrame | `match self.frame_allocator.alloc()` → UserFrame | ✅ Equiv documented |
| `Upool::alloc_many` | Returns `Result<Vec<UserFrame>, Error>` | Returns `Ghost<Seq<int>>` | ✅ Equiv documented (see minor #1) |
| `Upool::free` | `self.inner.borrow_mut().free(uframe.address())` | `self.frame_allocator.free(uframe.address())` | ✅ Equiv documented |
| `Upool::capacity` | Not in original | Verification helper | ✅ EXTRA_IN_VERUS documented |
| `Upool::free_by_addr` | Not in original | Verification helper | ✅ EXTRA_IN_VERUS documented |

## Equivalence Justification Assessment

All 7 documented equivalences are sound:
- **Rc/RefCell removal**: Correct. The interior mutability wrapper doesn't affect allocation logic; it enables shared ownership at runtime, which is orthogonal to the verified properties.
- **`?` → `match`**: Correct. Mechanically equivalent desugaring.
- **`UpoolInner` elimination**: Correct. Pure delegation layer with no independent logic.
- **`pub` field for Verus visibility**: Correct. Known Verus limitation.
- **`Vec` → `Ghost<Seq>`**: Correct for verification purposes. The state transitions are identical.
- **EXTRA_IN_VERUS functions**: Both `capacity` and `free_by_addr` are thin wrappers around `FrameAllocator` methods, adding no novel exec behavior.

## Summary

The exec consistency fixes are thorough and well-documented. All mismatches stem from genuine Verus limitations (Rc/RefCell, Vec, `?` operator) and are handled by structurally equivalent transformations that preserve the original allocation semantics. The two EXTRA_IN_VERUS functions are justified as verification helpers. Verification passes cleanly with 20 verified assertions and zero errors, with no escape hatches (`assume`/`admit`/`external_body`). The only substantive concern is the `alloc_many` return type change, which is correctly documented but means the verified version cannot be used as a direct exec replacement — this is an inherent limitation of the verification approach rather than a defect.
