// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Create Thread Kernel Call Proofs.
// Proof lemmas for the create_thread kcall verification model.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Proof Functions — Validation Error Propagation
//==================================================================================================

/// Proof: when args address is invalid, the result is InvalidArgument
/// regardless of all other inputs.
///
/// # Description
///
/// The first validation step checks whether thread_create_args lies in user
/// space. If it fails, the function returns immediately with InvalidArgument.
/// No subsequent steps are executed.
pub proof fn lemma_args_addr_invalid_propagates(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    requires
        !spec_args_addr_valid(input),
    ensures
        spec_create_thread_result(input, pm_outcome) ==
            (CreateThreadResultView::Error { error_code: ERROR_CODE_INVALID_ARGUMENT() }),
        spec_is_error(spec_create_thread_result(input, pm_outcome)),
{
}

/// Proof: when copy_from_user fails, its error code propagates.
///
/// # Description
///
/// If the args address is valid but copy_from_user fails, the error code
/// from the copy operation is returned. No further validation occurs.
pub proof fn lemma_copy_error_propagates(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    requires
        spec_args_addr_valid(input),
        !spec_copy_succeeded(input),
    ensures
        spec_create_thread_result(input, pm_outcome) ==
            (CreateThreadResultView::Error { error_code: input.copy_error_code }),
        spec_is_error(spec_create_thread_result(input, pm_outcome)),
{
}

/// Proof: when user_fn is invalid, the result is InvalidArgument.
///
/// # Description
///
/// If args address and copy succeed but user_fn validation fails,
/// the function returns InvalidArgument.
pub proof fn lemma_user_fn_invalid_propagates(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    requires
        spec_args_addr_valid(input),
        spec_copy_succeeded(input),
        !spec_user_fn_valid(input),
    ensures
        spec_create_thread_result(input, pm_outcome) ==
            (CreateThreadResultView::Error { error_code: ERROR_CODE_INVALID_ARGUMENT() }),
        spec_is_error(spec_create_thread_result(input, pm_outcome)),
{
}

/// Proof: when user_stack is invalid, the result is InvalidArgument.
///
/// # Description
///
/// If prior validations pass but user_stack validation fails (either
/// out of user space or too small), the function returns InvalidArgument.
pub proof fn lemma_user_stack_invalid_propagates(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    requires
        spec_args_addr_valid(input),
        spec_copy_succeeded(input),
        spec_user_fn_valid(input),
        !spec_user_stack_valid(input),
    ensures
        spec_create_thread_result(input, pm_outcome) ==
            (CreateThreadResultView::Error { error_code: ERROR_CODE_INVALID_ARGUMENT() }),
        spec_is_error(spec_create_thread_result(input, pm_outcome)),
{
}

/// Proof: when user_tda is invalid, the result is InvalidArgument.
///
/// # Description
///
/// If prior validations pass but user_tda is present and invalid,
/// the function returns InvalidArgument.
pub proof fn lemma_user_tda_invalid_propagates(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    requires
        spec_args_addr_valid(input),
        spec_copy_succeeded(input),
        spec_user_fn_valid(input),
        spec_user_stack_valid(input),
        !spec_user_tda_valid(input),
    ensures
        spec_create_thread_result(input, pm_outcome) ==
            (CreateThreadResultView::Error { error_code: ERROR_CODE_INVALID_ARGUMENT() }),
        spec_is_error(spec_create_thread_result(input, pm_outcome)),
{
}

/// Proof: validation error at any step produces InvalidArgument
/// (except copy_from_user which preserves its own error code).
///
/// # Description
///
/// Generalizes all validation error paths: if any validation fails,
/// the result is an error.
pub proof fn lemma_validation_failure_is_error(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    requires
        !spec_all_validations_passed(input),
    ensures
        spec_is_error(spec_create_thread_result(input, pm_outcome)),
{
}

//==================================================================================================
// Proof Functions — PM Error Propagation
//==================================================================================================

/// Proof: when all validations pass but PM create_thread fails,
/// the PM error code propagates.
///
/// # Description
///
/// If all five validation steps pass and PM create_thread returns an error,
/// that error code is faithfully propagated as the final result.
pub proof fn lemma_pm_error_propagates(
    input: CreateThreadInputView,
    error_code: int,
)
    requires
        spec_all_validations_passed(input),
    ensures
        spec_create_thread_result(
            input,
            CreateThreadOutcomeView::CtError { error_code },
        ) == (CreateThreadResultView::Error { error_code }),
        spec_is_error(spec_create_thread_result(
            input,
            CreateThreadOutcomeView::CtError { error_code },
        )),
{
}

//==================================================================================================
// Proof Functions — Success / Exhaustiveness
//==================================================================================================

/// Proof: success requires all validations to pass and PM to succeed.
///
/// # Description
///
/// The result is Success if and only if:
/// 1. All five validation steps passed.
/// 2. PM create_thread succeeded (returned Ok(tid)).
pub proof fn lemma_success_requires_all_steps(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    ensures
        spec_is_success(spec_create_thread_result(input, pm_outcome))
            <==> (
                spec_all_validations_passed(input)
                && spec_pm_create_thread_ok(pm_outcome)
            ),
{
    if spec_all_validations_passed(input) {
        match pm_outcome {
            CreateThreadOutcomeView::CtOk { .. } => {},
            CreateThreadOutcomeView::CtError { .. } => {},
        }
    }
}

/// Proof: the result is always exactly Success or Error (exhaustive and exclusive).
///
/// # Description
///
/// Every possible input combination produces exactly one result category.
/// Success and Error are mutually exclusive.
pub proof fn lemma_result_exhaustive(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    ensures
        ({
            let result: CreateThreadResultView = spec_create_thread_result(input, pm_outcome);
            spec_is_success(result) || spec_is_error(result)
        }),
        ({
            let result: CreateThreadResultView = spec_create_thread_result(input, pm_outcome);
            !(spec_is_success(result) && spec_is_error(result))
        }),
{
    if spec_all_validations_passed(input) {
        match pm_outcome {
            CreateThreadOutcomeView::CtOk { .. } => {},
            CreateThreadOutcomeView::CtError { .. } => {},
        }
    }
}

/// Proof: success implies valid TID in the result.
///
/// # Description
///
/// If the overall result is Success, then it carries a TID value that
/// matches the PM create_thread outcome's TID.
pub proof fn lemma_success_implies_valid_tid(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    requires
        spec_is_success(spec_create_thread_result(input, pm_outcome)),
    ensures
        spec_all_validations_passed(input),
        spec_pm_create_thread_ok(pm_outcome),
        pm_outcome matches CreateThreadOutcomeView::CtOk { tid }
            ==> spec_create_thread_result(input, pm_outcome)
                == (CreateThreadResultView::Success { tid_value: tid }),
{
    // Follows from spec_create_thread_result definition.
}

//==================================================================================================
// Proof Functions — Error Code Preservation
//==================================================================================================

/// Proof: the spec constant ERROR_CODE_INVALID_ARGUMENT matches
/// ErrorCode::InvalidArgument.
pub proof fn lemma_error_code_matches()
    ensures
        ERROR_CODE_INVALID_ARGUMENT() == ErrorCode::InvalidArgument as int,
{
    assert(ErrorCode::InvalidArgument as int == 22int);
}

/// Proof: validation failures always produce InvalidArgument (except copy_from_user).
///
/// # Description
///
/// Steps 1, 3, 4, 5 all return InvalidArgument on failure. Step 2
/// (copy_from_user) returns its own error code.
pub proof fn lemma_validation_error_codes(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    requires
        !spec_all_validations_passed(input),
    ensures
        // If copy succeeded (or wasn't reached), error is InvalidArgument.
        (spec_args_addr_valid(input) && spec_copy_succeeded(input))
            ==> (spec_create_thread_result(input, pm_outcome)
                matches CreateThreadResultView::Error { error_code }
                && error_code == ERROR_CODE_INVALID_ARGUMENT()),
        // If copy failed, error is the copy error code.
        (spec_args_addr_valid(input) && !spec_copy_succeeded(input))
            ==> (spec_create_thread_result(input, pm_outcome)
                matches CreateThreadResultView::Error { error_code }
                && error_code == input.copy_error_code),
        // If args addr invalid, error is InvalidArgument.
        !spec_args_addr_valid(input)
            ==> (spec_create_thread_result(input, pm_outcome)
                matches CreateThreadResultView::Error { error_code }
                && error_code == ERROR_CODE_INVALID_ARGUMENT()),
{
}

//==================================================================================================
// Proof Functions — Short-Circuit Behavior
//==================================================================================================

/// Proof: early validation failures make the PM outcome irrelevant.
///
/// # Description
///
/// When any validation step fails, the PM outcome does not affect the
/// final result. This proves the short-circuit behavior.
pub proof fn lemma_short_circuit_on_validation_failure(
    input: CreateThreadInputView,
    pm1: CreateThreadOutcomeView,
    pm2: CreateThreadOutcomeView,
)
    requires
        !spec_all_validations_passed(input),
    ensures
        spec_create_thread_result(input, pm1) == spec_create_thread_result(input, pm2),
{
}

/// Proof: absent user_tda always passes validation step 5.
///
/// # Description
///
/// When user_tda is None (has_user_tda == false), the TDA validation
/// step is automatically satisfied regardless of user_tda_valid.
pub proof fn lemma_absent_tda_always_valid(
    input: CreateThreadInputView,
)
    requires
        !input.thread_args.has_user_tda,
    ensures
        spec_user_tda_valid(input),
{
}

/// Proof: copy error codes are always valid positive error codes.
///
/// # Description
///
/// When copy_from_user fails and the error code satisfies
/// `spec_is_valid_error_code`, the propagated error code in the
/// final result is also valid. This links the copy_from_user
/// external body postcondition to the pipeline result.
pub proof fn lemma_copy_error_code_valid(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
)
    requires
        spec_args_addr_valid(input),
        !spec_copy_succeeded(input),
        spec_is_valid_error_code(input.copy_error_code),
    ensures
        spec_create_thread_result(input, pm_outcome) ==
            (CreateThreadResultView::Error { error_code: input.copy_error_code }),
        spec_is_valid_error_code(input.copy_error_code),
{
}

//==================================================================================================
// Proof Functions — Error Code Domain
//==================================================================================================

/// Proof: error codes in the verified ErrorCode subset are valid positive codes.
///
/// # Description
///
/// Links `spec_is_error_code_value` (the verified ErrorCode subset) to
/// `spec_is_valid_error_code` (code > 0). All ErrorCode values in the
/// subset (2, 3, 12, 14, 16, 22) are positive. This lemma enables
/// module-level proofs to strengthen from the broad `code > 0` constraint
/// to specific ErrorCode values when the call site's error codes are known.
pub proof fn lemma_error_code_value_implies_valid(code: int)
    requires
        spec_is_error_code_value(code),
    ensures
        spec_is_valid_error_code(code),
{
}

/// Proof: the USER_STACK_SIZE spec constant matches config::memory_layout::USER_STACK_SIZE.
///
/// # Description
///
/// Documents that `USER_STACK_SIZE() == 524288` (512 * 1024 bytes).
/// Source: `src/libs/config/src/lib.rs` line 120:
///   `pub const USER_STACK_SIZE: usize = 512 * crate::constants::KILOBYTE;`
/// If the kernel constant changes, this lemma will need updating.
pub proof fn lemma_user_stack_size_matches_config()
    ensures
        USER_STACK_SIZE() == 524288nat,
{
}

/// Proof: the THREAD_CREATE_ARGS_SIZE spec constant matches ThreadCreateArgs layout.
///
/// # Description
///
/// Documents that `THREAD_CREATE_ARGS_SIZE() == 28` bytes on x86-32.
/// Layout: VirtualAddress(usize=4) + usize(4) + usize(4)
///       + VirtualAddress(4) + usize(4) + Option<VirtualAddress>(8)
///       = 28 bytes.
///
/// Source: `src/libs/sys/src/sys/pm/thread_create_args.rs`.
/// If `ThreadCreateArgs` fields change, this lemma will need updating.
pub proof fn lemma_thread_create_args_size_matches()
    ensures
        THREAD_CREATE_ARGS_SIZE() == 28nat,
{
}

} // verus!
