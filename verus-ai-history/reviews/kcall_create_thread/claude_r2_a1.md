# Review: kcall_create_thread (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

None.

### High

- **Location:** `is_user_region` / `is_user_addr` external bodies (exec: create_thread.rs:302–323)
  **Description:** The external body functions `is_user_region(valid: bool)` and `is_user_addr(valid: bool)` accept a pre-computed boolean and return it unchanged. This means the verification does not prove that the actual `Vmem::is_user_region(VirtualAddress, usize)` and `Vmem::is_user_addr(VirtualAddress)` calls in the original code are invoked with the correct arguments (i.e., `unsafe_thread_create_args` and `size_of::<ThreadCreateArgs>()` for step 1; `thread_create_args.user_fn` for step 3; etc.). The abstraction gap between "pre-computed boolean input" and "concrete address + size validation" is significant. An implementation bug that passes the wrong address to `is_user_region` would not be caught. The module documentation (lines 117–139) acknowledges this explicitly, but it remains the single largest soundness gap.
  **Suggested Fix:** Introduce spec-level linking lemmas or ghost address parameters (similar to `ghost_pid`) that thread the concrete address/size values through the external bodies, so the spec can assert that the correct address was validated. For example, `is_user_region` could take `Ghost(addr): Ghost<nat>, Ghost(size): Ghost<nat>` and postconditions could relate those to the `CreateThreadInputView.arg0` field.

### Medium

- **Location:** `ThreadCreateArgsModel.user_stack_size` type (exec: create_thread.rs:215)
  **Description:** The original `ThreadCreateArgs.user_stack_size` is `usize` (which is 32-bit on x86-32), and the verified model uses `u32`. While these are equivalent on the target architecture, this creates a silent dependency on the target being 32-bit. If Nanvix ever targets a 64-bit architecture, the model would need updating. The assumption is documented in the trust boundary T5 but not enforced.
  **Suggested Fix:** Add a comment or a compile-time assertion in the model noting this dependency. Alternatively, consider using `usize` in the model if Verus supports it, or add a spec-level assertion `size_of::<usize>() == 4`.

- **Location:** `user_fn_arg0` and `user_fn_arg1` fields omitted (exec: create_thread.rs:207–218)
  **Description:** The original `ThreadCreateArgs` has `user_fn_arg0: usize` and `user_fn_arg1: usize` fields. These are not present in `ThreadCreateArgsModel`. While these fields are not validated in the original `create_thread` function (they are simply passed through to `pm.create_thread`), their omission means the model cannot verify that argument values are faithfully forwarded. The ghost `ThreadCreateArgsView` passed to `pm_create_thread` does not carry these fields either.
  **Suggested Fix:** Consider adding ghost fields for `user_fn_arg0` and `user_fn_arg1` to `ThreadCreateArgsView` to enable future argument-forwarding verification, or document explicitly that argument passthrough is out of scope.

- **Location:** `spec_is_valid_error_code` is overly permissive (spec: create_thread.spec.rs:240–242)
  **Description:** `spec_is_valid_error_code(code: int) -> bool` only checks `code > 0`. While this is intentionally broad (as documented), it means the copy_from_user error propagation path does not constrain the error code to actual `ErrorCode` discriminant values. A buggy `copy_from_user` returning an arbitrary positive integer would satisfy the spec.
  **Suggested Fix:** This is acknowledged in the documentation. For stronger guarantees, the `copy_from_user` postcondition could use `spec_is_error_code_value` (which enumerates known discriminants) instead of `spec_is_valid_error_code`, or at least add a `recommends` clause suggesting the stronger predicate.

### Low

- **Location:** Hardcoded error code `22i32` in exec (exec: create_thread.rs:487, 522, 537, 553, 567)
  **Description:** The exec function uses the literal `22i32` for `InvalidArgument` instead of referencing the `ErrorCode` enum or a named constant. While the proof file (line 255–260) proves `ERROR_CODE_INVALID_ARGUMENT() == ErrorCode::InvalidArgument as int == 22`, the exec code itself hardcodes the numeric value, making it fragile if the `ErrorCode` discriminant were to change.
  **Suggested Fix:** Use `ErrorCode::InvalidArgument as i32` in the exec code instead of `22i32`, or define a verified `const` for this. This would make the exec code self-documenting and reduce the risk of mismatch.

- **Location:** Dummy PM outcome in error paths (exec: create_thread.rs:481, 497, 516, 531, 545, 561)
  **Description:** Each early-return error path constructs a dummy `CreateThreadOutcomeView::CtOk { tid: 0 }` for the ghost PM outcome. While this is correct (the short-circuit lemma proves PM outcome is irrelevant on validation failure), it would be cleaner to use a dedicated sentinel or `CtError` variant to avoid confusion during code review.
  **Suggested Fix:** Consider using `CreateThreadOutcomeView::CtError { error_code: 0 }` as the dummy, or define a `spec fn IRRELEVANT_PM_OUTCOME() -> CreateThreadOutcomeView` to make intent explicit.

- **Location:** `user_stack_base` address not threaded through ghost (exec/spec)
  **Description:** The original validates `is_user_region(thread_create_args.user_stack_base, thread_create_args.user_stack_size)`. The model captures `user_stack_valid` as a boolean but does not ghost-track the `user_stack_base` address. Similarly `user_fn` address is not ghost-tracked. Only `pid` and `arg0` have ghost identity tracking.
  **Suggested Fix:** Add ghost fields for `user_fn_addr`, `user_stack_base_addr`, and `user_tda_addr` to `CreateThreadInputView` for completeness, paralleling the `arg0` ghost tracking.

## Positive Observations

- **Comprehensive pipeline modeling:** The spec faithfully captures all six validation steps of the original `create_thread` function in the correct order with proper short-circuit semantics. The `spec_create_thread_result` function is a clean sequential cascade matching the original control flow.

- **Rich proof coverage:** 14 proof lemmas cover error propagation for each validation step, short-circuit behavior, result exhaustiveness, success-implies-all-valid bidirectional proof, error code domain, and TID validity. The `lemma_success_requires_all_steps` biconditional (`<==>`) is particularly strong.

- **Excellent documentation:** The module-level documentation is exceptionally thorough, with clear enumeration of verified properties, out-of-scope properties, trust boundaries, abstraction correctness discussion, and an API mapping table. This is well above average for verification documentation.

- **Clean separation of concerns:** Spec, proof, and exec are cleanly separated into three files. The spec contains only `open spec fn` definitions and view types. The proof file contains only `proof fn` lemmas. The exec file contains the model and external bodies. This follows good Verus practice.

- **Verification passes cleanly:** All 18 verification conditions pass with no errors, confirming the proofs are mechanically checked.

- **Ghost identity tracking for PID:** The `ghost_pid` parameter is threaded through both `copy_from_user` and `pm_create_thread`, proving the same PID from `KcallArgs` is used consistently across both operations.

- **Trust boundaries are explicitly documented:** Each `external_body` function has a numbered trust boundary (T1–T6) with a clear description of what is trusted and why. The abstraction correctness section (lines 114–139) is honest about the limitations of the boolean abstraction.

## Summary

The verification of `kcall_create_thread` is well-executed and demonstrates strong coverage of the validation pipeline's control-flow logic. All 6 steps of the original function are modeled, and 14 proof lemmas establish key properties including error propagation, short-circuit behavior, result exhaustiveness, and the bidirectional success condition. The documentation is exemplary.

The primary weakness is the boolean abstraction for memory validation functions (`is_user_region`, `is_user_addr`), which means the verification proves "if the booleans are correct, the dispatch logic is correct" rather than "the actual address validation is correct." This is acknowledged and is a reasonable per-module design choice, but it means full end-to-end soundness requires composing with VMM module proofs. Adding ghost address parameters to the external bodies would close the largest remaining gap. The hardcoded `22i32` literals and the omission of `user_fn_arg0`/`user_fn_arg1` from the model are minor issues that could improve maintainability and completeness.
