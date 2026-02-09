# Review: sleeping_process (claude-opus-4.6)

## Grade: A-

## Previous Review Issue Disposition

### Medium #1 (find_thread_mut mutation gap) → Downgraded to Low, Accepted

**Verdict: Rejection justified.** The actual kernel call sites (e.g., `process/manager/mod.rs:414-417`) use `find_thread_mut` to mutate thread-internal data like TDA (`set_thread_data_area`) and FPU state — none of which affect thread identity or list membership. The verification model tracks thread IDs in lists, and mutations through `find_thread_mut` don't change thread IDs. The frame condition (all thread ID lists preserved) is therefore semantically correct for the modeled abstraction level. The trust boundary is well-documented in `sleeping.spec.rs:47-50`.

### Medium #2 (RunnableProcess::wf() too weak) → Downgraded to Low, Accepted

**Verdict: Rejection justified.** Investigation of sibling modules confirms this is a consistent project-wide design choice: `runnable.spec.rs` defines `wf()` for its own `RunnableProcess` boundary type as `true` (no checks), and `running.spec.rs` defines it with only `ready_thread_ids.len() >= 1`. Both sibling modules provide separate helper predicates (`spec_ids_disjoint()`, `wf_strict()`) for stricter checking when needed. The sleeping module follows the same boundary type convention. Individual postconditions carry the necessary information for downstream consumers.

### Medium #3 (wakeup_alarm oracle abstraction) → Downgraded to Low, Accepted

**Verdict: Rejection justified.** The alarm comparison is trivially correct arithmetic (`now >= alarm`). The complex and error-prone part — the partition/conservation logic, stable ordering, and thread ID preservation — is fully verified. The oracle preconditions are tightly constrained (length conservation + content inclusion + no duplicates + disjointness + subsequence ordering). The trust boundary is documented in the exec file (lines 47-49).

### Low #1 (InterruptedProcess::wf() weak) → Accepted

**Verdict: Rejection justified.** No dedicated InterruptedProcess verification module exists in the project. This is a boundary type consistently modeled with minimal `wf()` across all sibling modules. The postconditions of `terminate()` and `wakeup_alarm()` carry the necessary content-level information.

### Low #2 (wakeup() existential postcondition) → Not addressed, Still valid but Low

**Verdict: Still open.** The existential form `exists|idx: int| ... && rp.sleeping_thread_ids@ == spec_remove_at(...)` remains. Under `wf()` uniqueness this has a unique witness, so the sleeping list is fully determined. A direct-equality postcondition using a `spec_remove_element(s, val)` function would be more ergonomic for downstream proofs, but this is a stylistic preference, not a correctness issue.

### Low #3 (Unused View types) → Not addressed, Still valid but Low

**Verdict: Still open.** `SleepingProcessView`, `RunnableProcessView`, and `InterruptedProcessView` are defined with `View` trait implementations but never used in any postcondition or lemma. The `lemma_view_equality` proof exercises the view for `SleepingProcess`, but no postcondition is expressed in terms of view mappings. These could become stale; however, they provide infrastructure for future cross-module composition.

### Low #4 (state_mut redundant wf clause) → **Fixed**

**Verdict: Properly fixed.** The redundant `self.wf() == old(self).wf()` postcondition was removed from the `external_body` function `state_mut()`. This reduces the trust surface of the external_body without losing any verification strength, since `wf()` preservation follows automatically from the individual field-level frame conditions. The fix is minimal and correct.

## Issues Found

### Critical

- None.

### High

- None.

### Medium

- None.

### Low

1. **wakeup() existential postcondition form (carried from R1)**
   - **Location:** `wakeup()` postcondition in exec (`sleeping.rs:282-285`)
   - **Description:** Uses `exists|idx: int|` to describe the resulting sleeping list. Under no-duplicates, the existential has a unique witness so the list is fully determined, but a direct-equality spec function would be more ergonomic for downstream consumers.
   - **Suggested Fix:** Add `spec_remove_element(s: Seq<int>, val: int) -> Seq<int>` that internally locates the unique index and returns the result. Replace the existential postcondition with direct equality.

2. **View types defined but not exercised in postconditions (carried from R1)**
   - **Location:** `SleepingProcessView`, `RunnableProcessView`, `InterruptedProcessView` in spec (`sleeping.spec.rs:63-98`)
   - **Description:** View type infrastructure exists but is never used in any function postcondition. Only `lemma_view_equality` exercises the view for `SleepingProcess`. This could become stale.
   - **Suggested Fix:** Either use view types in at least one transition postcondition (e.g., express `terminate()` output as an `InterruptedProcessView` mapping), or add a comment documenting they exist for cross-module composition.

3. **find_thread_mut retains redundant `self.wf() == old(self).wf()` (new)**
   - **Location:** `find_thread_mut()` in exec (`sleeping.rs:558`)
   - **Description:** While `state_mut()` had this redundant clause removed (fixing Low #4 from R1), `find_thread_mut()` at line 558 still contains `self.wf() == old(self).wf()`. Unlike `state_mut()`, `find_thread_mut` is NOT `external_body` — Verus verifies this from the body, so the redundancy is harmless (machine-checked rather than trusted). However, for consistency with the `state_mut()` fix, it could be replaced with an explicit `self.wf()` ensures clause (which is more useful than a boolean equality).
   - **Suggested Fix:** Replace `self.wf() == old(self).wf()` with just `self.wf()` for consistency and clarity. Since the body doesn't modify self, Verus can verify `self.wf()` directly.

## Positive Observations

- **Complete function coverage:** All 9 public functions from the original source have verified counterparts.
- **Strong well-formedness invariant:** `wf()` faithfully models Rust ownership semantics via uniqueness and disjointness predicates, with the exec-level `sleeping_count` tied to the ghost sequence length.
- **Sound verification:** No `assume` statements anywhere. Two `external_body` functions (`state()`, `state_mut()`) are justified trust boundaries for `Box<ProcessState>` access, with appropriate frame conditions.
- **Tightly constrained oracle design:** The `wakeup_alarm()` oracle is constrained by 8 preconditions (length conservation, content inclusion ×2, no-duplicates ×2, disjointness, subsequence ×2, identity preservation on !has_expired). This leaves very little room for an incorrect oracle to satisfy all constraints simultaneously.
- **Thorough proof library:** 11 lemmas covering construction, remove-at operations, find-thread consistency, view equality, and boundary type well-formedness. The proof-by-contradiction structure in `lemma_remove_at_removes_element` is particularly well-crafted.
- **Clean spec/proof/exec separation:** Proper three-file split with clear responsibilities. Section headers follow project coding standards.
- **Stable partition ordering:** The `spec_is_subsequence` predicate captures the important front-to-back processing order of the original implementation.
- **Consistent boundary type convention:** The minimal `wf()` for boundary types aligns with the project-wide pattern confirmed in sibling modules.
- **20 verified, 0 errors** in ~6 seconds.

## Summary

The prover addressed the one clear-cut fix from R1 (removing the redundant `self.wf() == old(self).wf()` from the `external_body` `state_mut()`). The remaining issues from R1 were either correctly rejected as documented trust boundaries or accepted as low-priority stylistic suggestions.

Investigation of the actual kernel code confirms that `find_thread_mut` mutations only affect thread-internal data (TDA, FPU state), not thread identity or list membership, validating the verification model's frame condition. The boundary type `wf()` weakness is a consistent project-wide design choice, not a sleeping-module-specific gap.

The verification is complete, sound, and well-documented. The three remaining Low issues are ergonomic improvements (existential postcondition form, unused view types, minor redundancy in `find_thread_mut`) that do not affect correctness or soundness. Grade remains A- due to these minor polish items.
