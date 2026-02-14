# Review: kheap Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Status

**PASS**: 43 verified, 0 errors (confirmed by running `./verus-ai/scripts/verify.sh kheap`).

## Issues Found

### Critical

- None.

### Minor

1. **`layout_to_slab_size` range gap not explicitly documented in exec code**: The original `layout_to_allocator` has the same behavior (sizes 513–4095 return error), but neither the original nor the Verus version has a comment in the match body explaining this intentional gap. The proof file has `lemma_size_gap_returns_error` which documents it at the proof level, which is adequate. Not a consistency issue.

2. **`MIN_SLAB_SIZE` and `MIN_HEAP_SIZE` are hardcoded literals**: The original defines `MIN_SLAB_SIZE = SLAB_COUNT * mem::PAGE_SIZE` and `MIN_HEAP_SIZE = NUM_OF_SLABS * MIN_SLAB_SIZE` as expressions. The Verus version hardcodes `131072` and `1048576` with comments showing the derivation. This is a Verus limitation (const expressions involving multiplication are not always supported), and the values are correct: `32 * 4096 = 131072`, `8 * 131072 = 1048576`. Acceptable.

3. **`from_raw_parts` validation checks moved to preconditions**: The original performs three runtime checks (alignment, minimum size, size-is-multiple) and returns errors. The Verus version moves these to `requires` clauses. This is the standard Verus pattern — the checks are still enforced, just shifted to the caller. The `slab_size < MIN_SLAB_SIZE` check is still present as a runtime guard (line 291), preserving one defense-in-depth check. Sound approach.

## Detailed Analysis

### 1. MISMATCH Functions — Properly Restored or Equivalence Documented?

| Function | Status | Assessment |
|----------|--------|------------|
| `from_raw_parts` | ✅ Documented | Core logic preserved: computes `slab_size = size / NUM_OF_SLABS`, constructs 8 slabs at consecutive offsets `addr + i * slab_size`. Uses `Slab::from_raw_parts_at_offset` instead of pointer arithmetic — semantically identical. Ghost fields added for spec only. |
| `allocate` | ✅ Documented | Dispatch logic identical: `layout_to_slab_size(size)` → match on `SlabSize` → call `slab.allocate()`. Type changes (`Layout` → `usize`, `*mut u8` → `usize`, `AllocError` → `Error`) are necessary Verus adaptations. |
| `deallocate` | ✅ Documented | Same pattern as `allocate`: dispatch to correct slab via `layout_to_slab_size`, then `slab.deallocate(addr)`. Type changes parallel `allocate`. |
| `init` | ✅ Documented | Changed from parameterless (reads `static mut HEAP_STORAGE`) to explicit `(addr, size)` parameters. Body is identical: delegates to `Kheap::from_raw_parts(addr, size)`. |

### 2. MISSING Functions — Added or Justified?

| Function | Status | Assessment |
|----------|--------|------------|
| `alloc` (GlobalAlloc) | ✅ Justified NOT ADDED | Thin wrapper accessing `static mut HEAP` and delegating to `Kheap::allocate`. Requires `GlobalAlloc` trait impl, `static mut`, and `*mut u8` — none modelable in Verus. The verified `allocate` covers the core logic. |
| `dealloc` (GlobalAlloc) | ✅ Justified NOT ADDED | Same justification as `alloc` — delegates to verified `deallocate`. |
| `layout_to_allocator` | ✅ Replaced by `layout_to_slab_size` | Match arms are identical: `1..=8 → Slab8`, ..., `4096 → Slab4096`, `_ → Err`. Only differs in parameter type (`&Layout` → `usize`) and error type. Verified with `spec_layout_to_slab_size` spec function. |

### 3. Equivalence Justifications Sound?

**Yes.** All type changes are necessitated by Verus limitations:
- `*mut u8` → `usize`: Verus cannot model raw pointers. Integer addresses are semantically equivalent for address arithmetic.
- `Layout` → `usize`: The original only uses `layout.size()`, so passing the size directly is equivalent.
- `AllocError` → `Error`: Both represent allocation failure. Error wrapping with `.map_err(|_| AllocError)` is faithfully elided since the Verus `Slab` already returns `Error`.
- `static mut` → explicit parameters: Standard Verus adaptation. The logic flow is preserved.
- Ghost fields (`base_addr`, `total_size`): Erased at compile time, no runtime effect. Needed for specification of extent and containment properties.

### 4. Does Exec Code Faithfully Represent Original?

**Yes**, with well-documented adaptations. The core algorithmic logic is preserved:
- **Slab selection**: Same 8-arm match with identical size ranges.
- **Heap construction**: Same pattern of dividing memory into 8 equal slabs at consecutive offsets.
- **Allocation dispatch**: Same `size → slab_size → slab.allocate()` pattern.
- **Deallocation dispatch**: Same `size → slab_size → slab.deallocate(addr)` pattern.
- **Constants**: `NUM_OF_SLABS = 8`, `SLAB_COUNT = 32`, `PAGE_SIZE = 4096` — all match original.
- **SlabSize enum**: All 8 variants with identical discriminant values.

The Verus code additionally provides:
- Strong postconditions on `allocate` (address validity, alignment, frame conditions).
- Strong postconditions on `deallocate` (block freed, frame conditions).
- Liveness guarantees (`can_allocate` → allocation succeeds).
- Disjointness proofs between slabs.
- Alignment proofs for all slab base addresses.

### 5. Verification Quality

The proof infrastructure is thorough:
- **`kheap.spec.rs`**: `KheapView` abstraction with `inv()`, disjointness, extent, alignment specs.
- **`kheap.proof.rs`**: 10+ lemmas covering modular arithmetic, slab block divisibility, capacity conservation, liveness propagation, and the size-gap property.
- **Test functions**: Both exec (`test_layout_to_slab_size_verified`, `test_slab_size_as_usize_verified`) and proof tests (`test_slab_size_ordering_verified`, `test_spec_layout_to_slab_coverage_verified`) exercise the verified code.

## Summary

The kheap exec consistency fixes are well-executed. All 4 MISMATCH functions have semantically equivalent Verus implementations with thorough documentation of every adaptation. All 3 MISSING functions are justified as unverifiable thin wrappers around already-verified core logic. The equivalence justifications are technically sound — every type change is necessitated by a concrete Verus limitation (raw pointers, `Layout`, `GlobalAlloc`, `static mut`). The verification passes cleanly with 43 verified properties and 0 errors. The proof infrastructure is comprehensive, covering correctness, disjointness, alignment, and liveness properties that go beyond the original unverified code.
