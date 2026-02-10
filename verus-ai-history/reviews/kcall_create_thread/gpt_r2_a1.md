# Review: kcall_create_thread (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- **Location:** `ThreadCreateArgsView` / `ThreadCreateArgsModel` (spec/exec) and `create_thread_model` (exec)
  - **Description:** The model does not include the concrete addresses (`user_fn`, `user_stack_base`, `user_tda`) from `ThreadCreateArgs`. The validation booleans are independent of the addresses, and the ghost addresses used in `is_user_addr/is_user_region` are not linked to the copied arguments. As a result, the verification does not prove that the checks are performed on the actual user-provided addresses or that the PM call receives arguments derived from the copied structure, weakening semantic equivalence.
  - **Suggested Fix:** Extend `ThreadCreateArgsView`/`ThreadCreateArgsModel` to include the concrete address fields and connect them to the ghost addresses. Add a relation between `copy_from_user` success and the produced `ThreadCreateArgsView`, then use those fields in validation and in the PM call.

### Medium
- **Location:** `CreateThreadInputView.args_size` and `create_thread_model` preconditions (spec/exec)
  - **Description:** There is no constraint that `args_size` equals `size_of::<ThreadCreateArgs>()` or that `arg0` equals `KcallArgs.arg0`. This makes the spec permissive to incorrect size/address checks and weakens equivalence with the original implementation.
  - **Suggested Fix:** Add a spec constant for `THREAD_CREATE_ARGS_SIZE()` and a precondition equating `ghost_args_size` to it, and tie `ghost_arg0` to the `KcallArgs` view used by the model.

- **Location:** `USER_STACK_SIZE()` spec constant and `lemma_user_stack_size_matches_config` (spec/proof)
  - **Description:** The stack size constant is hard-coded to `524288` with no linkage to `config::memory_layout::USER_STACK_SIZE`. If the kernel constant changes, the verification can still pass while the model diverges from the implementation.
  - **Suggested Fix:** Import the config constant into the Verus model (or define a shared spec constant) and prove equality, or generate the spec constant from the Rust source to avoid drift.

### Low
- None.

## Positive Observations
- All validation steps, error propagation rules, and short-circuit behavior are explicitly modeled and proven.
- The exec/spec/proof split is clean, and trust boundaries are well documented.
- The model captures the InvalidArgument vs. copy/PM error distinctions and success equivalence cleanly.

## Summary
The verification thoroughly checks the control-flow pipeline and error propagation, but it abstracts away critical data-flow ties between copied arguments and validated addresses. Strengthening the model to bind address fields and size constants to the real inputs would materially improve semantic equivalence and robustness against implementation drift.
