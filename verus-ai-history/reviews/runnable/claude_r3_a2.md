# Review: runnable (claude-opus-4.6) — Round 2

## Grade: A

## Verification Result

All 43 items verified, 0 errors. Clean pass via `./verus-ai/scripts/verify.sh runnable`.

## Previous Issues — Disposition

### High: `state_mut()` cross-module obligation

**Status: Fixed (documentation)**

The prover updated both the exec file (lines 52–58) and spec file (lines 58–62) to explicitly document the cross-module verification obligation. The new text is stronger and more precise: it names `spec_pid()` immutability as the specific invariant, states callers "must prove" preservation, and clarifies when the obligation is discharged. Verified by inspecting the diff — the old text ("should be addressed when verifying callers") was vague; the new text is a clear obligation statement. This is the correct resolution given that Verus cannot express `&mut` returns; the risk is now explicitly tracked.

### Medium: `wakeup()` oracle parameter

**Status: Acknowledged, no change needed**

The `found` oracle remains. This is inherent to the verification model (ghost-only sleeping list). The precondition `found == spec_seq_contains(...)` correctly constrains it. No new issues introduced. The documentation at exec lines 477–489 and spec lines 80–87 remains thorough. Acceptable.

### Medium: `EXIT_STATUS_INTERRUPTED()` hardcoded constant

**Status: Not fixed**

The TODO at spec lines 166–168 remains unchanged. The constant `4` is still hardcoded with no CI cross-check. This is a maintainability risk, not a soundness issue — if `ErrorCode::Interrupted` changes its numeric value, the spec silently becomes wrong. However, this is a low-severity concern: `ErrorCode` values in OS kernels are ABI-stable and rarely change.

**Verdict:** Acceptable to defer. The TODO is clear and tracked.

### Medium: `find_thread()` / `find_thread_mut()` omitted from exec

**Status: Acknowledged, no change needed**

These remain spec-only models. The `spec_find_thread` spec (spec lines 269–281) correctly models the search priority and exhaustive coverage. Proof lemmas (`lemma_find_thread_ready`, `lemma_find_thread_not_found`, `lemma_find_thread_iff_has_thread`) verify the spec's properties. No exec-level alternative is feasible without Verus reference support.

### Low: `terminate()` duplicate postconditions

**Status: Fixed**

The duplicate conditions (`self.spec_interrupted_count() == 0 && self.spec_sleeping_count() == 0`) in the `TerminateResult::Zombie` branch have been removed. The remaining conditions at lines 418–419 already assert `self.spec_interrupted_count() == 0` and `self.spec_sleeping_count() == 0`. Verified via diff — three lines removed cleanly. No regression (verification still passes with 43 items).

### Low: `run()` arbitrary `interrupt_reason` value

**Status: Fixed (documentation)**

The comment was updated from "Unconstrained; downstream may refine." to "Arbitrary witness value; postcondition does not constrain interrupt_reason." (exec line 371). This is clearer — it explicitly states the value is arbitrary and the postcondition does not mention it.

### Low: Thread ID disjointness not in `wf()`

**Status: Acknowledged, no change needed**

This remains a deliberate design choice. The `spec_ids_disjoint()` predicate (spec lines 371–384) is provided for downstream proofs. The documentation is thorough.

## New Issues Introduced

None. The changes are minimal and surgical — two documentation improvements and one postcondition cleanup. No structural changes to specs, proofs, or exec code. The verification count remains at 43.

## Remaining Issues

### Medium

- **Location:** `EXIT_STATUS_INTERRUPTED()` — spec file, line 169
- **Description:** Hardcoded constant `4` matching `ErrorCode::Interrupted`. Still has a TODO for CI cross-check but no implementation. Low-severity maintainability risk.
- **Suggested Fix:** Add a cross-module assertion or CI test. Acceptable to defer as tracked TODO.

### Low

- **Location:** `wakeup()` — exec file, line 498; `found` oracle parameter
- **Description:** Oracle parameter shifts verification burden to callers. Well-constrained by precondition and thoroughly documented. Inherent to the ghost-model approach.
- **Suggested Fix:** No action needed at this module level. Verify callers provide correct `found` values when those modules are verified.

## Positive Observations

All positive observations from Round 1 remain valid:

- **Comprehensive content-level postconditions:** All state transitions specify exact sequence contents, not just counts.
- **Minimal trust boundary:** Single `external_body` (`clock_now()`), no `assume` or `trusted` annotations.
- **Oracle elimination:** Successfully eliminated oracles from `terminate()` and `run()`.
- **Strong inductive proofs:** `lemma_min_index_rec_bounds` and `lemma_seq_has_min` are clean and correct.
- **Well-documented trust assumptions:** Both spec and exec files have thorough documentation.
- **Clean three-file split:** 693 exec / 512 spec / 750 proof lines, well-separated concerns.
- **Semantic equivalence verified:** `terminate()` list ordering, `run()` tie-breaking, and `wakeup()` thread movement all match the original's behavior.

Additional observation from this round:

- **Responsive to review:** The prover addressed documentation issues precisely — upgrading vague language to explicit obligation statements. The postcondition cleanup was correct and clean. Changes were minimal and introduced no regressions.

## Summary

The prover addressed all fixable issues from Round 1. The `state_mut()` cross-module obligation is now explicitly documented as a verification requirement in both files. The `terminate()` postcondition redundancy is removed. The `interrupt_reason` comment is clarified. The only remaining substantive item is the `EXIT_STATUS_INTERRUPTED` hardcoded constant, which has a tracked TODO and poses minimal practical risk. The verification is sound, well-structured, and production-quality.
