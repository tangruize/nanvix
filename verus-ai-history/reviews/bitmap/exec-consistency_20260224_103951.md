# Review: bitmap Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Status
- **libs::bitmap**: 85 verified, 0 errors ✅
- Command: `verus --crate-type lib lib.rs --verify-module libs::bitmap`

## Issues Found

### Critical
- None.

### Minor
1. **`from_raw_array` precondition relaxation changes caller contract.** The original source has no preconditions on the array contents (the caller passes `mut array` and the function zeroes it). The old Verus code required `forall|i| array@[i] == 0` as a precondition, pushing zeroing responsibility to the caller. The fix correctly removes this precondition and restores the zeroing loop, matching the original. However, existing callers that relied on the old precondition contract should be verified — the consistency report confirms slab (83 verified) and frame (26 verified) still pass, so this is safe.

2. **`alloc_range` incremental usage update.** The original does `self.usage += size` atomically after the allocation loop; the Verus version increments `self.usage = self.usage + 1` on each iteration. This is semantically equivalent (same final value) and necessary for the loop invariant to track `self.inv()` at each step. The justification is sound, but it does mean the intermediate states differ from the original (usage is partially updated mid-loop). Since this is a single-threaded context with no observable side effects during the loop, this is acceptable.

3. **Bounds check in inner loop of `alloc_range`.** The Verus code adds `if idx >= self.number_of_bits { ... }` inside the inner scan loop. The original has no such check because the outer loop condition `start <= self.number_of_bits - size` combined with `offset < size` guarantees `idx < self.number_of_bits`. The added check is dead code (unreachable at runtime) introduced solely for verification — Verus needs it to prove the precondition of `test_unchecked`. This is harmless but should be documented as unreachable.

### Observations (Non-issues)
- `for` → `while` conversions are correct and necessary (Verus has no iterator support).
- `self.bits[w] |= x` → `self.bits.set(w, self.bits[w] | x)` is correct (Verus has no compound assignment on indexed arrays).
- `is_multiple_of()` → `% ... != 0` is semantically identical.
- `start += offset + 1` → `start = idx + 1` where `idx = start + offset` is algebraically identical.
- Parameter renames (`index` → `bit_index`) avoid shadowing and have no semantic impact.
- Intermediate variable bindings in `test` (`byte_val`, `result_val`) are semantically transparent.

## MISMATCH Functions Review

| Function | Status | Assessment |
|----------|--------|------------|
| `new` | ✅ FIXED | Zeroing loop correctly restored. `while` replaces `for` (Verus limitation). `% != 0` replaces `is_multiple_of()` (equivalent). Proof block correctly establishes `inv()` post-zeroing. |
| `from_raw_array` | ✅ FIXED | Zeroing loop restored, `mut` parameter restored, caller-side precondition removed. Contract now matches original: function accepts any array and zeroes it. |

## EQUIVALENT Functions Review

| Function | Status | Assessment |
|----------|--------|------------|
| `number_of_bits` | ✅ Sound | Identical logic, only added requires/ensures. |
| `alloc` | ✅ Sound | Identical delegation to `alloc_range(1)`. Proof block establishes liveness precondition. |
| `alloc_range` | ✅ Sound | Core search-and-allocate logic preserved. All structural differences (for→while, compound assignment, incremental usage, bounds guard) are well-justified Verus adaptations. Fast-skip path preserved. Error messages and codes match original. |
| `set` | ✅ Sound | Identical logic flow (test → index → bitwise OR → increment). Verus syntax adaptations only. |
| `clear` | ✅ Sound | Mirror of `set` with AND-NOT and decrement. Matches original exactly. |
| `test` | ✅ Sound | Same bitwise test logic. Intermediate bindings are cosmetic. |
| `index` | ✅ Sound | Identical bounds check + delegation to `index_unchecked`. Rename is non-semantic. |
| `index_unchecked` | ✅ Sound | Identical arithmetic. Rename is non-semantic. |

## Extra Functions Review

| Function | Status | Assessment |
|----------|--------|------------|
| `new_managed` | ✅ Acceptable | Thin wrapper around `new()` with stronger preconditions for slab API. No exec divergence. |
| `usage` | ✅ Acceptable | Simple getter needed by verified callers. Not in original but harmless. |
| `clear_range` | ✅ Acceptable | Bulk deallocation extension. Follows same pattern as `clear`. Used by slab/frame allocators. |
| `test_unchecked` | ✅ Acceptable | Extracted helper for verification. Combines `index_unchecked` + bit test. |
| Verified test functions (10) | ✅ Acceptable | Proof-level tests that verify bitmap properties. Not executable. Good coverage: set/clear, alloc/dealloc, range operations, usage tracking, frame preservation, cross-boundary ranges. |

## Spec and Proof Quality

- **Spec model** uses `Set<int>` as the primary abstraction, which is clean and avoids expensive `Seq<bool>` axioms. The `BitmapView` struct with `set_bits: Set<int>` elegantly captures the bitmap's logical state.
- **Invariant (`inv()`)** is comprehensive: covers well-formedness, finiteness, usage tracking, and structural consistency.
- **Proof lemmas** are well-organized into categories (finiteness, cardinality, free range properties, view synchronization, bit-level operations). The bit-vector proofs (`by (bit_vector)`) correctly handle low-level bitwise reasoning.
- **Frame conditions** are strong: both pointwise (`forall|i| ... self.is_bit_set(i) == old(self).is_bit_set(i)`) and set-based (`self@.set_bits =~= old(self)@.set_bits.insert(...)`) postconditions ensure callers can reason about unchanged bits.

## Summary

The exec consistency fixes are well-executed. The two MISMATCH functions (`new` and `from_raw_array`) have been correctly restored to match the original source's zeroing behavior, with appropriate Verus adaptations (while loops instead of for-each iterators). The eight EQUIVALENT functions faithfully preserve the original's exec logic with only necessary Verus syntax transformations. All equivalence justifications are sound. The extra functions (5 utility + 10 verified tests) are justified additions that don't alter the core bitmap semantics. Verification passes with 85/0, and downstream modules (slab: 83/0, frame: 26/0) are confirmed compatible. The only area for minor improvement would be adding a comment marking the inner bounds check in `alloc_range` as unreachable dead code for documentation clarity.
