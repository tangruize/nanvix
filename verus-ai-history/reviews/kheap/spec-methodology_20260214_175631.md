# Review: kheap Spec Methodology (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **`view()` missing explicit `pub` keyword** (kheap.spec.rs:231): The `view()` function is declared as `closed spec fn view(...)` without an explicit `pub` modifier. While it implements the `View` trait (so visibility is inherited from the trait and effectively public), the guidelines recommend explicitly declaring it as `pub closed spec fn` for clarity and consistency. This is cosmetic — Verus treats it correctly.

### Low
- **Proof function uses direct field access** (kheap.proof.rs:832): `test_heap_extent_respected_verified` ensures clause references `heap.base_addr@` and `heap.total_size@` (direct Kheap field access) instead of `heap@.base_addr` and `heap@.total_size` (through the view). Per the methodology, proof functions are permitted to reference internals, so this is acceptable but slightly inconsistent with the style used in public method specs. The same proof function uses `heap@.is_valid_heap_addr(addr)` correctly in its requires clause, mixing the two conventions.

## Detailed Assessment

### 1. View types use abstract types ✅
`KheapView` (kheap.spec.rs:38-59) is declared as a `ghost struct` using only abstract types: `SlabView` for slab fields and `int` for `base_addr` and `total_size`. No concrete types (`usize`, `Vec`, etc.) appear in the view. `SlabView` is itself an abstract ghost type from the slab module.

### 2. `view()` declared as `pub closed spec fn` ✅ (minor)
The `view()` function (kheap.spec.rs:231) is `closed spec fn` implementing the `View` trait. It correctly maps concrete `Kheap` fields to `KheapView` using the `@` operator for sub-views. Missing explicit `pub` is noted above but functionally correct.

### 3. `inv()` exists and is `pub closed spec fn` ✅
`Kheap::inv()` (kheap.spec.rs:268-296) is properly declared as `pub closed spec fn`. It comprehensively validates:
- All 8 individual slab invariants.
- Correct block sizes for each slab.
- Disjoint memory regions (`all_slabs_disjoint()`).
- Slabs within heap extent (`all_slabs_within_extent()`).
- Proper alignment (`all_slabs_aligned()`).
- Valid base address and total size (> 0).

`SlabSize::inv()` (kheap.spec.rs:15-17) is also `pub closed spec fn`, trivially `true` for the simple enum.

### 4. Public method specs avoid `self.field` ✅
All three public methods (`from_raw_parts`, `allocate`, `deallocate`) use `self@` (the view) in their requires/ensures clauses:
- `self@.is_empty()`, `self@.get_slab(slab_size)`, `self@.base_addr`, etc.
- No direct `self.slab_8_bytes` or similar concrete field access in public method specs.
- Direct field access (`self.slab_8_bytes`) only appears in private proof lemmas (e.g., `lemma_inv_implies_slab_invs`), which is correct per the methodology.

### 5. Public methods require/ensure `inv()` ✅
- `from_raw_parts`: ensures `heap.inv()` (kheap.rs:232).
- `allocate`: requires `old(self).inv()` (kheap.rs:557), ensures `self.inv()` (kheap.rs:559).
- `deallocate`: requires `old(self).inv()` (kheap.rs:673), ensures `self.inv()` (kheap.rs:683).
- `init`: ensures `heap.inv()` (kheap.rs:780).

Frame conditions are properly specified for both success and error paths.

### 6. No remaining assume/admit/unjustified external_body ✅
Grep confirms no `assume`, `admit`, or `#[verifier::external_body]` in any of the three kheap files. All proofs are fully discharged.

### 7. Verification passes ✅
Verification succeeds: **43 verified, 0 errors** in ~10 seconds.

## Additional Observations

### Strengths
- **Excellent view design**: `KheapView` properly abstracts away implementation details while exposing meaningful properties (slab views, disjointness, extent, alignment, allocation state).
- **Rich spec helpers on `KheapView`**: Functions like `get_slab()`, `total_allocated()`, `total_capacity()`, `is_empty()`, `can_allocate_in_slab()`, `all_slabs_disjoint()`, `is_valid_heap_addr()` are all `pub open spec fn` on the view type, following the guideline to put reusable spec helpers on the view.
- **Strong liveness guarantees**: Both `allocate` and `deallocate` have liveness postconditions (allocation succeeds if slab has capacity; deallocation always succeeds given valid preconditions).
- **Frame conditions**: All public methods specify exactly which slabs change and which remain unchanged, enabling precise reasoning by callers.
- **No soundness gaps**: Zero assumes, admits, or external_body annotations.
- **Well-structured proofs**: Lemmas in the proof file are cleanly organized with documentation, and proof obligations are fully discharged.
- **SMT optimization note**: The `can_allocate_in_slab()` helper includes a comment explaining its necessity for SMT term sharing (kheap.spec.rs:100-107), showing awareness of verification performance.

### Design Notes
- `SlabSize::inv()` is trivially `true`. This is acknowledged in the documentation and is appropriate for a simple enum with no internal consistency requirements.
- The `spec_slabs_disjoint` static helper on `Kheap` (kheap.spec.rs:250-256) duplicates `KheapView::slabs_disjoint` (kheap.spec.rs:157-163). This appears intentional for proof convenience but could be consolidated.

## Summary

The kheap module demonstrates exemplary adherence to the spec methodology guidelines. The `KheapView` abstraction uses only ghost/abstract types, `view()` and `inv()` are properly closed, public method specifications consistently use the view (`self@`) rather than concrete fields, and invariants are required/ensured across all public methods. All 43 verification conditions pass with zero soundness gaps. The only issues are cosmetic: a missing explicit `pub` on the trait-inherited `view()` and a minor style inconsistency in one proof function.
