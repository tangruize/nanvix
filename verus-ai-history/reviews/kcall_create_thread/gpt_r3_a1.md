# Review: kcall_create_thread (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `create_thread_model` (exec) / overall module architecture.
  **Description:** The verification proves control-flow correctness over *abstract boolean inputs* and ghost addresses, but there is no refinement proof that these inputs correspond to the real `KcallArgs` fields or to the concrete `Vmem::is_user_region/is_user_addr` computations. This leaves equivalence to the real kernel function as an assumption: a caller could choose arbitrary booleans/ghost values and still satisfy the model’s preconditions.
  **Suggested Fix:** Add a verified wrapper or refinement lemma that derives the model inputs from a `KcallArgsView` (or directly from `KcallArgs`) and links them to VMM specs (e.g., `args_addr_valid == spec_is_user_region(arg0, size)` and `user_fn_valid == spec_is_user_addr(thread_args.user_fn)`), so the model is tied to concrete semantics.

### Medium
- **Location:** `THREAD_CREATE_ARGS_SIZE()` and `USER_STACK_SIZE()` (spec), `assert_*` bridges (exec).
  **Description:** The spec uses hard-coded constants for struct size and stack size, but the bridging `assert_thread_create_args_size` and `assert_user_stack_size` functions are never invoked. If the kernel layout or config changes, verification will still pass with stale constants, breaking equivalence silently.
  **Suggested Fix:** Add build-time checks or tests that call the bridge functions (or `static_assert` in the kernel) to bind spec constants to runtime values.

- **Location:** `copy_from_user` model + `create_thread_model` (exec).
  **Description:** The model treats `thread_args` as an independent input; `copy_from_user` does not return or constrain the actual copied values. This allows executions where `copy_from_user` “succeeds” but subsequent validation uses unrelated `thread_args`, which is weaker than the real behavior.
  **Suggested Fix:** Refactor the copy model to return a `ThreadCreateArgsModel` (or add a ghost output) and use that value for subsequent validations, with a postcondition tying it to the ghost view.

### Low
- None.

## Positive Observations
- The model mirrors the concrete control-flow structure and short-circuiting of the original function, with explicit proofs for each validation step.
- Error-code propagation and success conditions are captured precisely, including explicit lemmas for InvalidArgument and PM error passthrough.
- The spec/proof/exec split is clean, with thorough documentation of trust boundaries and out-of-scope properties.

## Summary
The verification is solid for the validation pipeline logic and error propagation, but it remains a *model-only* proof with weak refinement to concrete inputs and copy semantics. Binding spec constants to runtime values and linking the abstract booleans to VMM/PM specs would materially improve equivalence and soundness.
