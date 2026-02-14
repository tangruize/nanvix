# Review: kpool Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Status

✅ **PASSED** — 34 verified, 0 errors.

## Issues Found

### Critical

- None.

### Minor

1. **`alloc_many` return type divergence**: The original `alloc_many` returns `Vec<KernelFrame>` with each frame carrying an `Rc<RefCell<KpoolInner>>` clone. The Verus version returns `Result<usize, Error>` (starting index). The consistency report correctly identifies the `Rc<RefCell>` limitation, but the caller loses the ability to individually track/free frames via RAII. This is acceptable since Verus uses explicit `free_range`/`free_contiguous` as replacements, but downstream callers must adapt their usage pattern. The equivalence justification is sound.

2. **`base_addr` hardcoded to 0 in View**: In `kpool.spec.rs` (line 210), the `View` impl for `Kpool` sets `base_addr: 0`. The original computes frame addresses as `region.start() + index * PAGE_SIZE`. The Verus version's frame address computation in specs (`frame_addr`) adds `base_addr + frame_idx * FRAME_SIZE`) would always produce `frame_idx * FRAME_SIZE` since `base_addr` is always 0. This is not a correctness issue for allocation/deallocation invariants (which are index-based), but it means the spec doesn't faithfully model physical address computation. The `FrameAddress` type used in exec code does carry proper addresses, so this only affects specification-level reasoning about absolute addresses.

3. **`alloc_noncontiguous` is an extra function with no original counterpart**: The report lists it as "justified extra" but it's a genuinely new API not in the original. It allocates frames one-by-one in a loop (unlike the original's contiguous `alloc_range`). While useful for verification, it represents functionality beyond the original scope. Acceptably documented.

### Observations (Non-Issues)

- The `alloc_contiguous` + `alloc_range` split of the original `KpoolInner::alloc_range` is well-motivated. `alloc_contiguous` captures the search-and-allocate semantics, while `alloc_range` captures the book-by-index semantics. The `alloc_many` wrapper correctly delegates to `alloc_contiguous`.

- The `free` function taking `KernelFrame` (with `pool_id` provenance check) instead of `FrameAddress` is a verification improvement over the original, which relied on `Rc` identity. The provenance precondition `kframe@.pool_id == old(self)@.id()` is stronger than what the original enforces at compile time.

- The `base()` function on `KernelFrame` is properly maintained alongside the new `address()` accessor, preserving API compatibility with the original.

## Review Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** All 5 mismatches (struct field renames, `Rc<RefCell>` → `pool_id`, `clear` omission, `alloc_range` split, `free` signature change) are documented with clear Verus limitation justifications. No mismatch was silently dropped.

### 2. Were MISSING functions added with proper verification?

**Partially.** `alloc_many` was added (the only actionable missing function). The remaining 4 missing functions (`clear`, `deref`, `deref_mut`, `drop`) are correctly identified as impossible to verify due to `unsafe` blocks and `Rc<RefCell>` requirements. Each omission is well-justified as orthogonal to allocation safety.

### 3. Are equivalence justifications sound?

**Yes.** The justifications accurately describe Verus limitations:
- `Rc<RefCell<T>>` → explicit `pool_id` is the standard Verus pattern for interior mutability.
- Omitting `unsafe` code (clearing, deref) is correct since Verus cannot verify `unsafe`.
- Splitting `alloc_range` into search + book is a sound decomposition.
- Replacing `Drop` with explicit `free()` is necessary and well-documented.

### 4. Does the exec code now faithfully represent the original source?

**Yes, within Verus limitations.** The core allocation logic (bitmap-based frame tracking, single alloc, contiguous range alloc, free) is faithfully represented. The structural differences (`pool_id` vs `Rc<RefCell>`, explicit free vs Drop, no clearing) are all necessitated by Verus's inability to model unsafe code, interior mutability, and RAII. The `alloc_many` wrapper correctly maps to the original API name with equivalent contiguous allocation semantics.

### 5. Does verification still pass?

**Yes.** 34 verified, 0 errors (up from 33 before the `alloc_many` addition).

## Summary

The exec consistency fix is well-executed. The single added function (`alloc_many`) correctly wraps `alloc_contiguous` with matching specs, and the 14 documented equivalences are thorough and technically sound. The Verus limitations (no `Rc<RefCell>`, no `unsafe`, no `Drop`) are genuine and the workarounds (explicit `pool_id`, explicit `free()`, omitting `clear`/`deref`) are the standard approach in Verus verification. The only minor concern is the hardcoded `base_addr: 0` in the View, which limits spec-level address reasoning but doesn't affect allocation safety properties.
