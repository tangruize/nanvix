# Review: frame Exec Consistency (claude-opus-4.6)

## Grade: A-

## Verification Status
- **26 verified, 0 errors** — full verification passes.
- No `assume`, `admit`, or unjustified `external_body` found.

## Issues Found

### Critical
- None.

### Minor

1. **`alloc_range` signature diverges from original API.**
   The original Nanvix `alloc_range` takes `&TruncatedMemoryRegion<PhysicalAddress>` as its parameter. The Verus version splits this into two functions: `alloc_range(start_frame, count)` (core logic) and `alloc_range_from_region(region)` (API wrapper). While `alloc_range_from_region` preserves the original API shape, the function named `alloc_range` no longer matches the original signature. This is a naming concern, not a correctness issue — the wrapper delegates correctly. The consistency report documents this refactoring adequately.

2. **`alloc_range` error path for `bitmap.test()` failure (lines 679-688).**
   When `bitmap.test(idx)` returns `Err`, the function returns the error but the proof block is empty — it cannot satisfy the `exists` postcondition (that some frame in range is allocated). This is a proof gap. In practice, the `bitmap.test()` preconditions guarantee this branch is unreachable (idx < capacity is always satisfied), but Verus does not formally prove unreachability here. The postcondition `result is Err ==> exists|i| ... old(self)@.is_allocated(i)` may not be discharged for this path. However, since Verus reports 26 verified / 0 errors, either: (a) the bitmap.test postcondition guarantees it never fails given valid idx, or (b) the verifier can discharge this through other reasoning. Acceptable but warrants a `// VERIFIED: unreachable` comment.

3. **Logging fidelity.**
   All `error!()` macro calls replaced with `error.log()`. The original includes contextual formatting (e.g., `error!("{error:?} (frame={frame:?})")`), while `error.log()` loses the contextual parameters. This is a known Verus limitation, documented in the consistency report and in code comments. Acceptable.

### Observations (Non-Issues)

1. **`book` correctly restored.** The original calls `bitmap.set()` directly and returns `ResourceBusy` (or similar) if the frame is already allocated. The Verus version now matches: no `bitmap.test()` guard, direct `bitmap.set()`. The postcondition correctly specifies that failure implies the frame was already allocated (`result is Err ==> old(self)@.is_allocated(...)`). This is a faithful restoration.

2. **`from_raw_storage` correctly restored.** Now calls `Ok(Self::new(Bitmap::from_raw_array(storage)))`, matching the original exactly. The previous version had inlined the `new()` body.

3. **`new` equivalence is sound.** Only difference is the missing `info!()` log. Exec logic (`Self { bitmap }`) is identical. The proof block establishing `is_freshly_initialized` is a verification-only addition.

4. **`alloc` verified optimization is sound.** The original does a runtime bounds check via `FrameNumber::from_raw_value()`. The Verus version proves `frame_idx <= MAX_FRAME_NUMBER` from the invariant (`capacity <= MAX_FRAME_NUMBER + 1 ∧ frame_idx < capacity`) and constructs `FrameNumber` directly. This eliminates a redundant check that can never fail. The optimization is well-documented with a `VERIFIED OPTIMIZATION` comment and the proof assertion is explicit.

5. **`free` equivalence is sound.** Exec logic is identical: `frame.into_frame_number().into_raw_value()` then `bitmap.clear()`. Only `error!()` → `error.log()` difference.

6. **`alloc_range` two-phase check-then-set logic preserved.** The original iterates with `for..=` (inclusive range) using `start + size/FRAME_SIZE - 1`. The Verus version uses `while idx < end_frame` with `end_frame = start + count` (exclusive). The arithmetic is equivalent: original's `start_frame_number..=end_frame_number` where `end = start + size/FRAME_SIZE - 1` covers the same indices as `start..(start + count)` where `count = size/FRAME_SIZE`. Both check all frames first, then set all frames.

7. **Extra functions are justified.** `alloc_index`, `alloc_range_inner`, `free_range_inner` are extracted for proof modularity. `alloc_range_unchecked`, `alloc_contiguous_range`, `free_range` are verification extensions not in the original. `capacity` is a simple getter. `alloc_range_from_region` is the API wrapper. All are well-documented.

## Criteria Assessment

| Criterion | Assessment |
|-----------|-----------|
| 1. MISMATCH functions restored | ✅ `book` and `from_raw_storage` properly restored to match original semantics. |
| 2. MISSING functions added | ✅ `from_raw_storage` was not actually missing (AST tool misdetection); correctly identified and documented. |
| 3. Equivalence justifications sound | ✅ All four documented equivalences (`new`, `alloc`, `free`, `alloc_range`) have sound reasoning. The `alloc` verified optimization is mathematically justified. |
| 4. Exec code faithful to original | ✅ Core functions (`new`, `from_raw_storage`, `alloc`, `free`, `book`) faithfully represent the original. `alloc_range` is refactored but semantically equivalent via `alloc_range_from_region` wrapper. |
| 5. Verification passes | ✅ 26 verified, 0 errors. No assumptions or admits. |

## Summary

The exec consistency fixes are well-executed. The two MISMATCH functions (`book` and `from_raw_storage`) were properly restored to match original Nanvix semantics — `book` no longer has the idempotency guard that changed observable behavior, and `from_raw_storage` delegates to `new()` as the original does. The four documented equivalences are sound, with the `alloc` verified optimization being the most notable departure (eliminating a provably-redundant runtime bounds check). The `alloc_range` refactoring into separate check-then-allocate functions with an API wrapper is a reasonable structural change for Verus verification needs. Verification passes cleanly at 26/0. The only minor concerns are the empty proof block in the `bitmap.test()` error path of `alloc_range` and the `alloc_range` naming divergence from the original signature.
