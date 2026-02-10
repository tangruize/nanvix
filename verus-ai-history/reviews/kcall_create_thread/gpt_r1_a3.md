# Review: kcall_create_thread (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Location:** `CreateThreadInputView.arg0` / `spec_args_addr_valid` / `is_user_region` (spec/exec)
  - **Description:** The model still treats `args_addr_valid` as an unconstrained boolean. There is no spec-level linkage between `arg0`, the `VirtualAddress::from_raw_value(args.arg0 as usize)` cast, `size_of::<ThreadCreateArgs>()`, and the boolean passed to `is_user_region`. The module notes this in comments, but the correctness of the pointer/size check remains unverified, so semantic equivalence is still incomplete.
  - **Suggested Fix:** Encode concrete address/size in the view (or pass `ghost_arg0` and a size ghost into `is_user_region`/`copy_from_user`) and add a linking lemma to relate the boolean predicate to the actual address/size computation.

### Low
- **Location:** `USER_STACK_SIZE()` and `lemma_user_stack_size_matches_config` (spec/proof)
  - **Description:** The constant is still hardcoded with a manual lemma and comments pointing to the kernel config. This remains a maintenance hazard: if the kernel constant changes, the proof can drift without an enforced link.
  - **Suggested Fix:** Add an automated linkage (codegen or CI check) that fails verification when the kernel constant changes, or import a verified constant shared with the kernel config.

## Positive Observations
- The previous over-constraint on error codes has been removed from external body postconditions; they now only require `code > 0`, which matches the original behavior.
- PID and arg0 are ghost-tracked consistently, and the stack-size check remains a concrete numeric comparison.
- The spec/proof/exec split remains clean and the documented trust boundaries are explicit.

## Summary
The earlier error-code domain issue is fixed, but the model still does not link `arg0` to the actual address/size validation logic, leaving a key semantic gap. The stack-size constant remains manually synchronized. Overall, verification is improved but not yet fully sound for pointer validation.
