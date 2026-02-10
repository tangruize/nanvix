# Review: kcall_handler (gemini-3-pro-preview)

## Grade: A-

## Issues Found

### Medium
- **Manual Model Divergence Risk**: The verified code (`verus/split/kernel/kcall/handler.rs`) is a manual abstraction/model of the original source (`src/kernel/src/kcall/handler.rs`). It uses `external_body` functions to mimic the original implementation calls. If the original source changes (e.g., adding a new kcall, changing yield logic), the verified model will remain valid but will no longer reflect the actual system behavior. There appears to be no mechanism to enforce synchronization between the source and the verified model.
  - *Location*: `verus/split/kernel/kcall/handler.rs` vs `src/kernel/src/kcall/handler.rs`
  - *Fix*: Ideally, verify the source code directly. If a split model is necessary, implement a "sync check" or "model generation" step in CI that ensures the model structure matches the implementation (e.g., ensuring all match arms in the source are present in the model's classification).

### Low
- **Hardcoded Spec Constants**: Constants `SPEC_ERROR_INVALID_SYSCALL` (88) and `SPEC_INITD_PID` (1) are hardcoded in the specification. If the system definitions in `sys::error::ErrorCode` or `sys::pm::ProcessIdentifier` change, the verification will use stale values.
  - *Location*: `verus/split/kernel/kcall/handler.spec.rs` (Lines 28, 31)
  - *Fix*: Import the constants from the `sys` crate directly in the spec, or add a regression test that asserts the spec values match the system values.

- **Abstracted IKC Polling Logic**: The internal logic of the IKC message polling loop (batching `IKC_POLL_BATCH_SIZE`, checking `MAX_IKC_MESSAGES`) is abstracted away into the `poll_messages_raw` external body. The verification proves the handler yields correctly based on the result of this function, but it does not verify that the polling logic itself respects the batch/buffer constraints.
  - *Location*: `verus/split/kernel/kcall/handler.rs` (Line 264)
  - *Fix*: This is acceptable for a handler-level verification, but the scope limitation should be explicitly noted.

## Positive Observations
- **Strong Control Flow Verification**: The verification accurately captures the handler's control flow, particularly the "yield if and only if idle" property (`lemma_full_work_no_yield`, `lemma_initial_iteration_no_work`) and the "terminate only on INITD exit" property (`lemma_only_initd_terminates`).
- **Comprehensive Dispatch Modeling**: The `HandlerDispatchCategory` and associated lemmas (`lemma_classification_totality`, `lemma_classification_correctness`) prove that every possible `u32` kcall number is correctly classified and routed. This mathematically guarantees that no kcall number falls through the cracks or causes undefined routing behavior.
- **Clean Separation**: The project structure cleanly separates the executable model (`handler.rs`), the specifications (`handler.spec.rs`), and the proofs (`handler.proof.rs`). This separation of concerns makes the verification artifacts readable and maintainable.
- **Robust Invariants**: The `spec_loop_invariant` is simple yet powerful, effectively proving that the kernel loop continues precisely as long as the init daemon is alive, linking the high-level system lifecycle to the low-level loop iteration.

## Summary
The verification of `kcall_handler` is high-quality and sound. It successfully creates a formal model of the kernel's main event loop and proves critical safety and liveness properties. The approach uses a "shadow model" where the implementation is abstracted into a Verus-friendly form with `external_body` boundaries for complex subsystems. This is a pragmatic choice for verifying the orchestrator logic without verifying the entire kernel. The primary risk is the potential for the manual model to drift from the actual implementation, but the verification logic *within* the model is rigorous and complete relative to the stated goals.