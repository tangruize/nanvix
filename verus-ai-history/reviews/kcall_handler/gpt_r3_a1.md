# Review: kcall_handler (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- **Location:** `kcall_handler_loop` (exec, `verus/split/kernel/kcall/handler.rs`)  
  **Description:** The verified model is fuel-bounded and can return `terminated = false` when fuel is exhausted, which has no counterpart in the original `kcall_handler` (it runs until INITD terminates). This is a semantic gap that weakens equivalence and prevents proving the handler always returns a valid `ExitStatus` under the real execution model.  
  **Suggested Fix:** Add a wrapper spec for the real `kcall_handler` that assumes liveness (INITD eventually terminates) and proves termination without a fuel-exhaustion path, or encode fuel as a ghost bound tied to a liveness assumption (`spec_initd_terminates_within`) and prove `terminated == true` under that assumption.

### Medium
- **Location:** `poll_scoreboard_full` / `ScoreBoardPollResult` (exec, `verus/split/kernel/kcall/handler.rs`)  
  **Description:** The model allows `has_error = true` and then silently continues the loop (via `run_iteration` ignoring `has_error`), while the original code treats such errors as `unreachable!()` (panic). This weakens equivalence and allows behaviors that cannot occur in the real system.  
  **Suggested Fix:** Strengthen the model with a precondition `!result.has_error`, or model the panic path explicitly (e.g., by returning a termination/abort flag) so the verified behavior matches the original fail-stop semantics.

- **Location:** `event_init` (exec, `verus/split/kernel/kcall/handler.rs`)  
  **Description:** The external body assumes initialization always succeeds; the original `event::init(hal)` panics on failure. The panic path is not modeled, so the verification excludes a real execution path.  
  **Suggested Fix:** Model `event_init` as returning a `Result` and explicitly handle the failure case (abort/termination), or add a top-level precondition documenting the assumption that initialization never fails.

- **Location:** `spec_initd_terminates_within` / `lemma_loop_termination_completeness` (spec/proof, `handler.spec.rs` / `handler.proof.rs`)  
  **Description:** The liveness predicate is not connected to the exec model: the recorded history is defined to exclude termination by invariant, so `spec_initd_terminates_within(history)` is always false and cannot drive termination proofs. This leaves liveness unproven even under the stated assumption.  
  **Suggested Fix:** Introduce a separate “environment outcomes” trace (or a ghost trace of potential harvests) that can include termination, and relate it to the exec loop to prove `terminated == true` when INITD terminates within the bound.

### Low
- **Location:** `kcall_handler` coverage mapping (exec docs, `verus/split/kernel/kcall/handler.rs`)  
  **Description:** There is no verified function with the same signature as the original `kcall_handler(hal, mm, pm) -> ExitStatus`; the mapping relies on a documented correspondence to `kcall_handler_loop(fuel, stdio_enabled)`. This is acceptable for modeling, but it makes traceability and equivalence auditing harder.  
  **Suggested Fix:** Add a thin, verified wrapper with the original signature that calls the modeled loop under stated assumptions (stdio flag, liveness), to make coverage explicit.

## Positive Observations
- Dispatch classification is fully enumerated with regression lemmas that mirror the original match arms, reducing drift risk.
- Yield behavior and INITD-termination control flow are modeled explicitly, including the subtle “no yield on termination” path.
- Specs, proofs, and exec code are cleanly separated, with clear trust-boundary documentation and feature-flag gating.

## Summary
The verification captures the core safety control flow (dispatch, yield, INITD-only termination), but the fuel-bounded loop and unmodeled panic paths introduce real semantic gaps. Tightening the termination/liveness story and making error paths faithful to the original would materially improve equivalence and soundness.
