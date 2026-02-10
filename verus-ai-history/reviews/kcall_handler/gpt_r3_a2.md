# Review: kcall_handler (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `kcall_handler_loop` (exec, `verus/split/kernel/kcall/handler.rs`)  
  **Description:** The model remains fuel-bounded and can return `terminated = false` when fuel is exhausted, which still has no counterpart in the original infinite loop. This keeps a semantic gap and prevents proving that the handler always returns an `ExitStatus` under real execution.  
  **Suggested Fix:** Add a wrapper specification (or top-level theorem) that assumes INITD eventually terminates and discharges the fuel bound (e.g., link fuel to the liveness assumption), so the verified model proves termination for the real handler.

### Medium
- **Location:** `event_init` / `kcall_handler_init` (exec, `verus/split/kernel/kcall/handler.rs`)  
  **Description:** Event initialization failure still isn’t modeled; the verification assumes `event::init` always succeeds even though the original code panics on failure. This is an unmodeled fail-stop path.  
  **Suggested Fix:** Model the panic path explicitly (e.g., return an abort flag) or add a documented precondition on `kcall_handler_init` that initialization never fails.

- **Location:** `spec_handler_correct_under_liveness` / `lemma_oracle_connected_liveness` (spec/proof)  
  **Description:** The new oracle model is not actually connected to the exec loop. `lemma_oracle_connected_liveness` does not take an oracle or assume any correspondence between oracle outcomes and `harvest_zombies()`, so the liveness implication remains unproven. The comments claim an oracle bridge that is not encoded.  
  **Suggested Fix:** Add a lemma that explicitly relates the exec loop’s observed harvest outcomes to the oracle (e.g., history is a prefix of the oracle’s non-terminating outcomes) and use it to prove `spec_handler_correct_under_liveness`.

### Low
- **Location:** `kcall_handler` coverage mapping (exec docs, `verus/split/kernel/kcall/handler.rs`)  
  **Description:** There is still no verified wrapper with the original `kcall_handler(hal, mm, pm) -> ExitStatus` signature, which makes coverage traceability weaker.  
  **Suggested Fix:** Provide a thin verified wrapper that calls the modeled loop under stated assumptions (stdio flag + liveness).

## Positive Observations
- The scoreboard error path is now ruled out (`poll_scoreboard_full` ensures `!has_error`), aligning with the original `unreachable!()` semantics.
- The new oracle model documents the intended liveness story more clearly, even though it still needs a formal connection to exec outcomes.

## Summary
Some earlier issues were partially addressed (scoreboard errors), but the core semantic gap from fuel-bounding and unmodeled panic/liveness paths remain. Verification is improved yet still not complete or fully sound with respect to the real handler behavior.
