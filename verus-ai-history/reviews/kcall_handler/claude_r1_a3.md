# Review: kcall_handler (claude-opus-4.6)

## Grade: A-

## Previous Issue Resolution

| # | Previous Issue (R1A2) | Status | Verification |
|---|---|---|---|
| M1 | `spec_loop_invariant` returns `true` — vacuous | ✅ Fixed | Now uses `Seq<HarvestOutcome>` history with `forall` quantifier over elements. Inductive lemma has real proof body. |
| M2 | `spec_loop_exits_with` ignores `exit_status` param | ✅ Fixed | Replaced by `spec_loop_exits(outcome)` with no unused parameter. |
| L1 | `spec_iteration_transition` dead spec function | ✅ Fixed | Removed. Iteration tracking now via history sequence length. |
| L2 | `run_full_iteration()` narrower ensures undocumented | ✅ Fixed | Added explicit comment (exec lines 539–543) documenting the intentional narrowing. |

### Detailed Verification of M1 Fix (Loop Invariant)

This was the core remaining concern. Let me verify the fix is substantive, not cosmetic:

**`spec_loop_invariant` (spec line 349–351):**
```
forall|i: int| 0 <= i < history.len() ==> !spec_should_terminate(#[trigger] history[i])
```
This states: "no prior harvest outcome indicated INITD termination." This is the correct semantic content — the loop is still running because INITD was never harvested.

**`lemma_loop_invariant_base` (proof line 465–469):** Proves invariant for empty sequence. Correct — vacuously true for the base case, which is the standard approach.

**`lemma_loop_invariant_inductive` (proof lines 478–501):** This is the key proof. It:
1. Takes a history satisfying the invariant + a non-terminating outcome.
2. Proves the extended history also satisfies the invariant.
3. The proof body (lines 489–500) explicitly handles both cases: old elements use the existing invariant, the new element uses the continuation condition.

This is a **genuine inductive proof** with a non-trivial proof body. Verus verified it (37 conditions, 0 errors). The previous tautology has been replaced with real reasoning.

**`lemma_invariant_excludes_termination` (proof lines 540–550):** Useful lemma instantiating the universal quantifier at a specific index. Provides a clean API for deriving element-wise properties.

**Assessment:** The loop invariant framework is now substantive and correctly structured.

## Issues Found

### Critical

*(none)*

### High

*(none)*

### Medium

*(none)*

### Low

- **Location:** `ScoreBoardPollOutcome`, `DispatchOutcome`, `HandlerTermination` (spec, handler.spec.rs lines 46–53, 63–68, 112–115)
  - **Description:** These three spec-level types are defined but never referenced by any exec function, proof lemma, or other spec function. They appear to be remnants from an earlier design. `ScoreBoardPollOutcome` is superseded by the exec-level `ScoreBoardPollResult` struct. `DispatchOutcome` and `HandlerTermination` are completely unused.
  - **Suggested Fix:** Remove these dead types to reduce spec surface area, or add a comment if they are intended for future use by downstream modules.

- **Location:** Loop invariant ↔ exec disconnection (spec/proof vs exec)
  - **Description:** The loop invariant (`spec_loop_invariant` with `Seq<HarvestOutcome>` history) is purely at the spec/proof level. No exec function carries or maintains a ghost history parameter. This means the inductive reasoning is self-contained in the proof layer but is not threaded through `run_full_iteration()` ensures. A caller composing `run_full_iteration()` in an actual loop cannot use the invariant without manually tracking the ghost history.
  - **Suggested Fix:** This is acceptable for the current verification scope (the exec code is a model with `unimplemented!()` bodies). If the exec model is ever instantiated, consider adding a `Ghost<Seq<HarvestOutcome>>` parameter to `run_full_iteration()` to thread the invariant through the exec layer. No action required now.

## Positive Observations

- **Loop invariant is now genuine**: The `Seq<HarvestOutcome>` history model with `forall` quantifier over elements is the correct abstraction. The inductive proof in `lemma_loop_invariant_inductive` has a real proof body that handles both the historical and new-element cases. This is a meaningful improvement over the previous `true` invariant.
- **Clean response to all 4 prior issues**: M1 was substantively rewritten (not just patched), M2 was simplified correctly (unused parameter removed), L1 dead code was removed, and L2 was documented. No issues were dismissed without justification.
- **No new issues introduced**: The refactored spec functions (`spec_loop_exits`, `spec_loop_continues`, `spec_extend_history`) are clean and well-documented. The new proof lemmas (`lemma_history_length_after_extension`, `lemma_invariant_excludes_termination`) are useful rather than tautological.
- **Verification count stable at 37**: Same number of verified conditions, confirming no verification coverage was lost during refactoring.
- **All previous strengths retained**: Dispatch classification (21 correct mappings), work flag monotonicity, yield-iff-idle, INITD-only termination, harvest biconditional, bidirectional kcall phase ensures, full iteration composition with yield — all still present and verified.
- **No `assume` statements** in any file. All 37 verification conditions pass honestly.
- **8 well-documented external_body functions** with clear trust boundary annotations (T1–T4) and justified postcondition decisions.

## Summary

The verification has matured across three review rounds from B+ to A-. All substantive issues from the previous review have been genuinely addressed. The loop invariant — the last major concern — now uses a `Seq<HarvestOutcome>` history model with a properly structured inductive proof. The remaining issues are cosmetic: three unused spec types from earlier iterations, and the inherent gap between spec-level loop reasoning and exec-level function contracts (which is acceptable for a model-based verification).

The module now verifies the essential correctness properties of the kcall handler: dispatch routing totality and correctness (21 kcall numbers), yield-iff-idle, INITD-only termination with PID linkage, work flag monotonicity, and loop invariant preservation. Trust boundaries are clearly identified and justified. This is a solid verification of a complex kernel event loop.
