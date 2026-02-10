# Review: kcall_create_thread (gpt-5.2-codex)

## Grade: B+

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `copy_from_user` external body / `create_thread_model` (exec)
  - **Description:** The data-flow from `args.arg0` to the copied `ThreadCreateArgsView` is still not modeled. `copy_from_user` does not take the source address (arg0) or relate it to the `thread_args` view used for validation, so the model permits validations over arbitrary args unrelated to the pointer checked in step 1, weakening semantic equivalence.
  - **Suggested Fix:** Add a ghost source address parameter (arg0) to `copy_from_user` and a spec relation tying the provided `ThreadCreateArgsView` to the data copied from that address. Require `ghost_arg0` to match that source address in `create_thread_model`.

- **Location:** `THREAD_CREATE_ARGS_SIZE()` and `lemma_thread_create_args_size_matches` (spec/proof)
  - **Description:** The size constant is hard-coded to 28 and only documented in a lemma; it is not linked to `core::mem::size_of::<ThreadCreateArgs>()`, so a layout change can silently desynchronize the model from the implementation.
  - **Suggested Fix:** Import a shared constant from the Rust definition or add a build-time/verification-time bridge that asserts equality with `size_of::<ThreadCreateArgs>()`.

- **Location:** `USER_STACK_SIZE()` and `lemma_user_stack_size_matches_config` (spec/proof)
  - **Description:** The stack size is still hard-coded (524288) with no proof linking it to `config::memory_layout::USER_STACK_SIZE`, leaving drift risk.
  - **Suggested Fix:** Tie the spec constant to the config constant via a shared module or add a verification bridge that checks equality.

### Low
- **Location:** API mapping table in `create_thread.rs` module header (exec)
  - **Description:** The mapping still lists `copy_from_user(succeeded, error_code, ghost_pid)` and omits the new `ghost_args_view` parameter, which is now part of the trust boundary.
  - **Suggested Fix:** Update the mapping and trust-boundary description to reflect the current signature.

## Positive Observations
- The previous address-linkage gap is substantially improved: `ThreadCreateArgsView` now carries concrete addresses and `create_thread_model` enforces that the validated addresses match the copied args.
- The exec/spec/proof split remains clean, and the validation pipeline and error propagation are still fully proven.

## Summary
The update strengthens address tracking and fixes the core linkage between validation calls and copied argument addresses, but the model still does not tie the copied args to the `arg0` pointer or to real layout/constants, leaving drift and equivalence gaps. Overall verification quality improved, but it is not yet fully robust against implementation changes.
