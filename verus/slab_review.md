# Slab module review

## Overview
`verus/slab.rs` still contains redundant proofs and checks. Below are the remaining issues to address after the recent reductions.

## Remaining issues
- **Power-of-two ladder**: Eight separate `lemma_power_of_two_*` witnesses are still present. Collapse into one generic lemma or rely on `spec_is_power_of_two` plus `vstd` arithmetic lemmas.
- **Duplicated arithmetic helpers**: Local lemmas (`lemma_mul_divisible`, `lemma_div_cancel`, `lemma_mul_inequality`, `lemma_div_mul_le`, `lemma_distributive`) overlap with `vstd::arithmetic`; prefer library lemmas and delete these duplicates.
- **Repeated finiteness/subset reasoning**: Liveness lemmas (e.g., `lemma_can_allocate_implies_bitmap_has_free_bit`, `lemma_bitmap_full_implies_slab_full`, `lemma_dealloc_from_full_enables_alloc`) re-prove finiteness/subset facts each time. Extract a shared helper and reuse it.
- **Metadata/data disjointness & inv unfolders**: Specs/proofs are duplicated (`metadata_data_disjoint`, `lemma_metadata_data_disjoint`) and empty “unfold inv” lemmas (`lemma_inv_from_components`, `lemma_view_fields`). Keep a single canonical proof or use `reveal(inv)` locally instead of multiple wrappers.
- **Verified tests co-located in main module**: Numerous proof-only tests restate invariants/preconditions. Move them to a dedicated proof/test module or prune ones that add no new obligations to slim the core file.
- **Runtime checks duplicating preconditions**: `deallocate` still performs bounds/alignment checks already guaranteed by `requires` clauses. Consider gating/removing these in verified builds to reduce code paths and proof burden.

## Status
Recent cleanup removed a few trivial arithmetic lemmas and redundant proof-only tests; core specs and invariants remain intact. The items above are still open for further simplification.
