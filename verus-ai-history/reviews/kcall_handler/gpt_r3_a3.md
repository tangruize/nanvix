# Review: kcall_handler (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `kcall_handler_loop` (exec, `verus/split/kernel/kcall/handler.rs`)  
  **Description:** The model remains fuel-bounded and can return `terminated = false` when fuel is exhausted, which still has no counterpart in the original infinite loop. This is a semantic gap that prevents proving the real handler always returns an `ExitStatus`.  
  **Suggested Fix:** Add a wrapper theorem or model for the real handler that assumes INITD eventually terminates and discharges the fuel bound (e.g., prove termination under the liveness/oracle assumption).

### Medium
- **Location:** `event_init` / `kcall_handler_init` (exec, `verus/split/kernel/kcall/handler.rs`)  
  **Description:** The panic path on event initialization failure is still not modeled; the verification assumes success via an external body postcondition and only documents this in comments. This leaves a real fail-stop path outside the model.  
  **Suggested Fix:** Encode the assumption as a formal precondition on `kcall_handler_init` or model the panic/abort path explicitly.

- **Location:** `spec_handler_correct_under_liveness` (spec, `verus/split/kernel/kcall/handler.spec.rs`)  
  **Description:** The spec claims liveness-correctness from `spec_oracle_has_termination` alone, but the proof (`lemma_oracle_connected_liveness`) additionally requires `spec_oracle_matches_history`. As written, the spec is too strong and not actually justified.  
  **Suggested Fix:** Strengthen the spec to include `spec_oracle_matches_history` (or a similar correspondence assumption), or provide a proof that does not require it.

### Low
- **Location:** `kcall_handler` coverage mapping (exec docs, `verus/split/kernel/kcall/handler.rs`)  
  **Description:** There is still no verified wrapper with the original `kcall_handler(hal, mm, pm) -> ExitStatus` signature, which weakens traceability to the source API.  
  **Suggested Fix:** Provide a thin verified wrapper that calls the modeled loop under stated assumptions (stdio flag + liveness).

## Positive Observations
- The scoreboard error path is now ruled out (`poll_scoreboard_full` ensures `!has_error`), aligning with the original `unreachable!()` semantics.
- The oracle model and `lemma_oracle_connected_liveness` now explicitly state the oracle-to-history correspondence assumption, improving clarity over the previous version.

## Summary
Some prior issues were fixed (scoreboard error path, explicit oracle correspondence), but the core semantic gap from fuel bounding remains and the liveness spec is still too strong relative to the proof. Verification is improved yet still not complete or fully sound with respect to the real handler behavior.
