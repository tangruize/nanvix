# Review: kcall_handler (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

None.

### High

- **Location:** `kcall_handler_loop()` (exec), `LoopResult` ensures (exec)
  **Description:** The `exit_status` field in `LoopResult` is completely unconstrained by the postconditions. When `terminated == true`, the model does not prove that `exit_status` equals the value returned by the INITD zombie harvest. The only constraint flows implicitly through `kcall_handler_lifecycle_step` → `run_full_iteration` → `run_iteration`, but `kcall_handler_loop` drops this connection when assigning `exit_status = step.exit_status` without an ensures clause. In the original, `kcall_handler` returns `ExitStatus` directly from the `break status` statement — a property central to handler correctness.
  **Suggested Fix:** Add ensures clause: `result.terminated ==> result.exit_status == <ghost tracked status>`. Thread a ghost variable through the loop tracking the exit status from the terminating harvest, and constrain it in the final postcondition.

- **Location:** `kcall_handler_loop()` (exec)
  **Description:** The fuel-based termination model introduces a semantic gap with the original code. The original `kcall_handler` runs an unbounded loop that *always* terminates when INITD exits — there is no concept of fuel exhaustion. When `fuel` is exhausted (`!result.terminated`), the model returns a result that has no counterpart in the original. The postconditions prove properties for the fuel-exhaustion case that never occurs in production. This means the model cannot prove total correctness (handler always returns a valid ExitStatus). The documentation acknowledges this but it remains a fundamental model limitation.
  **Suggested Fix:** Document more prominently that the fuel parameter is a verification artifact. Consider adding a top-level spec function `spec_handler_correct(exit_status: nat) -> bool` that states the expected end-to-end property under the liveness assumption `spec_initd_terminates_within`, connecting the conditional liveness proof to a usable top-level correctness theorem.

### Medium

- **Location:** `poll_scoreboard_full()` (exec, external_body)
  **Description:** The original code has two `unreachable!()` paths: (1) `ScoreBoard::get_mut()` returns `Err` (line 119), and (2) `scoreboard.handle()` returns an error other than `TryAgain` (line 112). These are defense-in-depth assertions that the scoreboard is always accessible and only returns expected errors. The verified model's `poll_scoreboard_full()` external body simply elides these paths entirely — no error variant exists in `ScoreBoardPollResult`. While documented under T1, this means the verification assumes scoreboard infallibility without proving it or even requiring it as an explicit precondition.
  **Suggested Fix:** Add a precondition or documented ghost invariant expressing that the scoreboard is in a valid accessible state. Alternatively, add a `has_error: bool` field to `ScoreBoardPollResult` and prove that when `has_error` is true, no work is done and the iteration proceeds correctly (matching the original's `unreachable!` / continue behavior).

- **Location:** `run_iteration()` (exec), original handler lines 100-105
  **Description:** In the original, `scoreboard.handled(ret)` can fail (`Err(e)` at line 101-102), logging a warning but still setting `kcall_handled = true`. The verified model's `signal_handled()` is a void external body with no error modeling. The fact that the kcall is considered "handled" even when signaling fails is correct behavior, but the model doesn't verify this subtle invariant — it's invisible because `signal_handled` has no postcondition at all.
  **Suggested Fix:** Consider adding a comment or a trivial postcondition (`ensures true`) with documentation that the "handled" flag is set regardless of signaling success, matching the original's behavior where `kcall_handled = true` follows the if-let on line 101.

- **Location:** `harvest_zombies()` (exec, external_body)
  **Description:** The `exit_status` field in `ZombieHarvestResult` is completely unconstrained by the postcondition. While the documentation notes this is intentional (exit status depends on PM state, T2), this is the source of the High-priority exit status propagation gap. Without any constraint on `exit_status`, the model cannot prove that the handler returns the correct status even when INITD terminates.
  **Suggested Fix:** If end-to-end exit status correctness is desired, add a ghost parameter or tracked field that links `exit_status` to a spec-level process termination model.

- **Location:** `spec_classify_handler_kcall()` (spec), handler.spec.rs lines 185-208
  **Description:** The dispatch classification uses hardcoded integer literals (0, 1, 2, 4, 6, ...) without referencing the `KcallNumber::NR_*` constants. If the syscall number assignments change in `src/libs/sys/src/sys/number.rs`, the spec must be updated manually. The `lemma_dispatch_coverage_matches_source()` regression check in the proof file documents the expected values but cannot detect source drift automatically — it only checks that the spec is self-consistent.
  **Suggested Fix:** Add a comment in `spec_classify_handler_kcall()` citing the source file and line for each constant. Consider a CI check that compares the constants against the source definitions.

### Low

- **Location:** `drain_remaining_zombies()` (exec, external_body)
  **Description:** The post-loop zombie drain is modeled as a no-op external body with `ensures true`. The original code's `while let Ok(Some((pid, status))) = pm.harvest_zombies(mm) { info!(...) }` is simple but its termination property (the drain loop eventually terminates) is not verified. The property that "no zombies remain after drain" is acknowledged as depending on PM state.
  **Suggested Fix:** Low priority. If desired, model the drain as a bounded loop similar to `kcall_handler_loop` with a fuel parameter, proving that each iteration makes progress (fewer zombies remain).

- **Location:** `event_init()` (exec, external_body)
  **Description:** `event_init()` has `ensures true`, modeling the assumption that event initialization always succeeds. The original panics on failure (`panic!("failed to initialize event manager")`). The assumption is well-documented but means the model does not cover the failure path at all.
  **Suggested Fix:** No action needed — the panic path aborts the kernel, so the handler loop never starts. The documentation is adequate.

- **Location:** `poll_scoreboard_full()` (exec, external_body), postcondition
  **Description:** The postcondition `!result.has_call ==> result.kcall_number == 0u32` constrains `kcall_number` to 0 when no call is pending. This is a reasonable default, but 0 corresponds to `KcallNumber::Debug`. If a code path accidentally uses `kcall_number` when `has_call` is false, it would classify as Debug rather than Invalid. The verified code correctly gates on `has_call` in `handle_kcall_phase()`, so this is not exploitable, but the default value choice is mildly confusing.
  **Suggested Fix:** Consider using a sentinel value (e.g., `u32::MAX` which maps to Invalid) as the default `kcall_number` when `!has_call`, or document why 0 was chosen.

- **Location:** Spec file (handler.spec.rs), `SPEC_ERROR_INVALID_SYSCALL()`
  **Description:** The spec constant is defined and regression-tested but never actually used in any ensures clause that connects to exec behavior. `make_invalid_syscall_error()` hardcodes `88i32` directly and its ensures references the spec constant, but no higher-level postcondition on `run_iteration` or `kcall_handler_loop` exposes the error code to callers. The constant exists for documentation but has limited verification utility.
  **Suggested Fix:** No action needed for correctness. If error code propagation becomes important, thread it through the iteration results.

## Positive Observations

- **Excellent documentation**: All three files (spec, proof, exec) have thorough documentation. The exec file's module-level doc comment includes an API mapping table, trust boundary descriptions, scope limitations, and verified properties list — exceptional for a verification module.
- **Clean trust boundaries**: The five trust boundaries (T1-T5) are clearly identified, each external body is annotated with its trust boundary, and the postconditions on external bodies are neither too strong (no unsound assumptions) nor too weak (key structural invariants like `is_initd ↔ (found && pid==1)` are captured).
- **Rigorous loop invariant proof**: The inductive loop invariant proof chain (`lemma_loop_invariant_base` → `lemma_loop_invariant_inductive` → `lemma_invariant_excludes_all_termination` → `lemma_loop_termination_completeness`) is correct and well-structured. The conditional liveness property (contrapositive: if INITD terminates within fuel, the loop returns terminated) is a meaningful theorem.
- **Accurate control flow modeling**: The subtle behavior where INITD termination `break`s before the yield check is correctly modeled via the `!result.should_terminate` guard in `run_full_iteration()` (line 724), with an excellent comment explaining the modeling choice.
- **Harvest notification semantics**: The model correctly captures that `harvested_process` is only set when (a) a zombie was found, (b) it was not INITD, and (c) `notify_termination` succeeded — matching the original's three-way condition.
- **Feature flag modeling**: The `stdio_enabled` parameter with `poll_messages_gated()` cleanly models the `cfg_if!` compile-time feature gate, with a proven postcondition that non-stdio builds never report message work.
- **Regression lemmas**: `lemma_spec_constants_match_source()` and `lemma_dispatch_coverage_matches_source()` provide valuable drift detection, documenting expected constant values and kcall number coverage with source file citations.
- **All 47 verification conditions pass** with no errors, indicating a sound and complete proof within the stated trust boundaries.
- **Kcall number mappings verified correct**: Cross-checked all 21 handler kcall numbers against `src/libs/sys/src/sys/number.rs` constants — all match exactly.
- **INITD PID (1) and ENOSYS (88) constants verified correct** against `src/libs/sys/src/sys/pm/pid.rs:60` and `src/libs/sysapi/src/errno.rs:173`.

## Summary

This is a high-quality control-flow verification of the kernel call handler event loop. The verification accurately models the handler's three-phase iteration (kcall dispatch, IKC polling, zombie harvest), yield-when-idle behavior, and INITD-triggered termination. The 21 kcall dispatch routes are classified correctly and proven exhaustive.

The main limitations are: (1) exit status propagation is unverified — the model cannot prove that the handler returns the correct exit status from INITD, (2) the fuel-based loop model cannot prove total termination without a liveness assumption, and (3) individual subsystem dispatch correctness is outside scope (by design). These are reasonable scope choices for a control-flow module, but the exit status gap (#1) should be addressed since it's the handler's primary functional output.

The specification quality is strong — specs are neither too weak (they capture yield correctness, termination conditions, dispatch routing, and the loop invariant) nor too strong (they don't over-constrain external subsystem behavior). The proof structure is clean with good separation of concerns. The documentation is exemplary for a verification module.

**Recommendations for future work:**
1. Thread exit status through ghost state to prove end-to-end correctness.
2. Add a top-level correctness theorem connecting the conditional liveness proof to a usable specification.
3. Consider CI automation to detect kcall number constant drift.
