# Review: kcall_handler (gemini-3-pro-preview)

## Grade: C

## Issues Found

### Critical
- **Shadow Verification (No Coverage of Real Code)**: The verification is performed on a "shadow model" (`verus/split/kernel/kcall/handler.rs`) which re-implements the control flow of the handler loop. The actual executable code (`src/kernel/src/kcall/handler.rs`) is neither compiled nor verified by the `verify.sh` command. The entry point `kcall_handler` is completely missing from the verified artifact, meaning the function actually called by the kernel is unverified.
- **Divergence Risk**: Because the model is manually maintained separately from the source, there is no mechanical guarantee that the verified properties apply to the running kernel. Any change to the original `handler.rs` requires a manual update to the shadow model, which is error-prone.

### High
- **Hardcoded Kcall Numbers**: The spec function `spec_classify_handler_kcall` hardcodes the mapping of integer kcall numbers to dispatch categories (e.g., `if number == 0 { ... }`). These constants are defined in `src/libs/sys/src/sys/number.rs` in the real codebase. If the system constants change, the verification model will silently become incorrect.
  - *Suggested Fix*: Import the constants from `::sys::number` (if available in the verification crate) or add a mechanism to ensure they match the source (e.g., a build-time check or a test that asserts equality).

### Medium
- **Unconstrained External Bodies**: The model relies heavily on `external_body` functions like `harvest_zombies`, `dispatch_to_subsystem`, and `poll_scoreboard_full`. While necessary for modularity, some return values are under-constrained. For example, `dispatch_to_subsystem` has no postcondition linking the input kcall number to the operation performed, effectively modeling it as a random result generator that respects error codes.
- **Missing Liveness Proof**: The "liveness" property (that the loop eventually terminates) is modeled as an assumption (`spec_initd_terminates_within`) rather than being proven from the properties of the system. While understandable given the dependency on userspace (INITD), the current model essentially assumes "it will stop because we say it will."

### Low
- **Spec Drift**: The `spec_classify_handler_kcall` function hardcodes 21 cases. The original source has a `match` statement. If a new kcall is added to the handler, the spec needs update.
- **Complexity of Model**: The model introduces significant complexity (`LoopIterationState`, `HandlerTermination`, `HarvestOutcome`, `Ghost` history) to verify what is essentially a `while` loop with a few flags. The ratio of verification code to "code being modeled" is very high.

## Positive Observations
- **Clear Trust Boundaries**: The `external_body` functions clearly delimit what is verified (control flow, state updates) vs. what is assumed (subsystem correctness, hardware behavior). The documentation for each boundary (T1-T5) is excellent.
- **Invariant Quality**: The loop invariant (history contains no terminating outcomes) is well-formulated and sufficient to prove the safety property (loop only exits on INITD termination).
- **Yield Correctness**: The model correctly captures the subtle logic of "yield if and only if no work was done", including the gating of IKC messages by the `stdio` feature flag.

## Summary
The current verification of `kcall_handler` is a **shadow model verification**. It proves that an abstract model of the handler loop satisfies certain correctness properties (yield discipline, termination condition). However, because the verified code is a completely separate re-implementation from the actual kernel source, it provides no mechanical guarantee about the correctness of the running OS. The grade is C because while the model itself is verified (passed), it fails the primary objective of verifying the *source code*. To improve this, the verification should target the actual `src/kernel/src/kcall/handler.rs` file, even if that requires more complex `external_body` definitions to mock dependencies.
