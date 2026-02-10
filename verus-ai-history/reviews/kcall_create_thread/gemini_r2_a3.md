# Review: kcall_create_thread (gemini-3-pro-preview) - Iteration 3

## Grade: C

## Issues Found

### High
- **Verification Gap (Model vs. Implementation) - NOT FIXED**: The verification continues to target `create_thread_model`, a standalone shadow function, rather than the actual `create_thread` kernel function. The actual implementation remains completely unverified. There is no guarantee that the code running in the kernel matches the logic verified in the model. This is a critical soundness gap.
  - **Location**: `verus/split/kernel/pm/kcall/create_thread.rs` (exec)
  - **Status**: Persisting. The prover has not attempted to refactor this to target the actual function.

### Medium
- **Manual Sync of Spec Constants - NOT FIXED**: The suggested integration test (`src/tests/verification_sync.rs`) to enforce synchronization of spec constants was not created. The bridge functions `assert_thread_create_args_size` and `assert_user_stack_size` exist but appear unused in the test suite, leaving the synchronization manual and error-prone.
  - **Location**: `verus/split/kernel/pm/kcall/create_thread.rs` (exec)
  - **Status**: Persisting.

## Positive Observations
- **Argument Passthrough Fixed**: The low-priority issue regarding `user_fn_arg0` and `user_fn_arg1` was successfully addressed. The `ThreadCreateArgsModel` and `ThreadCreateArgsView` structs were updated, and a new lemma `lemma_args_passthrough_preserved` was added to prove these arguments are correctly passed to the process manager.

## Summary
While the prover successfully addressed the minor issue regarding argument passthrough, the two significant issues remain untouched. The verification relies on a shadow model approach that leaves the actual kernel code unverified, and the mechanism for syncing constants remains fragile. The "High" priority issue must be addressed for this to be considered a sound verification of the kernel component.
