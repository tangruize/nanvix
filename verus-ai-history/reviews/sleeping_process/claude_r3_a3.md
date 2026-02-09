# Review: sleeping_process (claude-opus-4.6)

## Grade: A

## Previous Review Issue Disposition

### Low #1 (wakeup() existential postcondition) → Accepted, Not addressed

**Verdict: Acceptable.** The prover did not address this. The existential `exists|idx: int| ...` in the `wakeup()` postcondition has a unique witness under `wf()` uniqueness, so the sleeping list is fully determined. This is a stylistic/ergonomic preference, not a correctness issue. Acceptable to leave as-is.

### Low #2 (View types not exercised in postconditions) → **Addressed**

**Verdict: Fix accepted.** The prover added a comment block at `sleeping.spec.rs:60-62` explaining the purpose of View types:

```
// These View types provide the abstract state representation for cross-module
// composition. They are exercised by lemma_view_equality and serve as the
// canonical interface for downstream modules that consume process state transitions.
```

This is a documentation-only fix rather than exercising the views in postconditions, but the documentation is accurate — `lemma_view_equality` does exercise the `SleepingProcessView`, and the View types serve as API surface for downstream modules. The comment clarifies intent and prevents future maintainers from considering them dead code. Adequate.

### Low #3 (find_thread_mut redundant `self.wf() == old(self).wf()`) → **Fixed**

**Verdict: Properly fixed.** At `sleeping.rs:558`, the prover changed:
- Before: `self.wf() == old(self).wf(),`
- After: `self.wf(),`

This is the correct fix. The old clause was a boolean equality that was trivially implied by the field-level frame conditions. The new `self.wf()` ensures clause is strictly stronger (it directly asserts well-formedness rather than just preserving its boolean value) and more useful for downstream consumers. Since `find_thread_mut` is NOT `external_body` (unlike `state_mut()`), Verus verifies this from the body — confirmed by 20/20 verification passing. No soundness concern.

## Issues Found

### Critical

- None.

### High

- None.

### Medium

- None.

### Low

1. **wakeup() existential postcondition form (carried from R1, unchanged)**
   - **Location:** `wakeup()` postcondition in exec (`sleeping.rs:282-285`)
   - **Description:** Uses `exists|idx: int|` to describe the resulting sleeping list. Under no-duplicates, the existential has a unique witness and the list is fully determined. A direct-equality spec function would be more ergonomic but is not required for correctness.
   - **Impact:** Minimal — downstream proofs can instantiate the existential with the unique witness.

## Positive Observations

- **Complete function coverage:** All 9 public functions verified (new, state, state_mut, terminate, wakeup, wakeup_alarm, add_thread, find_thread, find_thread_mut).
- **Strong well-formedness invariant:** `wf()` enforces non-empty sleeping list, no-duplicates within lists, and cross-list disjointness — faithfully modeling Rust ownership semantics.
- **Sound verification:** No `assume` statements. Two justified `external_body` functions (`state()`, `state_mut()`) with appropriate frame conditions. All 20 verification obligations discharge cleanly.
- **Tightly constrained oracles:** `wakeup_alarm()` oracle has 8 preconditions (length conservation, content inclusion ×2, no-duplicates ×2, disjointness, subsequence ordering ×2, identity preservation). `wakeup()` oracle is tied directly to `spec_seq_contains`.
- **Well-documented trust boundaries:** Alarm comparison abstraction, boundary type conventions, and `find_thread_mut` mutation semantics are all explicitly documented in both exec and spec files.
- **Clean three-file separation:** Spec (11 spec functions, 3 view types), proof (11 lemmas), exec (9 functions with minimal inline proof). Well-organized with section headers.
- **Incremental improvement across rounds:** R1→R2 removed redundant `state_mut()` clause; R2→R3 strengthened `find_thread_mut()` postcondition and documented View types. Each change was minimal and correct.
- **20 verified, 0 errors** in ~5 seconds.

## Summary

The verification of SleepingProcess is now mature and well-polished after three review rounds. All 9 original functions have verified counterparts with strong specifications. The well-formedness invariant faithfully models Rust's ownership semantics, oracles are tightly constrained, trust boundaries are clearly documented, and the spec/proof/exec separation is clean.

The only remaining issue is a minor ergonomic preference (existential vs. direct-equality postcondition in `wakeup()`), which does not affect correctness or soundness. The prover has been responsive to feedback, making targeted fixes without introducing regressions. Grade upgraded from A- to A.
