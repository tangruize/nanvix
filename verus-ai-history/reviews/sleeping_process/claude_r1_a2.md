# Review: sleeping_process (claude-opus-4.6)

## Grade: A-

## Verification Result

21 verified, 0 errors (down from 24 — 3 trivial lemmas correctly removed).

## Previous Issue Resolutions

### High #1: `wakeup_alarm` oracle lacks content conservation — ✅ FIXED

The prover added five new preconditions to `wakeup_alarm()` (sleeping.rs:391–398):
```
forall|i: int| 0 <= i < interrupted_ids@.len() ==>
    Self::spec_seq_contains(self.sleeping_thread_ids@, interrupted_ids@[i]),
forall|i: int| 0 <= i < remaining_ids@.len() ==>
    Self::spec_seq_contains(self.sleeping_thread_ids@, remaining_ids@[i]),
Self::spec_no_duplicates(interrupted_ids@),
Self::spec_no_duplicates(remaining_ids@),
Self::spec_seqs_disjoint(interrupted_ids@, remaining_ids@),
```
**Verified correct.** Combined with `spec_no_duplicates(sleeping_thread_ids@)` from `wf()` and the existing length conservation, these constraints ensure the oracle partitions are a true permutation of the original sleeping list. By pigeonhole: every original element must appear in exactly one partition, no fabricated IDs are possible, and no duplication can occur. This is equivalent in strength to multiset equality, achieved without Verus multiset support. Well done.

### Medium #1: `wf()` lacks thread ID uniqueness — ✅ FIXED

`wf()` now includes (sleeping.spec.rs:192–194):
```
Self::spec_no_duplicates(self.sleeping_thread_ids@)
Self::spec_no_duplicates(self.zombie_thread_ids@)
Self::spec_seqs_disjoint(self.sleeping_thread_ids@, self.zombie_thread_ids@)
```
New spec helpers `spec_no_duplicates` and `spec_seqs_disjoint` are correctly defined (sleeping.spec.rs:170–180). The `new()` constructor correctly requires these properties as preconditions (sleeping.rs:150–152). The spec comment about ownership semantics (sleeping.spec.rs:30–36) was updated from "NOT enforced" to "IS enforced" with an explanation.

### Medium #2: `wakeup()` existential is ambiguous without uniqueness — ✅ ADDRESSED

The prover's argument is correct: with `spec_no_duplicates` now in `wf()`, the `exists|idx|` in the `wakeup()` postcondition is uniquely determined — there is at most one index where `sleeping_thread_ids@[idx] == tid@`. Comment added at sleeping.rs:280 confirms this reasoning. No postcondition change needed.

### Low #1: `lemma_wakeup_alarm_conservation` tautology — ✅ FIXED (removed)

### Low #2: `lemma_pid_preserved` trivial — ✅ FIXED (removed)

### Low #3: `lemma_terminate_result_has_interrupted` trivial — ✅ FIXED (removed)

### Low #4: `find_thread`/`find_thread_mut` documentation — ✅ FIXED

Both functions now have expanded doc comments (sleeping.rs:497–502, 518–521) documenting the Verus limitation and caller obligations. `find_thread_mut` specifically notes that callers must preserve thread identity and list membership after mutation.

## New Issues Introduced

None. The changes are clean, correct, and introduce no regressions.

## Remaining Issues

### Low

- **Location:** `lemma_wakeup_alarm_expired_is_wf()` (proof: sleeping.proof.rs:136–147)
  **Description:** Proves `interrupted_ids.len() >= 1` from `interrupted_ids.len() > 0`. This is a trivially true implication that adds no proof value.
  **Suggested Fix:** Remove or merge into a more substantive lemma.

- **Location:** `lemma_terminate_preserves_total_threads()` (proof: sleeping.proof.rs:71–81)
  **Description:** Proves `sleeping_count + zombie_count == sleeping_count + zombie_count`, which is true by the definition of `spec_total_thread_count()`. This is a tautology.
  **Suggested Fix:** Remove or strengthen to prove a cross-operation property (e.g., that terminate's result preserves total thread count when accounting for the sleeping→interrupted transition).

- **Location:** `lemma_wf_and_found_implies_sleeping_positive()` (proof: sleeping.proof.rs:99–106)
  **Description:** Proves `sleeping_count > 0` given `wf()` and `spec_seq_contains`. Since `wf()` already guarantees `sleeping_thread_ids@.len() >= 1` and `sleeping_count == sleeping_thread_ids@.len()`, this is trivially derivable. The lemma is used in `wakeup()`, but the proof assistant should handle this automatically.
  **Suggested Fix:** Try removing and check if `wakeup()` still verifies without it. If needed, keep it.

## Positive Observations

- **All 7 previous issues resolved:** Every issue from the first review was addressed — the High and Medium issues substantively, the Low issues through removal or documentation updates.
- **Strong content conservation on wakeup_alarm oracle:** The combination of subset membership, no-duplicates, disjointness, and length conservation effectively achieves multiset partition equality without requiring Verus multiset support. This is an elegant solution.
- **wf() now captures Rust ownership semantics:** The addition of uniqueness and disjointness predicates to `wf()` bridges the gap between Rust's linear type guarantees and the ghost-level Seq model. This strengthens every operation's postcondition.
- **Proof file cleaned up:** Removed 3 tautological lemmas (24→21 verified items), improving signal-to-noise in the proof file.
- **Documentation consistency:** Module headers, spec comments, and per-function doc comments were all updated to reflect the new invariants. No stale documentation remains.
- **Complete function coverage maintained:** All 9 original functions still have verified counterparts with strong contracts.
- **Verification passes cleanly:** 21/21 with no errors, no assumes, and only 2 justified `external_body` annotations (`state()`, `state_mut()`) with proper frame conditions.

## Summary

All issues from the first review have been addressed. The `wakeup_alarm` oracle now has tight content conservation constraints that, combined with the strengthened `wf()` uniqueness invariant, effectively prove that the alarm-based partition is a true permutation of the original sleeping list. The `wf()` predicate now properly models Rust's ownership semantics via `spec_no_duplicates` and `spec_seqs_disjoint`. Three trivial proof lemmas were removed.

The remaining issues are limited to a few trivially-provable lemmas in the proof file that add clutter but no unsoundness. No new issues were introduced. The verification is sound, complete in coverage, and well-documented.
