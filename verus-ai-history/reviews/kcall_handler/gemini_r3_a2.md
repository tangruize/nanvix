# Review: kcall_handler (gemini-3-pro-preview) - Iteration 2

## Grade: C

## Issues Found

### Critical
- **Shadow Verification (No Coverage of Real Code)**: The verification remains a "shadow model". While the documentation now correctly identifies this limitation and cites the "standard methodology," the fact remains that the running kernel code (`src/kernel/src/kcall/handler.rs`) is not verified. The model is a manual copy of the control flow. Any divergence between the source and the model (drift) renders the verification result moot for the actual system.

### High
- **Hardcoded Kcall Numbers**: The issue of hardcoded constants persists. `handler.spec.rs` defines `spec_classify_handler_kcall` using magic numbers (0, 1, 2...) instead of importing constants from `::sys::number::KcallNumber`.
  - The `sys` crate is available in `lib.rs`, so these constants should be accessible.
  - The "regression lemmas" in `handler.proof.rs` (`lemma_classification_correctness`) merely check the spec against itself (e.g., `spec_classify(0) == Debug`), which is a tautology, not a regression check against source truth.
  - *Fix Required*: Import `KcallNumber` and use it in the spec or proofs to ensure the model matches the source definitions.

### Medium
- **Unconstrained External Bodies**: `dispatch_to_subsystem` is still under-specified. It does not constrain the result based on the input `kcall_number`. The verification effectively assumes that "dispatching does something valid," but doesn't prove that `dispatch(NR_Debug)` actually calls the debug subsystem.
- **Liveness Assumption**: The liveness property is assumed (`spec_initd_terminates_within`) rather than derived. This is a known limitation for this component level, but remains a gap in the correctness argument.

## Positive Observations
- **Improved Documentation**: The update added a comprehensive "Verification Model" section that honestly describes the shadow model approach, trust boundaries (T1-T5), and drift mitigation strategies. This transparency is valuable.
- **Exec-to-Spec Linkage**: The rigorous mapping of harvest results to spec outcomes (`spec_harvest_to_outcome`) and the associated lemma (`lemma_harvest_to_outcome_termination`) provide a strong logical connection between the execution model and the specification properties.

## Summary
The verification has not substantively changed since the first review; only the documentation has been improved. The fundamental issue is that this is a **shadow model** that duplicates the target code rather than verifying it. While the prover argues this is "standard methodology," it introduces a critical risk of drift. Furthermore, the persistent use of **hardcoded magic numbers** for syscall dispatch—when the source constants are available—is a fixable flaw that makes the model brittle. The grade remains a **C** as the verification captures the *intended* logic of the loop but fails to mechanically link it to the *actual* source code or constants.
