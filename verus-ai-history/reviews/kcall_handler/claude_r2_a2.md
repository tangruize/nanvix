# Review: kcall_handler (claude-opus-4.6)

## Grade: A

## Previous Issue Resolution

### High Issues (3 from R1)

1. **`run_full_iteration()` yield logic divergence** — **FIXED.**
   The prover added a detailed comment (handler.rs:703-710) explaining why the `!result.should_terminate` guard is necessary in the sequential model: in the original, INITD termination causes `break status` during the harvest phase, exiting the loop before reaching the yield check. In the model, `run_full_iteration` always returns, so the guard suppresses the yield that would never be reached in the original. The reasoning is correct and well-documented. The guard is not just a cosmetic difference — it is *required* for semantic equivalence.

2. **`poll_scoreboard_full()` no postcondition** — **FIXED.**
   A postcondition was added: `!result.has_call ==> result.kcall_number == 0u32` (handler.rs:736). Additionally, a comment (lines 727-729) explains that `kcall_number` is intentionally unconstrained when `has_call` is true because `classify_and_check_invalid` handles all `u32` values — any out-of-range number maps to `Invalid` and returns `InvalidSysCall`. This justification is sound: the classification function is total over `u32`, so no range constraint is needed.

3. **`harvest_zombies()` unconstrained `exit_status`** — **FIXED (via documentation).**
   A comment was added (handler.rs:295-299) explaining that `exit_status` is intentionally unconstrained because its correctness depends on ProcessManager state (T2) and is outside the handler's verification scope. This is a legitimate trust boundary — the handler cannot verify the semantics of a value originating from process termination state.

### Medium Issues (4 from R1)

4. **`spec_classify_handler_kcall` u32::MAX handling** — Was already a non-issue in R1. `u32::MAX` is tested in `lemma_invalid_kcall_classification` (proof line 118). No change needed.

5. **`dispatch_to_subsystem()` no postcondition** — **FIXED.**
   A postcondition was added: `result.is_error ==> result.error_code != 0i32` (handler.rs:254). This is a sound basic invariant consistent with the Nanvix error model where error codes are always non-zero.

6. **`kcall_handler_loop()` fuel semantic gap** — **FIXED (via documentation).**
   A detailed "Semantic gap" paragraph was added (handler.rs:875-881) explaining that fuel exhaustion has no counterpart in the original code, and proving total termination would require a liveness assumption about INITD. This is a correct and honest assessment of the model's limitation.

7. **`event_init()` panic path not modeled** — **FIXED (via documentation).**
   Comments added (handler.rs:347-350) explicitly noting that the original panics on failure and the verification assumes successful initialization.

### Low Issues (3 from R1)

8-10. All three Low issues (signal_handled fallibility, empty proof bodies, spec type mismatch) were correctly identified as no-action-needed in R1. No changes were required.

## New Issues Check

### Critical

_None._

### High

_None._

### Medium

_None._

### Low

- **Location:** `poll_scoreboard_full()` postcondition (exec, handler.rs:736)
  **Description:** The `!has_call` case constrains `kcall_number` to `0u32`, which happens to be the `Debug` kcall number. If the struct were ever read without checking `has_call` (a caller bug), it would appear as a valid Debug call rather than an obviously invalid sentinel. Using `u32::MAX` (the `Invalid` sentinel in the source) would be marginally safer for defense-in-depth. However, since `kcall_number` is documented as "meaningful only when `has_call` is true" and no verified code reads it without the guard, this is purely cosmetic.
  **Suggested Fix:** No action required. Optionally consider `u32::MAX` as the no-call sentinel for marginally better ergonomics.

## Positive Observations

- **All actionable issues addressed.** Every High and Medium issue from R1 received either a code fix (postconditions added) or proper documentation explaining why the current approach is correct. No issues were dismissed without justification.
- **Comprehensive dispatch classification.** All 21 kcall numbers are correctly mapped. The classification function is total over `u32`, and the regression lemmas (`lemma_dispatch_coverage_matches_source`, `lemma_classification_correctness`) lock down the mapping against drift from the original source.
- **Sound loop invariant.** The inductive proof over `Seq<HarvestOutcome>` is clean: base case (`lemma_loop_invariant_base`), inductive step (`lemma_loop_invariant_inductive`), and termination exclusion (`lemma_invariant_excludes_termination`) form a complete argument that INITD was never seen while the loop was running.
- **Correct harvest notification semantics.** The model correctly captures that `harvested_process` is only set when: (1) a zombie was found, (2) it was NOT INITD (which breaks the loop first), and (3) `notify_termination` succeeded. This precisely mirrors the original's control flow where INITD hits `break status` before reaching the notification path.
- **Exec-to-spec bridge is well-founded.** `spec_harvest_to_outcome` + `lemma_harvest_to_outcome_termination` provides a formally verified link between exec-level boolean flags and spec-level enum reasoning, with the precondition `error ==> !found` matching the `harvest_zombies` postcondition.
- **Trust boundaries are clearly delineated.** The 5 trust boundaries (T1-T5) are documented with specific justifications for what is assumed vs. verified. External bodies carry meaningful postconditions where possible (harvest_zombies bidirectional constraint, dispatch_to_subsystem error code invariant).
- **Feature flag modeling is correct.** The `stdio_enabled` parameter in `poll_messages_gated` accurately models the `cfg_if!` conditional, ensuring non-stdio builds have `message_received = false` — which is verified to prevent phantom work from suppressing yields.
- **Clean split quality.** Spec (view types, spec functions in handler.spec.rs), proof (27 lemmas in handler.proof.rs), and exec (structures, external bodies, verified functions in handler.rs) are cleanly separated. The `include!` mechanism keeps the files independent while sharing the Verus module scope.
- **Verification passes cleanly.** 45 verified, 0 errors.

## Summary

The prover has genuinely addressed all issues from the R1 review. The three High issues received substantive fixes: the yield logic divergence is now documented with a thorough explanation of why the `!should_terminate` guard is necessary for semantic equivalence; `poll_scoreboard_full` gained a postcondition with documentation for why the `has_call` case is intentionally unconstrained; and the `exit_status` trust boundary is explicitly documented. The Medium issues were similarly resolved through postcondition additions and documentation.

No new issues were introduced by the fixes. The verification remains sound (45/0), the model faithfully captures the original's control flow, and the documentation is comprehensive. The only remaining observation is a cosmetic choice about the no-call sentinel value. Grade: **A**.
