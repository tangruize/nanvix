# Review: interrupted_process (claude-opus-4.6)

## Grade: A

## Previous Issue Disposition

### Medium #1 (from r2_a2): Bridging lemma `lemma_interrupted_view_subsumes_runnable_boundary` was trivial
- **Status: Genuinely fixed.**
- The previous lemma was replaced with `lemma_project_to_runnable_boundary` (proof lines 326–365). This is a materially different and superior lemma:
  - Returns a concrete tuple `(int, Seq<int>, Seq<int>)` matching the runnable module's boundary `InterruptedProcess` fields (`pid`, `interrupted_thread_ids`, `zombie_thread_ids`).
  - Postconditions prove non-trivial invariants on the projected fields: `projection.1.len() >= 1` (non-empty), `spec_no_duplicates(projection.1)`, `spec_no_duplicates(projection.2)`, `spec_seqs_disjoint(projection.1, projection.2)`.
  - Additionally proves that the sleeping threads (not in the projection) are disjoint from both interrupted and zombie in the projection — useful for integration proofs that need to reason about the dropped field.
  - Clearly documents the limitation: "This lemma cannot reference the runnable module's actual `InterruptedProcess` struct (Verus modules are verified independently)."
- **Verification:** I confirmed the runnable module's `InterruptedProcess::wf()` (runnable.spec.rs:429–431) only requires `interrupted_thread_ids@.len() >= 1`. The projection lemma's postconditions are a *superset* of that `wf()` requirement — they also prove no-duplicates and disjointness, which `wf()` in runnable does not require. This is actually stronger than needed, which is good.
- **Verdict: Resolved.** The projection lemma provides genuine value for cross-module integration proofs. It gives integration code a concrete tuple with proven invariants that can be used to construct the runnable module's boundary type.

### Medium #2 (from r2_a2): `find_thread()` spec-only trust gap
- **Status: Accepted as Verus limitation.** No code changes expected or needed.
- Documentation continues to properly tag this for trust-boundary inventory (exec line 76, spec lines 58–62).
- `lemma_find_thread_refinement_assumption` (proof lines 227–246) remains a documentation lemma — it only restates the spec definition. This is understood and acceptable.
- **Verdict: Stable. Properly documented Verus limitation.**

### Low #1 (from r2_a2): Clock oracle remains advisory
- **Status: Unchanged (acceptable).**
- Documentation in exec (lines 54–62) and spec (lines 73–77) explicitly states the link is advisory at this module level.
- **Verdict: Stable. Correct modular design.**

### Low #2 (from r2_a2): Duplicated `spec_no_duplicates` / `spec_seqs_disjoint`
- **Status: Not addressed.**
- **Verdict: Reasonable deferral.** Cosmetic issue, no correctness impact. The definitions are identical (verified by inspection: spec lines 200–210 for `InterruptedProcess`, spec lines 307–317 for `RunnableProcess`).

## Issues Found

### Critical

_None._

### High

_None._

### Medium

- **Location:** `find_thread()` / `find_thread_mut()` — spec-only model (inherited, stable)
  - **Description:** These functions bypass the executable `iter().find()` logic. The search predicate and collection ordering in the original code are trusted, not verified. This is a Verus limitation, not a prover oversight.
  - **Suggested Fix:** None at this time. Properly documented and tagged for trust-boundary inventory.

### Low

- **Location:** `spec_no_duplicates` / `spec_seqs_disjoint` — duplicated across types
  - **Description:** Identical helper specs on both `InterruptedProcess` and `RunnableProcess`. Minor divergence risk.
  - **Suggested Fix:** Extract to shared module. Low priority.

- **Location:** `lemma_project_to_runnable_boundary` postcondition — disjointness argument order
  - **Description:** The postcondition proves `spec_seqs_disjoint(ip@.sleeping_thread_ids, projection.1)` and `spec_seqs_disjoint(ip@.sleeping_thread_ids, projection.2)` (proof lines 359–362). Note the argument order: sleeping is first. The `wf()` predicate in this module uses the opposite order for sleeping-vs-interrupted disjointness: `spec_seqs_disjoint(interrupted_ids, sleeping_ids)` (spec line 223). While `spec_seqs_disjoint` is semantically symmetric (`forall |i,j| ... ==> a[i] != b[j]` is equivalent when swapped), the reversed argument order compared to `wf()` could cause minor friction in downstream proofs that expect a specific argument order. Not a soundness issue.
  - **Suggested Fix:** Consider matching the argument order to `wf()` for consistency. Very low priority.

## Positive Observations

- **Verification passes cleanly:** 23 verified, 0 errors. No `assume`, `external_body`, or `trusted` annotations in any executable or proof code (only mentions are in documentation comments).
- **Projection lemma is genuinely useful:** `lemma_project_to_runnable_boundary` returns a concrete tuple with non-trivial invariants (non-empty, no-duplicates, pairwise disjoint) that are *stronger* than the runnable module's boundary `wf()` requires. It also preserves information about the dropped sleeping thread field. This is a well-designed cross-module bridge.
- **Trust boundary documentation is exemplary:** Every trust gap (interrupt_reason mutation, clock oracle, find_thread spec-only model, ProcessState abstraction, boundary model inconsistency) is documented in both the exec module header and the spec trust assumptions section with clear scope, rationale, and remediation paths.
- **All original functions covered:** `new`, `from_sleeping`, `state`, `state_mut`, `resume`, `find_thread`, `find_thread_mut`, standalone `interrupt` — all 8 items have verified models.
- **`resume()` proof is rigorous:** All six pairwise disjointness conditions, no-duplicates preservation, front-not-in-tail, singleton well-formedness, admission time validity. PID immutability explicit.
- **`wf()` correctly models ownership:** Intra-list uniqueness and inter-list disjointness faithfully capture Rust's `NonEmptyVecDeque` ownership guarantees.
- **Clean spec/proof/exec separation:** No bleeding between concerns. Spec has only `open spec fn` and view types. Proof has only lemmas. Exec has inline proof only in `resume()`.
- **No regressions from fixes:** The projection lemma replacement did not weaken any guarantees. Verification count remained at 23 (old trivial lemma replaced by new substantive one).

## Summary

The prover effectively addressed the only substantive remaining issue from the previous review (the trivial bridging lemma). The replacement `lemma_project_to_runnable_boundary` provides genuine cross-module value: a concrete tuple projection with non-trivial invariants that match and exceed the runnable module's boundary requirements. The remaining issues are a Verus-inherent limitation (`find_thread` spec-only model) and minor cosmetic items (duplicated helpers, argument order consistency), none of which affect soundness or completeness.

The verification is mature: all functions are covered, the well-formedness invariant is strong, proofs are rigorous, there are no trusted primitives, and trust boundaries are thoroughly documented. This module is ready for integration with sibling process state modules.
