# Review: kcall_wait_cond (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### Medium
- **Equivalence Gap**: The verification targets `wait_cond_model`, which is a manual reimplementation of `wait_cond` (lines 65-131) using model functions. There is no automated mechanism to ensure `wait_cond` and `wait_cond_model` remain semantically equivalent. If the implementation in `wait_cond` changes, the model could become stale, leading to false confidence.
  - **Location**: `wait_cond.rs` (exec)
  - **Suggested Fix**: Add a prominent comment to `wait_cond` warning that any changes must be reflected in `wait_cond_model`. Ideally, refactor the code to verify `wait_cond` directly by abstracting `ProcessManager` calls behind a verifiable interface or `external_body` wrappers that can be used by both the implementation and the proof.

### Low
- **Mixed Content**: The `src/kernel/src/pm/kcall/wait_cond.rs` file contains both the unverified production code (`wait_cond`) and the verification model code (`wait_cond_model`, `GetCondOutcomeModel`, etc.). This mixes concerns and might include model code in the production build if not properly gated.
  - **Location**: `wait_cond.rs` (exec)
  - **Suggested Fix**: Move the model code to a separate module (e.g., `wait_cond.model.rs`) or ensure it is guarded by `#[cfg(test)]` or `#[cfg(verus_keep_ghost)]` to prevents it from polluting the kernel binary.

## Positive Observations
- **Comprehensive Specification**: The `spec_wait_cond_result` function and associated lemmas (like `lemma_result_exhaustive`) provide a complete and rigorous specification of the function's behavior, including complex error propagation and short-circuiting logic.
- **Accurate Modeling**: The `wait_cond_model` faithfully reconstructs the non-trivial control flow of the original function, particularly the "stored result" logic where `cond.wait` outcome is preserved even if subsequent cleanup steps fail (unless they fail with an error).
- **Clean Interface Modeling**: The use of `external_body` functions (T1-T7) to model the `ProcessManager` dependencies is a sound approach for verifying this component in isolation.
- **Safety Proofs**: The verification correctly proves that the mutex is released before waiting and reacquired afterwards, which is the critical safety property for a condition variable wait.

## Summary
The verification of `kcall_wait_cond` is high quality, with a detailed spec and robust proofs for the modeled logic. The primary limitation is the structural separation between the actual execution code and the verified model, which requires manual maintenance to ensure equivalence. The "split" verification strategy is applied correctly here, using `external_body` to define trust boundaries around the complex `ProcessManager` dependencies.
