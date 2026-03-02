# Review: slab Proof Extraction (claude-opus-4.6)

## Grade: C

## Issues Found

### Critical

- **12 proof blocks >5 lines remain inline in exec code (450 lines, 48.5% of file).** The core task — extracting >5-line proof blocks into lemmas — was largely **not done** for function-local proof blocks. The biggest offenders:
  - `deallocate` Ok-path: **128 lines** inline (lines 777–904). This block contains cardinality reasoning, subset proofs, insert/remove axioms, and strict-subset logic. All of this should be in a lemma like `lemma_deallocate_ok_can_allocate`.
  - `from_raw_parts` postconditions: **72 lines** inline (lines 361–432). Memory bounds, distributive property, alignment, and freshness proofs should be in `lemma_from_raw_parts_postconditions`.
  - `from_raw_parts` layout computation: **56 lines** inline (lines 233–288). Division/multiplication bounds reasoning belongs in `lemma_layout_bounds`.
  - `allocate` error path: **51 lines** inline (lines 504–554). View-equality and invariant-preservation reasoning should be `lemma_allocate_error_preserves_state`.
  - `allocate` postconditions: **46 lines** inline (lines 605–650). Address validity and allocation-status proofs should be `lemma_allocate_postconditions`.
  - `deallocate` bounds: **32 lines** inline (lines 712–743). Overflow and index-range reasoning should be `lemma_deallocate_index_bounds`.

- **Exec code is not readable with proof blocks in place.** The `from_raw_parts` function is ~330 lines where ~200 are proof. The `deallocate` function is ~220 lines where ~190 are proof. A reader cannot follow the exec logic without mentally skipping massive proof blocks.

### Major

- **Extracted lemmas are mostly utility/structural, not function-specific proof blocks.** The 45 lemmas in `lib.proof.rs` fall into these categories:
  - 13 power-of-two lemmas (constant witnesses, not extracted blocks)
  - 6 arithmetic utility lemmas (`lemma_mul_divisible`, `lemma_div_cancel`, etc.)
  - 12 structural lemmas (`lemma_inv_from_components`, `lemma_view_fields`, `lemma_allocated_blocks_finite`, etc.)
  - 8 property lemmas (`lemma_blocks_disjoint`, `lemma_no_memory_aliasing`, etc.)
  - 6 proof tests (`test_*` functions)

  While these are valuable, they were likely **pre-existing or independently created**, not extracted from inline proof blocks. The actual inline blocks that dominate the exec file were not refactored.

- **`lemma_allocate_adds_block` and `lemma_deallocate_removes_block` are trivial (ensures == requires).** Their postconditions repeat their preconditions verbatim — they prove nothing beyond what's assumed. These should either have meaningful postconditions or be removed.

### Minor

- **Naming is generally good.** Lemma names like `lemma_can_allocate_implies_bitmap_has_free_bit` and `lemma_dealloc_from_full_enables_alloc` are descriptive and follow consistent `lemma_` prefix convention.
- **Some lemmas have overly broad requires.** For example, `lemma_dealloc_from_full_enables_alloc` takes 9 preconditions — some could be bundled via `inv()`.
- **`lemma_mul_distribute` duplicates `lemma_distributive`.** Both prove distributive-property variants. One should be removed or unified.
- **Power-of-two lemma chain (13 lemmas) is boilerplate.** A single parameterized lemma with a `decreases` clause would be cleaner.

## Verification

- **Passes: 80 verified, 0 errors.** Verification is sound.

## Summary

The proof file (`lib.proof.rs`) contains well-named, well-structured lemmas for utility arithmetic, structural invariants, and high-level properties (disjointness, liveness, etc.). However, the primary objective — extracting large inline proof blocks into lemmas so the exec code reads cleanly — was **not accomplished**. All 12 proof blocks exceeding 5 lines remain inline in `lib.rs`, totaling 450 lines (48.5% of the file). The exec functions `from_raw_parts`, `allocate`, and `deallocate` are dominated by proof code, making the actual execution logic very difficult to follow. A successful extraction would move these blocks into descriptive lemmas in `lib.proof.rs` and replace them with single-line lemma calls in the exec code.
