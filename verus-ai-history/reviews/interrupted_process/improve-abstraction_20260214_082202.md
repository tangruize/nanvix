# Review: interrupted_process Abstraction (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical
- None.

### High
- **Location:** `interrupted.rs` (exec file), full diff from 532faf1a8..ebcf601a2.
  **Description (NO EXEC CHANGES criterion):** The exec file was significantly modified as part of this change. The struct fields changed from `Ghost<int>` / `Ghost<Seq<int>>` to concrete `u64` / `Vec<u64>`, and all functions were rewritten from ghost-only to real exec functions. While these changes were likely necessary to support proper `View` implementations (you need a concrete type to have `impl View`), the review criterion requires the `.rs` exec file to be unmodified from the previous verified version. The exec changes represent a fundamental restructuring from ghost-model to exec-model verification, which goes beyond "adding abstract state transition spec functions to View types."
  **Suggested Fix:** If the Ghost→concrete migration was intentional and prerequisite, document this explicitly in the commit message and spec header as a separate concern from the view-level abstraction additions. Ideally, the migration and the abstraction addition would be separate commits for reviewability.

### Medium
- **Location:** `interrupted.spec.rs`, `RunnableProcessView` (line 119–133).
  **Description (COMPLETENESS gap):** `RunnableProcessView` has no `wf()` spec function, unlike `InterruptedProcessView` which defines `wf()` at the view level (line 410). This means downstream modules cannot express `pre@.spec_resume(t).wf()` at the view level. The concrete `RunnableProcess::wf()` exists, and `resume()` ensures `result.wf()`, but there is no bridge to prove the view of the result is well-formed. `lemma_view_wf` (proof line 331) only covers `InterruptedProcess` → `InterruptedProcessView`; there is no analogous lemma for `RunnableProcess` → `RunnableProcessView`.
  **Suggested Fix:** Add `RunnableProcessView::wf()` mirroring `RunnableProcess::wf()` with `Seq` fields, and a bridging lemma `lemma_runnable_view_wf(p: &RunnableProcess) requires p.wf() ensures p@.wf()`. Then add an ensures clause to `lemma_resume_refines_spec`: `result@.wf()` (derivable from `result.wf()` + the new bridge).

- **Location:** `interrupted.proof.rs`, `lemma_new_is_wf` (line 39–55) and `lemma_from_sleeping_is_wf` (line 74–97).
  **Description (CORRECTNESS weakening):** These lemmas now have tautological ensures (the postconditions repeat the preconditions verbatim). Before the migration, they constructed an `InterruptedProcess` value and proved `wf()` on it. The comment says "Cannot construct an InterruptedProcess with `Vec<u64>` in proof mode" — this is a valid Verus limitation, but the lemmas are now trivially true and prove nothing new beyond what the caller already knows. They provide no additional assurance that construction produces well-formed state at the proof level.
  **Suggested Fix:** Either (a) reformulate the ensures to state that `InterruptedProcessView::spec_new(pid, interrupted_ids, zombie_ids).wf()` holds (operating on the View type which CAN be constructed in proof mode), or (b) remove these lemmas and rely solely on the exec-level `ensures result.wf()` postconditions. Option (a) is preferred as it keeps the proof-level regression guard meaningful.

### Low
- **Location:** `interrupted.spec.rs`, `spec_state_mut()` (line 474) and `spec_find_thread_mut()` (line 481).
  **Description:** Both functions are identical identity transitions (`self`). While semantically correct (both operations preserve all state), having two functions with the same body could confuse consumers about whether they differ. No semantic issue.
  **Suggested Fix:** Add a brief inline comment on each noting they are intentionally identical because neither operation mutates InterruptedProcess state in the model. Alternatively, define a single `spec_identity()` and have both delegate to it.

- **Location:** `interrupted.spec.rs`, `RunnableProcess::spec_no_duplicates` (line 388) and `RunnableProcess::spec_seqs_disjoint` (line 394).
  **Description:** These are exact duplicates of the same-named functions on `InterruptedProcess` (lines 218, 224). Since they are `open spec fn`, they are semantically equivalent but the duplication adds maintenance burden. No functional issue.
  **Suggested Fix:** Consider extracting to a shared spec helper module or using a single canonical definition. Not urgent since Verus module isolation may require this pattern.

- **Location:** `interrupted.proof.rs`, `lemma_new_wf` on `RunnableProcess` (line 557).
  **Description:** Same tautological ensures pattern as the `InterruptedProcess` construction lemmas. The old version constructed a `RunnableProcess` and proved `wf()`; the new version just restates preconditions. Additionally, the old version required `ready_times[i] >= 0` which was dropped — correct since `u64` is always non-negative, but the semantic change is undocumented.
  **Suggested Fix:** Same as the Medium issue above: reformulate using `RunnableProcessView` if a `wf()` is added there, or document the `u64` non-negativity rationale.

## Positive Observations

- **Complete coverage:** Every state-changing exec function (`new`, `from_sleeping`, `resume`, `state_mut`, `find_thread_mut`) has a corresponding view-level spec transition function on `InterruptedProcessView`. Read-only functions (`state`, `find_thread`) are correctly excluded.
- **Accurate modeling:** `spec_resume()` correctly models the front-pop, singleton ready list creation, admission time recording, and preservation of sleeping/zombie threads. The `subrange(1, len)` abstraction faithfully represents `Vec::remove(0)`.
- **Proper abstraction:** View types use `Seq<u64>` (abstract sequences) rather than `Vec<u64>` (concrete). Well-formedness predicates use quantifier-based formulations (`spec_no_duplicates`, `spec_seqs_disjoint`) rather than implementation-level operations.
- **Bridging lemmas:** Each view-level spec function has a corresponding bridging lemma in the proof file (`lemma_new_refines_spec`, `lemma_from_sleeping_refines_spec`, `lemma_resume_refines_spec`, `lemma_state_mut_refines_spec`, `lemma_find_thread_mut_refines_spec`) that connects exec postconditions to view-level transitions using `=~=` extensional equality.
- **`lemma_view_wf`** correctly bridges concrete `wf()` to view-level `wf()` for `InterruptedProcess`.
- **Verification passes cleanly:** 36 verified, 0 errors.
- **Trust gaps thoroughly documented:** The module header and spec comments document all abstraction gaps (per-thread state mutation, find_thread iterator semantics, ProcessState PID linking, admission time oracle) with formal integration obligations.
- **Well-structured spec transitions enable downstream composition:** Callers can now write `ensures result@ =~= old(self)@.spec_resume(admission_time)` instead of listing every field change, significantly improving proof ergonomics.

## Summary

The abstraction improvement is well-executed and achieves its primary goal: every state-changing operation now has a matching view-level spec transition function with a proven bridging lemma. The spec functions accurately model exec behavior, use proper abstract types, and verification passes cleanly. The main concern is that the exec file was modified (Ghost→concrete migration), which exceeds the scope of "adding view-level spec functions" and should ideally be a separate commit. The secondary gap is the missing `RunnableProcessView::wf()` which prevents complete view-level well-formedness reasoning for `resume()` results. The weakened construction lemmas (tautological ensures) should be reformulated to operate on View types for meaningful proof-level assurance. Overall, this is solid work that significantly improves the module's downstream usability.
