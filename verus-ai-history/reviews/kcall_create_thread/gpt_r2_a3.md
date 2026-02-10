# Review: kcall_create_thread (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `copy_from_user` external body and `create_thread_model` (exec)
  - **Description:** The new `ghost_src_addr` parameter is threaded through, but the model still does not connect the copied `ThreadCreateArgsView` to the validated source address. `copy_from_user` has no postcondition relating `ghost_src_addr` and `ghost_args_view`, and `lemma_copy_source_matches_validated_address` is tautological. This means the model can still validate arbitrary `thread_args` unrelated to the data at `arg0`.
  - **Suggested Fix:** Make `copy_from_user` return the `ThreadCreateArgsView` (or add an ensures clause tying `ghost_args_view` to the data at `ghost_src_addr`) and require `create_thread_model` to use that returned view. Add a non-trivial lemma that links the step-1 validated address to the copied args view.

- **Location:** `THREAD_CREATE_ARGS_SIZE()` and `assert_thread_create_args_size()` (spec/exec)
  - **Description:** The constant remains hard-coded and the new bridge function is never called anywhere, so verification still does not check the runtime `size_of::<ThreadCreateArgs>()`. Drift can occur without detection.
  - **Suggested Fix:** Add a Verus harness or build-time test that calls `assert_thread_create_args_size(core::mem::size_of::<ThreadCreateArgs>() as u32)` so CI enforces equality.

- **Location:** `USER_STACK_SIZE()` and `assert_user_stack_size()` (spec/exec)
  - **Description:** The stack-size constant is still hard-coded and the bridge function is unused, so there is no enforced link to `config::memory_layout::USER_STACK_SIZE`.
  - **Suggested Fix:** Add a harness/test calling `assert_user_stack_size(config::memory_layout::USER_STACK_SIZE as u32)` and wire it into the verification run.

### Low
- None.

## Positive Observations
- The API mapping/trust-boundary docs now reflect the extra ghost parameters for `copy_from_user`.
- Address fields are carried in `ThreadCreateArgsView` and are consistently linked to validation calls.

## Summary
The update improves the documentation and threads the copy source address, but it still does not prove that the copied arguments actually come from the validated `arg0` pointer, and the new bridge functions are not used to prevent constant drift. Verification quality improved slightly, yet there remain medium-severity gaps in semantic linkage and constant synchronization.
