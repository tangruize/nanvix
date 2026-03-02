# Review: slab Proof Extraction (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### Major
- None.

### Minor

1. **Three remaining inline proof blocks exceed 5 lines (borderline).**
   Lines 225–231, 250–255, and 392–397 in `lib.rs` are 6–7 lines each. However, the actual proof content is a single lemma call with multi-line argument formatting. These are acceptable as-is — extracting a 1-call wrapper lemma would add indirection with no readability benefit. The report's claim of "~440 → ~17 lines" is slightly misleading since these blocks span ~19 lines total, but the spirit is correct.

2. **`lemma_from_raw_parts_layout_bounds` has a wide parameter list (7 params).**
   This lemma takes `len`, `block_size`, `total_num_blocks`, `index_len`, `num_index_blocks`, `num_data_blocks`, and `addr` as separate `int` parameters. An alternative design would pass `&Slab` or a struct, but since `from_raw_parts` hasn't constructed the slab yet, the flat parameter list is justified. Still, it makes the requires clause verbose and harder to audit.

3. **`lemma_from_raw_parts_post_loop` has 13 requires clauses.**
   The precondition surface is quite large. Some clauses (e.g., `len > 0`, `len < i32::MAX as int`) are forwarded from the outer function and could be collapsed, but the ensures are tight and correct so this is a minor verbosity issue.

4. **Low-confidence trigger warnings on lines 1355 and 1538.**
   Verus emits automatic trigger warnings for two `assert forall` expressions in `lemma_alloc_establishes_postconditions` and `lemma_dealloc_clear_ok_postconditions`. These should be annotated with `#![auto]` to suppress the warnings, since the auto-chosen triggers are correct here.

### Observations (non-issues)

- **Naming conventions are consistent and descriptive.** All extracted lemmas follow the pattern `lemma_{function}_{purpose}` (e.g., `lemma_alloc_error_preserves_state`, `lemma_dealloc_offset_bounds`). This aligns well with existing lemma naming in the file.

- **Requires/ensures are appropriately tight.** The lemmas capture exactly the state available at each extraction point (bitmap postconditions, field equality, prior invariants) without over-generalizing. The ensures clauses match what the exec code needs at each call site.

- **Exec code reads cleanly.** `from_raw_parts`, `allocate`, and `deallocate` now have a clear flow: runtime logic interleaved with single-line `proof { Self::lemma_...(...); }` calls. The proof blocks no longer obscure the algorithmic structure.

- **Pre-existing lemmas are properly reused.** The extracted lemmas call `lemma_inv_from_components`, `lemma_view_fields`, `lemma_can_allocate_implies_bitmap_has_free_bit`, and other established helpers rather than duplicating proof logic.

- **Verification passes cleanly.** 92 verified, 0 errors. The 11 additional verified items (vs. baseline 81) correspond exactly to the 11 extracted lemma functions.

## Checklist

| Criterion | Pass | Notes |
|-----------|------|-------|
| All >5 line proof blocks extracted? | ✅* | Three 6-7 line blocks remain but contain only single lemma calls with formatted args — acceptable |
| Lemma names descriptive and consistent? | ✅ | `lemma_{function}_{purpose}` pattern throughout |
| Requires/ensures tight? | ✅ | Preconditions match extraction-point state; postconditions match what exec code needs |
| Exec code reads cleanly? | ✅ | Proof blocks replaced by single-line lemma calls; algorithmic flow is clear |
| Verification passes? | ✅ | 92 verified, 0 errors |

## Summary

The extraction is well-executed. 11 proof blocks (totaling ~460 lines) were extracted into descriptive, well-scoped lemmas, reducing the exec file from heavily interleaved proof/exec to clean algorithmic code with minimal proof call sites. The lemma contracts are tight — preconditions capture exactly the available state, and postconditions provide exactly what subsequent exec code requires. The three remaining "borderline" blocks are justifiably kept inline. The only actionable improvement is adding `#![auto]` annotations to suppress two low-confidence trigger warnings. Verification is clean at 92/0.
