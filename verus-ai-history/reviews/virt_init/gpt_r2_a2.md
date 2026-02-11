# Review: virt_init (gpt-5.2-codex)

## Grade: C

## Issues Found

### Critical
- None.

### High
- **Location:** `page_table_map_page` (exec: `verus/split/kernel/mm/virt/mod.rs`)  
  **Description:** Still an `external_body` with no ghost `PageTable` state refinement. The proof never establishes that concrete page tables contain the mappings recorded in `PageMapping`, so the core “mappings are actually installed” property remains unproven.  
  **Suggested Fix:** Introduce a ghost page-table model updated by `page_table_map_page` and connect it to the returned bases, or model a verified `PageTable` object with PTE state.
- **Location:** `sort_regions_by_start` / `init_full` (exec: `verus/split/kernel/mm/virt/mod.rs`)  
  **Description:** The trusted sort spec does not require that the output is a permutation of the input list. As specified, it may arbitrarily replace regions with any sorted, valid regions of the same length, so `init_full` is not proven equivalent to the original merge+sort pipeline.  
  **Suggested Fix:** Strengthen the sort spec with a permutation/element-preservation postcondition (e.g., multiset equality), or model sorting in verified code.
- **Location:** `validate_regions` / `init_checked` (exec: `verus/split/kernel/mm/virt/mod.rs`)  
  **Description:** The verifier now treats *any* overlap (end > start) as an error, but the original code only errors when page-table bases decrease. Overlapping regions within the same 4 MB page-table base would pass the original but are rejected here, so equivalence is not preserved.  
  **Suggested Fix:** Either model the original overlap detection more precisely (page-table-base ordering) or add a proof/assumption that input regions never overlap even within the same page-table base.

### Medium
- **Location:** `init` / `merge_regions` preconditions (exec: `verus/split/kernel/mm/virt/mod.rs`)  
  **Description:** The `regions[i].spec_end() <= INIT_MEMORY_SIZE` precondition still replaces the original runtime break at `MEMORY_SIZE - PAGE_SIZE`. Oversized regions would be partially mapped in the original but are rejected here.  
  **Suggested Fix:** Model the break condition or add a checked wrapper that matches the original partial-mapping behavior.
- **Location:** `get_mmio_paddr` (exec/spec: `verus/split/kernel/mm/virt/mod.rs`, `mod.spec.rs`)  
  **Description:** MMIO translation is still modeled as infallible and only constrained by alignment. The original returns `Result` and can fail on invalid MMIO addresses, so the model remains too strong.  
  **Suggested Fix:** Add a validity predicate for MMIO translation and model the error path (or prove the predicate from input invariants).
- **Location:** Module coverage (exec: `verus/split/kernel/mm/virt/mod.rs`)  
  **Description:** `VirtMemoryManager`/`manager.rs` is still excluded from the verified split. This continues to violate the coverage criterion for the original source file.  
  **Suggested Fix:** Add verified stubs/specs for `manager.rs` or explicitly scope the verification to only `virt_init` in the review criteria.
- **Location:** `init_full` postconditions (exec: `verus/split/kernel/mm/virt/mod.rs`)  
  **Description:** `init_full` now models the original preprocessing, but its `ensures` only state base alignment/ordering and drop the mapping coverage/correctness guarantees provided by `init_checked`. This is a spec weakening for the top-level API.  
  **Suggested Fix:** Propagate the full `InitResult::Ok` mapping guarantees from `init_checked` into `init_full`’s postconditions.

### Low
- **Location:** `PageMapping` / `spec_has_init_permissions` (spec: `verus/split/kernel/mm/virt/mod.spec.rs`)  
  **Description:** AccessPermission::RDWR is still not explicitly modeled; only boolean flags are recorded. The proof cannot state that the access-permission enum argument matches the original call.  
  **Suggested Fix:** Add an explicit permission field or a spec predicate that encodes the RDWR enum value.
- **Location:** `PageTableStorage::deref_len` / `deref_mut_len` (exec/spec: `verus/split/kernel/mm/virt/mod.rs`, `mod.spec.rs`)  
  **Description:** External-body specs still only assert length; aliasing/ownership and relation to backing storage are unmodeled, leaving a gap in equivalence to the unsafe deref behavior.  
  **Suggested Fix:** Add a ghost storage model and specify aliasing/ownership constraints for deref operations.

## Positive Observations
- The merge+sort pipeline (`init_full`) now reflects the original preprocessing steps and resolves the prior “unsorted input” mismatch at a high level.
- Alignment, loop-bound, and mapping-coverage lemmas remain strong and well-documented, and the ghost mapping sequence is still useful for functional reasoning.

## Summary
Several prior issues remain, and new equivalence gaps appear around the trusted sort specification and overlap detection semantics. The model is still not a full refinement of actual page-table state, and important behavioral differences (memory-bound truncation, MMIO failures) persist. Further strengthening of specs and refinement proofs is needed before the verification can be considered complete and sound.
