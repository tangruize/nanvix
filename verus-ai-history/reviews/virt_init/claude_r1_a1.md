# Review: virt_init (claude-opus-4.6)

## Grade: B-

## Issues Found

### Critical

- **Location:** mod.rs (exec) — missing `init()` function
  **Description:** The original source has a single public function `init()` (~120 lines) that is the entire purpose of this module. The verified code decomposes it into 8 helper functions (`virt_align_down`, `compute_pgtab_base`, `check_pgtab_monotonicity`, etc.) but never provides a verified `init()` function that composes them. There is no exec function modeling the double-nested loop (outer: regions, inner: pages), the page table pop/push/create state machine, or the error path for overlapping regions. The helpers are individually correct but the composition — where bugs actually live — is unverified.
  **Suggested Fix:** Implement a verified `init()` function (even if simplified) that takes a `Vec<MemRegion>`, iterates through regions and pages, maintains a `Vec` of page table bases, and proves the post-conditions defined in `VirtInitView`. This is the essential missing piece.

- **Location:** mod.spec.rs — `VirtInitView` (spec)
  **Description:** `VirtInitView` defines 5 key properties (`regions_sorted`, `regions_valid`, `page_tables_ordered`, `page_tables_aligned`, `page_tables_unique`) but is completely dead code. No exec function references it in `ensures` clauses. No proof function uses it. The properties are never connected to any executable or proven behavior. This means the specification of the init function's correctness is stated but never verified.
  **Suggested Fix:** Create a verified `init()` function whose `ensures` clause returns a `VirtInitView` satisfying all 5 properties. Alternatively, write proof lemmas that demonstrate these properties hold given the proven sub-lemmas.

### High

- **Location:** mod.rs (exec) — missing `Deref`/`DerefMut` verification
  **Description:** The original `PageTableStorage` has `Deref` and `DerefMut` impls containing `unsafe` raw pointer operations (`core::slice::from_raw_parts` / `from_raw_parts_mut`) for the `KernelPage` variant. The documentation states these are marked `external_body`, but they are not actually present in the verified code at all — neither as `external_body` stubs nor with any specification. The `PageTableStorage` enum in the verified code is a hollow shell with no methods.
  **Suggested Fix:** Add `external_body` stubs for `deref()` and `deref_mut()` with specs asserting the returned slice length equals `INIT_PAGE_SIZE / 4` (1024 entries). This captures the safety-relevant property without requiring unsafe verification.

- **Location:** mod.proof.rs / mod.rs — no loop invariant verification
  **Description:** The original `init()` has a complex inner `while` loop that maintains critical invariants: (1) `root_pagetables` stays sorted by page table address, (2) no duplicate page table entries, (3) every page in every region gets mapped. The verified code proves monotonicity as a standalone lemma but never connects it to a loop invariant. The pop_back/cmp/push_back pattern that reuses or creates page tables is the algorithmic core and is entirely unverified.
  **Suggested Fix:** Model the inner loop as an iterative function with explicit loop invariants: `root_pagetables` is sorted, all processed pages have been assigned to page tables, and the last page table base is ≤ `compute_pgtab_base(raw_vaddr)`.

- **Location:** mod.rs (exec) / mod.proof.rs — MMIO mapping not modeled
  **Description:** The original `init()` has two distinct code paths: identity mapping for non-MMIO regions (`paddr = vaddr`) and MMIO mapping via `unsafe { PhysicalAddress::from_mmio_address(...) }`. The MMIO path is completely absent from the verified code. Furthermore, the original has a subtle behavior where MMIO paddr is recomputed from `region.start()` (not current `raw_vaddr`) on each iteration (lines 206-213), meaning all pages in an MMIO region may map to the same physical frame. This behavior is neither modeled nor verified.
  **Suggested Fix:** Add an `external_body` spec for `from_mmio_address` modeling the MMIO address translation, and add a verified function modeling the MMIO paddr computation per iteration. At minimum, document this as an intentional abstraction boundary with a note about the MMIO paddr-reuse behavior.

### Medium

- **Location:** mod.proof.rs — `lemma_identity_mapping_non_mmio` (proof)
  **Description:** This proof ensures `vaddr == vaddr`, which is trivially true by reflexivity. It proves no meaningful property. The comment says it "documents the identity mapping design decision" but a comment would suffice — a vacuous proof gives false confidence that something substantive has been verified.
  **Suggested Fix:** Either remove this lemma (replace with a doc comment) or strengthen it to prove a meaningful property, e.g., that for a non-MMIO page at address `vaddr`, the frame address produced by the init algorithm equals `vaddr`. This requires modeling the `FrameAddress::new(PageAligned::from_address(PhysicalAddress::from_raw_value(raw_vaddr)))` chain.

- **Location:** mod.spec.rs vs original mod.rs — loop bound semantic mismatch
  **Description:** The original computes `end = raw_vaddr + (region.size() - 1)` and loops `while raw_vaddr < end`. This makes `end` an inclusive bound offset by 1. The spec's `spec_end` computes `start + size` (exclusive end). While `lemma_page_iteration_covers_region` uses `i < size / PAGE_SIZE` which produces the correct page count, the `-1` in the original is a potential off-by-one source that isn't explicitly reconciled with the spec. Additionally, if `size == PAGE_SIZE` (single-page region), then `end = raw_vaddr + 0 = raw_vaddr`, and `raw_vaddr < end` is false, so the loop body never executes and the page is never mapped — a potential original-code bug that the verification doesn't catch.
  **Suggested Fix:** Add a lemma or comment explicitly reconciling the original's `< (start + size - 1)` loop bound with the spec's `i < size / PAGE_SIZE` page count. Verify the single-page-region edge case.

- **Location:** Original mod.rs line 147 — overflow not verified
  **Description:** `let end: usize = raw_vaddr + (region.size() - 1);` can overflow if `raw_vaddr + region.size()` exceeds `usize::MAX`. The verified `get_nth_page_addr` has an overflow guard (`region_start + index * PAGE_SIZE <= usize::MAX`), but the end computation itself is not modeled or verified.
  **Suggested Fix:** Add a precondition or lemma verifying that `region.start + region.size - 1 <= usize::MAX` for all valid regions, consistent with `MemRegion::spec_is_valid` which already requires `spec_end() <= usize::MAX`.

- **Location:** mod.rs (exec) — page table pop/push state machine not modeled
  **Description:** The original `init()` maintains `root_pagetables` as a `LinkedList` with a specific protocol: pop_back the last entry, compare its address with the current page table address, then either reuse (Equal), create new and push both back (Greater), or error (Less). This three-way branching is the core algorithm logic and is not modeled in any verified function. The `check_pgtab_monotonicity` function proves the Less branch is unreachable for sorted inputs, but the Equal vs Greater distinction (reuse vs create) is unverified.
  **Suggested Fix:** Model the page table assignment decision as a verified function: given the last page table base and the current vaddr, determine whether to reuse or create a new page table, and prove the result is consistent with the expected output ordering.

### Low

- **Location:** mod.rs (exec) — `virt_align_down` requires `addr >= 0`
  **Description:** The precondition `addr >= 0` is always true for `usize` (unsigned). While harmless, it adds noise and may suggest the function was ported from a signed-integer context without cleanup.
  **Suggested Fix:** Remove the redundant `addr >= 0` requires clause.

- **Location:** mod.rs (exec) — `compute_pgtab_base` requires `true`
  **Description:** The precondition `requires true` is the default and can be omitted.
  **Suggested Fix:** Remove `requires true` for cleaner code.

- **Location:** mod.rs (exec) — `get_nth_page_addr` requires `index >= 0`
  **Description:** Same as above — `index` is `usize`, so `>= 0` is always true.
  **Suggested Fix:** Remove the redundant requires clause.

## Positive Observations

- **Alignment proofs are rigorous.** The 5 alignment lemmas (`lemma_align_down_le`, `_aligned`, `_monotone`, `_idempotent`, `_fixed_point`) form a complete algebraic characterization of `align_down`. They correctly use `vstd` arithmetic lemmas and are well-structured.
- **Monotonicity theorem is the right key insight.** `lemma_sorted_addrs_sorted_pgtab_bases` correctly identifies and proves the central safety property: sorted addresses produce sorted page table bases. This is the mathematical reason the overlap-detection error branch is unreachable for valid inputs.
- **Inter-region ordering is well-proven.** `lemma_non_overlapping_regions_ordered` correctly proves that non-overlapping sorted regions maintain page table ordering across region boundaries, using the page coverage and monotonicity lemmas compositionally.
- **Clean spec/proof/exec separation.** The three-file split is well-organized. Spec functions are `open` (enabling modular verification). Proof methods are grouped in `VirtProofs` for callable access from exec code. Documentation is thorough.
- **All 70 verification conditions pass** with no errors, confirming the internal consistency of all stated properties.
- **Good documentation.** The module-level doc comment clearly explains the abstraction decisions, what is and isn't modeled, and the relationship to other verified modules.

## Summary

The verification provides a solid foundation of **component-level properties** — particularly alignment arithmetic and monotonicity — but has a critical gap: **the `init()` function itself is never verified**. The decomposition into helpers is a valid strategy, but the composition step is missing entirely. The `VirtInitView` type demonstrates that the authors identified the right top-level properties to verify, but these properties are dead code — never connected to any executable or proof.

The result is that the verification proves "if you process sorted addresses, page table bases are sorted" but does NOT prove "the `init()` function processes addresses in sorted order and produces correctly ordered page tables." The gap between these two statements is exactly where the complex algorithmic logic lives (sorting, loop control, state machine, error handling).

**Priority recommendations:**
1. Implement a verified `init()` function that uses the existing helpers and proves `VirtInitView` properties — this would elevate the grade to A-/A.
2. Add `external_body` stubs for `Deref`/`DerefMut` with length specs.
3. Model and verify the page table reuse-vs-create decision (the Equal/Greater/Less branching).
4. Address the single-page-region edge case and MMIO mapping gap.
