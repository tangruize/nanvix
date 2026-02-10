# Review: kcall_handler (claude-opus-4.6) — Round 2

## Grade: A

## Previous Issue Disposition

### High #1: exit_status unconstrained in LoopResult — PARTIALLY FIXED
The prover added `termination_pid` tracking through `LifecycleStepResult` and `LoopResult`, with the ensures clause `result.terminated ==> result.termination_pid == 1u32` propagated through the loop invariant. This proves that termination was triggered by INITD. However, `exit_status` itself remains unconstrained — the model still cannot prove that the returned status equals what INITD produced. The prover's position (documented in `spec_handler_terminated_correctly` at spec line 462-464: "The `exit_status` value is intentionally unconstrained...") is defensible since the exit status originates from `harvest_zombies()` (T2 boundary) and its correctness genuinely depends on ProcessManager state. **Verdict: Partially addressed. The `termination_pid` fix is meaningful; the exit_status gap is now an explicit, documented design decision rather than an oversight. Downgraded from High to Low.**

### High #2: Fuel semantic gap + top-level correctness — FIXED
The prover added:
- `spec_handler_terminated_correctly()` (spec lines 471-479): Top-level correctness predicate checking terminated, pid==INITD, and loop invariant.
- `lemma_handler_top_level_correctness()` (proof lines 756-772): Connects `kcall_handler_loop` postconditions to the top-level spec.
The preconditions of the lemma exactly match the postconditions of `kcall_handler_loop`, making it directly usable by callers. The fuel semantic gap documentation in the exec file (lines 922-928) is already prominent. **Verdict: Fixed. The top-level theorem is sound and usable.**

### Medium #1: Scoreboard error paths elided — FIXED
`ScoreBoardPollResult` now includes `has_error: bool` (line 401) with documentation explaining the `unreachable!()` paths (lines 390-394). The `poll_scoreboard_full()` postcondition includes `result.has_error ==> !result.has_call` (line 770), ensuring that scoreboard errors produce no-work outcomes. Since `handle_kcall_phase` gates on `has_call`, the error → no-work path is proven sound without explicitly checking `has_error`. **Verdict: Fixed.**

### Medium #2: signal_handled documentation — FIXED
`signal_handled()` now has `ensures true` (line 242) with explicit documentation (lines 230-235) explaining that the original's `Err(e)` path logs a warning but does not affect `kcall_handled`. **Verdict: Fixed.**

### Medium #3: exit_status unconstrained in harvest_zombies — ACKNOWLEDGED
Still unconstrained, but the prover's reasoning is sound: the exit status originates from PM state (T2) and cannot be constrained without modeling process lifecycle. Now documented in multiple places (harvest_zombies doc at line 307-311, LifecycleStepResult doc at lines 789-790, LoopResult doc at lines 893-894, spec_handler_terminated_correctly doc at lines 462-464). **Verdict: Accepted as documented design decision. Downgraded to Low.**

### Medium #4: Hardcoded kcall constants — FIXED
`spec_classify_handler_kcall` now has a detailed doc comment (spec lines 186-193) listing all NR_* constants from `src/libs/sys/src/sys/number.rs` with their values. **Verdict: Fixed.**

### Low #1: drain_remaining_zombies — UNCHANGED
Still `ensures true`. Acceptable for T2 boundary. **Verdict: Accepted.**

### Low #2: event_init documentation — UNCHANGED
Adequate as-is. **Verdict: Accepted.**

### Low #3: poll_scoreboard_full default value — IMPROVED
The postcondition comment now explicitly states "Default is 0 (Debug); the `has_call` gate in handle_kcall_phase prevents this from being used" (lines 764-766). **Verdict: Addressed via documentation.**

### Low #4: SPEC_ERROR_INVALID_SYSCALL unused — UNCHANGED
Still used only in `make_invalid_syscall_error()` ensures. Acceptable. **Verdict: Accepted.**

## Issues Found

### Critical

None.

### High

None.

### Medium

None.

### Low

- **Location:** `kcall_handler_loop()` (exec), `LoopResult.exit_status`
  **Description:** The `exit_status` field remains unconstrained by postconditions. When `terminated == true`, the model proves `termination_pid == 1` (INITD triggered exit) but cannot prove the exit status equals what `harvest_zombies()` returned for INITD. This is now an explicit design decision documented in `spec_handler_terminated_correctly` and multiple struct doc comments. The limitation is inherent to the T2 trust boundary — constraining `exit_status` would require modeling ProcessManager lifecycle, which is out of scope.
  **Suggested Fix:** No action required. If PM verification is added in the future, `exit_status` can be ghost-tracked through a PM postcondition that links harvest results to process state.

- **Location:** `spec_handler_terminated_correctly()` (spec), `lemma_handler_top_level_correctness()` (proof)
  **Description:** The top-level correctness theorem does not directly state the contrapositive liveness property as a single ensures clause. The contrapositive ("if INITD terminates within fuel, then terminated == true") requires combining `lemma_handler_top_level_correctness` with `lemma_loop_termination_completeness` — a two-step reasoning chain that callers must assemble. A single combined lemma would be more ergonomic.
  **Suggested Fix:** Consider adding a convenience lemma that directly states: given `spec_initd_terminates_within(actual_outcomes)` and `actual_outcomes.len() <= fuel`, then `result.terminated == true`. This is derivable from existing proofs but would reduce caller burden.

- **Location:** `drain_remaining_zombies()` (exec, external_body)
  **Description:** Unchanged from R1. Post-loop zombie drain is `ensures true`. Acceptable for T2 boundary scope.
  **Suggested Fix:** No action needed.

## New Issues Introduced by Fixes

None. The additions are clean:
- `has_error` in `ScoreBoardPollResult` is properly constrained by `has_error ==> !has_call` and doesn't break any existing postconditions.
- `termination_pid` in `LifecycleStepResult` and `LoopResult` is threaded correctly through the loop invariant.
- `spec_handler_terminated_correctly` and `lemma_handler_top_level_correctness` are sound — the preconditions of the lemma exactly match `kcall_handler_loop`'s ensures.

## Positive Observations

All positive observations from R1 remain valid, plus:

- **Top-level correctness theorem**: The new `spec_handler_terminated_correctly` + `lemma_handler_top_level_correctness` pair provides a clean, usable top-level specification. The spec predicate clearly separates what is verified (termination trigger, loop invariant) from what is trusted (exit_status correctness).
- **Termination PID tracking**: The `termination_pid` field threaded through `LifecycleStepResult` → loop invariant → `LoopResult` is a meaningful addition that closes the gap between "loop terminated" and "loop terminated because of INITD."
- **Improved documentation quality**: Source citations in `spec_classify_handler_kcall`, explicit `ensures true` with rationale on `signal_handled`, and `has_error` field documentation all improve the model's auditability.
- **Scoreboard error modeling**: The `has_error` field with `has_error ==> !has_call` postcondition elegantly proves that scoreboard errors are safe (no-work outcome) without requiring the model to handle error paths explicitly in verified code.
- **48 verification conditions pass** (up from 47), 0 errors. The additional lemma is sound.
- **No regressions**: All original verified properties are preserved.

## Summary

The prover addressed the R1 review issues effectively. Of the 2 High issues, one (fuel gap + top-level theorem) is fully fixed with a sound `spec_handler_terminated_correctly` + `lemma_handler_top_level_correctness` pair. The other (exit_status unconstrained) is partially fixed — `termination_pid` is now tracked and proven to be INITD, but `exit_status` remains intentionally unconstrained as a documented T2 boundary limitation. All 4 Medium issues are resolved: scoreboard errors are modeled, `signal_handled` is documented, kcall constants have source citations, and `exit_status` in `harvest_zombies` is explicitly documented as out-of-scope.

The verification is now a complete and sound model of the handler's control-flow properties within its stated trust boundaries. The remaining gaps (exit_status value correctness, drain loop termination) are genuinely outside the handler module's verification scope and are well-documented. The addition of the top-level correctness theorem makes the verification usable by downstream modules.

**Grade improvement rationale**: A- → A. The top-level correctness theorem, termination PID tracking, scoreboard error modeling, and improved documentation collectively raise the verification from "very good with notable gaps" to "comprehensive within stated scope." No remaining High or Medium issues.
