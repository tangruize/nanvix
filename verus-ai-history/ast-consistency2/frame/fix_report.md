# Exec Consistency Fix: frame

## Summary
- Mismatches fixed: 2
- Missing functions added: 0 (was already present, misdetected by AST tool)
- Documented equivalences: 4

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `book` [book.diff](book.diff) [book_source.rs](book_source.rs) [book_verus.rs](book_verus.rs) | **RESTORED original logic** | Removed idempotency check (test-before-set). Original directly calls `bitmap.set()`, which returns `ResourceBusy` if frame already allocated. Verus version had added a `bitmap.test()` guard that silently succeeded on already-allocated frames, changing observable behavior. Specs updated to reflect non-idempotent semantics. |
| `from_raw_storage` [from_raw_storage.diff](from_raw_storage.diff) [from_raw_storage_source.rs](from_raw_storage_source.rs) [from_raw_storage_verus.rs](from_raw_storage_verus.rs) | **RESTORED original logic** | Restored to `Ok(Self::new(Bitmap::from_raw_array(storage)))` matching original. Verus version had inlined the `new()` constructor body and added a separate proof block. Now delegates to `new()` as the original does; proof obligations discharged by `new()`'s postconditions. |
| `alloc_range` [alloc_range.diff](alloc_range.diff) [alloc_range_source.rs](alloc_range_source.rs) [alloc_range_verus.rs](alloc_range_verus.rs) | **Renamed to match original API** | Renamed `alloc_range_from_region(region)` → `alloc_range(region)` to match the original Nanvix signature `fn alloc_range(&mut self, region: &TruncatedMemoryRegion<PhysicalAddress>)`. The internal helper that takes `(start_frame, count)` was renamed from `alloc_range` → `alloc_range_checked` to avoid name collision. Updated call site in kpool.rs. |
| `alloc_range_checked` (bitmap.test Err branch) | **Documented unreachability** | Replaced empty proof block with `// VERIFIED: unreachable` comment. `bitmap.test()` guarantees `Ok` when `index < number_of_bits`, and `idx < end_frame <= capacity == number_of_bits`, so the `Err` path is provably unreachable. |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | **Documented equivalence** | Exec logic is identical (`Self { bitmap }` + return). The original also calls `info!()` to log capacity; `info!()` macro is unavailable in Verus (known logging limitation). Added doc comment noting this. |
| `alloc` [alloc.diff](alloc.diff) [alloc_source.rs](alloc_source.rs) [alloc_verus.rs](alloc_verus.rs) | **Documented equivalence** | The original performs 3 steps: (1) `bitmap.alloc()`, (2) `FrameNumber::from_raw_value()` runtime bounds check, (3) `FrameAddress::from_frame_number()`. The Verus version bypasses step (2) by directly constructing `FrameNumber { value: frame_idx }`, proven safe via invariant (`capacity <= MAX_FRAME_NUMBER + 1 ∧ frame_idx < capacity ⟹ frame_idx <= MAX_FRAME_NUMBER`). This is a verified optimization eliminating a redundant runtime check. Already documented in code with `VERIFIED OPTIMIZATION` comment. Logging uses `error.log()` instead of `error!()` (Verus limitation). |
| `free` [free.diff](free.diff) [free_source.rs](free_source.rs) [free_verus.rs](free_verus.rs) | **Documented equivalence** | Exec logic is identical: `frame.into_frame_number().into_raw_value()` then `bitmap.clear()`. Only difference is `error!("{error:?} (frame={frame:?})")` → `error.log()` (Verus logging limitation). Already noted in code comment. |
| `from_raw_storage` (AST: MISSING) | **Not actually missing** | The AST tool reported this as MISSING_IN_VERUS, but it was present. The AST mismatch was due to different function body structure (inlined `new()` body vs calling `new()`). Now fixed to call `new()` to match original. |

## Extra Functions in Verus (Documented)
| Function | Justification |
|----------|---------------|
| `alloc_index` [alloc_index_verus.rs](alloc_index_verus.rs) | Helper returning raw frame index (usize) for internal use. Extracted for proof modularity — `alloc()` builds on top of this. |
| `alloc_range_inner` [alloc_range_inner_verus.rs](alloc_range_inner_verus.rs) | Loop body extracted from `alloc_range_checked` for Verus loop invariant verification. Marked `#[inline]` to avoid runtime overhead. |
| `alloc_range_unchecked` [alloc_range_unchecked_verus.rs](alloc_range_unchecked_verus.rs) | seL4-style variant requiring caller proof that all frames are free, eliminating runtime checks. Verification extension. |
| `alloc_range_checked` [alloc_range_checked_verus.rs](alloc_range_checked_verus.rs) | Internal helper implementing the check-then-set logic of the original `alloc_range`. Takes `(start_frame, count)` instead of region. |
| `alloc_contiguous_range` [alloc_contiguous_range_verus.rs](alloc_contiguous_range_verus.rs) | Extension for searching and allocating contiguous free ranges. Uses `bitmap.alloc_range()`. |
| `capacity` [capacity_verus.rs](capacity_verus.rs) | Simple getter delegating to `bitmap.number_of_bits()`. Used in specs. |
| `free_range` [free_range_verus.rs](free_range_verus.rs) | Symmetric counterpart to `alloc_range_unchecked` for freeing frame ranges. |
| `free_range_inner` [free_range_inner_verus.rs](free_range_inner_verus.rs) | Loop body extracted from `free_range` for Verus loop invariant verification. |

## Verification: PASS
- frame: 26 verified, 0 errors
- kpool: 33 verified, 0 errors (updated call site)
- No assume, admit, or unjustified external_body added
