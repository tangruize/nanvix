# Review: kcall_handler (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

_None._

### High

- **Location:** `run_full_iteration()` (exec, handler.rs:693)
  **Description:** Yield logic diverges from the original source. In the original `kcall_handler` (handler.rs:187-192), the yield occurs when `!kcall_handled && !message_received && !harvested_process`, with **no check for `!should_terminate`**. The INITD termination `break` (line 167) exits the loop *during* the harvest phase, before reaching the yield check, so the `!result.should_terminate` guard in `run_full_iteration` is unnecessary in practice. However, this means the verified model introduces an explicit short-circuit (`if result.should_yield && !result.should_terminate`) that doesn't exist in the original. While functionally equivalent because the `break` would have already exited, this is a semantic divergence from the original control flow that could mask bugs if the model changes.
  **Suggested Fix:** Add a comment documenting why this is equivalent, or restructure to match the original control flow more closely (i.e., check termination before yield in the lifecycle step, not inside the iteration).

- **Location:** `poll_scoreboard_full()` (exec, handler.rs:710-714)
  **Description:** The `poll_scoreboard_full` external body has **no postcondition at all**. This means Verus has no constraint on the returned `ScoreBoardPollResult`. The `has_call` and `kcall_number` fields are completely unconstrained. While the scoreboard protocol is stated to be verified separately, the lack of any postcondition (e.g., `has_call ==> kcall_number < u32::MAX`) means callers get no guarantees about the poll result, weakening downstream proofs that rely on the classification function.
  **Suggested Fix:** Add at minimum a postcondition constraining valid kcall number ranges, e.g., `result.has_call ==> result.kcall_number <= 31 || result.kcall_number == u32::MAX` (or document why this is intentionally unconstrained).

- **Location:** `harvest_zombies()` postcondition (exec, handler.rs:295-299)
  **Description:** The postcondition ties `is_initd` to `pid == 1u32` but does not constrain `exit_status`. The `exit_status` value propagates all the way to the final `LoopResult.exit_status` that becomes the return value of `kcall_handler`. Since `exit_status` is unconstrained in the external body, nothing in the verification connects the returned exit status to the actual process exit status. This is a specification gap — the verified code cannot guarantee it returns the correct exit status.
  **Suggested Fix:** Acknowledge this as a documented trust boundary limitation or add a postcondition like `result.found ==> result.exit_status <= MAX_EXIT_STATUS`.

### Medium

- **Location:** `spec_classify_handler_kcall` (spec, handler.spec.rs:185-208)
  **Description:** The spec function takes `u32` but the `KcallNumber::Invalid` variant is `u32::MAX` in the source. The spec maps all numbers outside 0-31 (except the defined ones) to `Invalid`, which is correct. However, the spec does not explicitly handle `u32::MAX` as a special case — it falls through to the `Invalid` catch-all. This is correct behavior but means the spec doesn't distinguish between "truly invalid numbers" and "the explicit Invalid sentinel value (u32::MAX)." A minor spec weakness.
  **Suggested Fix:** Consider adding `u32::MAX` as an explicit test case in `lemma_invalid_kcall_classification` (already present — this is a non-issue; confirmed at line 118).

- **Location:** `dispatch_to_subsystem()` (exec, handler.rs:251-255)
  **Description:** The `dispatch_to_subsystem` external body has **no postcondition**. While each subsystem's correctness is outside scope (per T2), the complete lack of postcondition means there is no modeled contract for what a subsystem dispatch returns. Even a minimal postcondition (e.g., `result.is_error ==> result.error_code != 0`) would strengthen the model.
  **Suggested Fix:** Add a basic postcondition capturing the invariant that error results have non-zero error codes.

- **Location:** `kcall_handler_loop()` (exec, handler.rs:851-884)
  **Description:** The `fuel` parameter is a modeling workaround for Verus's termination checker but introduces a semantic gap: in the original, the loop runs until INITD terminates with no bounded iteration count. The `LoopResult.terminated` can be `false` if fuel is exhausted, which has no counterpart in the original code. The loop invariant is preserved regardless, so safety properties hold, but the fuel abstraction means the full lifecycle model doesn't prove that the handler *always* returns an `ExitStatus` (it could return `terminated: false`).
  **Suggested Fix:** Document this as a known limitation in the proof file's header. Consider adding a ghost proof that given sufficient fuel (i.e., INITD eventually terminates), `terminated == true`.

- **Location:** Original handler.rs:52-54 vs verified model
  **Description:** The original `kcall_handler` calls `event::init(hal)` and **panics on failure** (`panic!("failed to initialize event manager")`). The verified `event_init()` external body has no postcondition and no error modeling. The panic path is not represented in the verification model. While panics are typically outside scope of functional verification, this is a divergence worth noting.
  **Suggested Fix:** Add a comment in the exec file noting that `event_init()` may panic and that the verification assumes successful initialization.

### Low

- **Location:** `signal_handled()` (exec, handler.rs:230-234)
  **Description:** The `signal_handled` external body has no postcondition. In the original, `scoreboard.handled(ret)` can fail (`Err(e)` → `warn!`), but the verified model doesn't model this failure path. The original sets `kcall_handled = true` regardless of whether `handled()` succeeds. The verification correctly sets `kcall_handled = true` in `handle_kcall_phase` unconditionally, which matches. However, the lack of modeling for the signal failure means we don't verify that warnings are emitted on failure.
  **Suggested Fix:** No action needed; the exec logic correctly matches the original behavior. Could optionally model the fallibility for completeness.

- **Location:** Proof file structure (handler.proof.rs)
  **Description:** Several proof lemmas have empty bodies (e.g., `lemma_classification_totality`, `lemma_classification_correctness`, `lemma_getpid_gettid_return_invalid`). While Verus can often discharge these automatically, the empty bodies don't make it clear whether the proof is trivial or relies on SMT solver heuristics. For critical proofs, explicit proof structure would be more robust against solver instability.
  **Suggested Fix:** No immediate action needed; Verus verifies them (45 verified, 0 errors). Consider adding brief proof sketches for the most important lemmas if solver performance becomes an issue.

- **Location:** `SPEC_ERROR_INVALID_SYSCALL` (spec, handler.spec.rs:31)
  **Description:** The spec constant is defined as `int` (value 88) but `HandlerKcallResult.error_code` is `i32`. The make_invalid_syscall_error function casts with `as i32`. While 88 fits in i32, the type mismatch between spec (`int`) and exec (`i32`) is a minor style inconsistency.
  **Suggested Fix:** Cosmetic only; no functional impact.

## Positive Observations

- **Comprehensive dispatch classification:** All 21 kcall numbers from the original match statement are correctly mapped to their categories with regression-style proof lemmas. The kcall number constants (verified against `src/libs/sys/src/sys/number.rs`) are all correct.
- **Strong loop invariant model:** The inductive loop invariant over a ghost history sequence is well-designed. The base case, inductive step, and termination exclusion are all proven. The `spec_extend_history` + `lemma_loop_invariant_inductive` pattern is a clean formalization of the "INITD never terminated in prior iterations" property.
- **Correct termination semantics:** The `spec_should_terminate` + `is_initd_terminated` chain correctly captures that only INITD termination (pid=1) exits the loop. The bidirectional postcondition on `harvest_zombies()` (`is_initd <==> found && pid == 1`) is well-specified.
- **Feature flag modeling:** The `stdio_enabled` parameter with `poll_messages_gated` correctly models the `cfg_if!` conditional compilation, ensuring non-stdio builds have `message_received = false`.
- **Yield correctness:** The yield-iff-idle property is cleanly specified and proven through the work flag monotonicity lemmas and the `should_yield` postcondition.
- **Exec-to-spec bridge:** The `spec_harvest_to_outcome` function with `lemma_harvest_to_outcome_termination` provides a clean bridge between exec-level boolean flags and spec-level enum reasoning.
- **Verification passes cleanly:** All 45 verification conditions pass with no errors.
- **Excellent documentation:** The exec file contains a comprehensive API mapping table, trust boundary documentation, scope limitations, and verification model overview.
- **Proper split quality:** Spec (view types, spec functions), proof (lemmas), and exec (structures, external bodies, verified functions) are cleanly separated with appropriate `include!` directives.

## Summary

The kcall_handler verification is a well-structured and thorough formalization of the kernel's main event loop. It correctly captures the core safety properties: dispatch routing totality, yield-iff-idle, INITD-only termination, work flag monotonicity, and loop invariant preservation. The kcall number mappings are verified correct against the source definitions.

The main areas for improvement are: (1) the `poll_scoreboard_full()` and `dispatch_to_subsystem()` external bodies lack postconditions, creating unconstrained trust gaps; (2) the yield logic adds an explicit `!should_terminate` guard not present in the original source; and (3) the fuel-bounded loop model cannot prove that the handler always terminates with a valid exit status.

None of these issues compromise the verified properties — they represent specification gaps rather than unsoundness. The verification successfully proves the properties it claims, the proofs are well-structured, and the documentation clearly delineates what is and is not in scope. Overall grade: **A-**, reflecting solid verification with room for tighter external body contracts.
