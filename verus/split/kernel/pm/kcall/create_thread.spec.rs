// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Create Thread Kernel Call Specification.
// Defines View types, spec constants, and spec functions for the create_thread
// kernel call verification model.
//
// ## Verification Model
//
// The create_thread kcall implements a multi-step validation pipeline:
//   1. Validate thread_create_args pointer lies in user space.
//   2. Copy thread_create_args from user space to kernel space.
//   3. Validate user_fn lies in user address space.
//   4. Validate user_stack lies in user address space with sufficient size.
//   5. Validate user_tda (if present) lies in user address space.
//   6. Call ProcessManager::create_thread(mm, pid, &thread_create_args).
//
// This spec models:
// - Each validation step as a boolean predicate.
// - The copy_from_user operation as a fallible step with error propagation.
// - The overall result as a sequential composition with short-circuit on error.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Spec Constants
//==================================================================================================

/// The ErrorCode value for InvalidArgument (repr(i32) = 22).
/// Used when any validation step fails.
pub open spec fn ERROR_CODE_INVALID_ARGUMENT() -> int {
    22
}

/// The minimum user stack size in bytes (512 KiB = 524288).
/// Corresponds to `config::memory_layout::USER_STACK_SIZE`.
pub open spec fn USER_STACK_SIZE() -> nat {
    524288
}

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of the thread creation arguments after copy_from_user.
///
/// # Description
///
/// Models the ThreadCreateArgs structure after it has been copied from
/// user space. Contains the validated fields needed for thread creation.
#[verifier::ext_equal]
pub struct ThreadCreateArgsView {
    /// Whether user_fn address is in user address space.
    pub user_fn_valid: bool,
    /// Whether user_stack region is in user address space.
    pub user_stack_valid: bool,
    /// The user stack size in bytes.
    pub user_stack_size: nat,
    /// Whether user_tda is present (Some).
    pub has_user_tda: bool,
    /// Whether user_tda address (if present) is in user address space.
    pub user_tda_valid: bool,
}

/// Abstract view of the create_thread validation pipeline input.
///
/// # Description
///
/// Models the inputs to the create_thread kcall that determine
/// the validation pipeline's behavior.
#[verifier::ext_equal]
pub struct CreateThreadInputView {
    /// The process identifier from KcallArgs (ghost-tracked).
    pub pid: nat,
    /// The raw arg0 value from KcallArgs (ghost-tracked address).
    pub arg0: nat,
    /// Whether the thread_create_args pointer lies in user space.
    pub args_addr_valid: bool,
    /// Whether copy_from_user succeeded.
    pub copy_succeeded: bool,
    /// The error code from copy_from_user failure (if any).
    pub copy_error_code: int,
    /// The thread creation arguments (valid only if copy succeeded).
    pub thread_args: ThreadCreateArgsView,
}

/// Abstract view of the ProcessManager::create_thread outcome.
///
/// # Description
///
/// Models the result of `pm.create_thread(mm, pid, &thread_create_args)`:
/// - `CtOk`: Thread was created successfully; carries the TID as i32.
/// - `CtError`: Thread creation failed; carries the error code.
#[verifier::ext_equal]
pub enum CreateThreadOutcomeView {
    /// ProcessManager::create_thread returned Ok(tid).
    CtOk { tid: int },
    /// ProcessManager::create_thread returned Err(error).
    CtError { error_code: int },
}

/// Abstract view of the create_thread kcall's final result.
///
/// # Description
///
/// The create_thread function returns KcallResult, which is either:
/// - Success: All validations passed and PM created the thread.
/// - Error: A validation step failed or pm.create_thread failed.
#[verifier::ext_equal]
pub enum CreateThreadResultView {
    /// Thread was created successfully; carries the TID value.
    Success { tid_value: int },
    /// An error occurred at some validation or creation step.
    Error { error_code: int },
}

//==================================================================================================
// Spec Predicates
//==================================================================================================

/// Whether the thread_create_args address is valid (lies in user space).
pub open spec fn spec_args_addr_valid(input: CreateThreadInputView) -> bool {
    input.args_addr_valid
}

/// Whether the copy_from_user operation succeeded.
pub open spec fn spec_copy_succeeded(input: CreateThreadInputView) -> bool {
    input.copy_succeeded
}

/// Whether the user function address is valid.
pub open spec fn spec_user_fn_valid(input: CreateThreadInputView) -> bool {
    input.thread_args.user_fn_valid
}

/// Whether the user stack region is valid (in user space and large enough).
pub open spec fn spec_user_stack_valid(input: CreateThreadInputView) -> bool {
    input.thread_args.user_stack_valid && input.thread_args.user_stack_size >= USER_STACK_SIZE()
}

/// Whether the user TDA is valid (either absent or in user space).
pub open spec fn spec_user_tda_valid(input: CreateThreadInputView) -> bool {
    if input.thread_args.has_user_tda {
        input.thread_args.user_tda_valid
    } else {
        true
    }
}

/// Whether all validation steps passed (precondition for PM call).
///
/// # Description
///
/// All six validation steps must pass before the PM create_thread call:
/// 1. Args address in user space.
/// 2. Copy from user succeeded.
/// 3. User function in user address space.
/// 4. User stack in user address space with sufficient size.
/// 5. User TDA (if present) in user address space.
pub open spec fn spec_all_validations_passed(input: CreateThreadInputView) -> bool {
    spec_args_addr_valid(input)
    && spec_copy_succeeded(input)
    && spec_user_fn_valid(input)
    && spec_user_stack_valid(input)
    && spec_user_tda_valid(input)
}

//==================================================================================================
// Spec Functions
//==================================================================================================

/// Spec function: models the complete create_thread kcall pipeline.
///
/// # Description
///
/// The create_thread function executes a sequential validation pipeline:
/// 1. Check args address → InvalidArgument on failure.
/// 2. Copy from user → propagate error code on failure.
/// 3. Check user_fn → InvalidArgument on failure.
/// 4. Check user_stack region and size → InvalidArgument on failure.
/// 5. Check user_tda (if present) → InvalidArgument on failure.
/// 6. Call pm.create_thread → Success with TID or Error.
///
/// Each step only executes if all previous steps succeeded (short-circuit).
///
/// The `copy_error_code` is constrained to be a valid positive error code
/// by the `copy_from_user` external body postcondition, ensuring the
/// propagated error code is always valid.
pub open spec fn spec_create_thread_result(
    input: CreateThreadInputView,
    pm_outcome: CreateThreadOutcomeView,
) -> CreateThreadResultView
    recommends
        !input.copy_succeeded ==> spec_is_valid_error_code(input.copy_error_code),
{
    if !spec_args_addr_valid(input) {
        CreateThreadResultView::Error { error_code: ERROR_CODE_INVALID_ARGUMENT() }
    } else if !spec_copy_succeeded(input) {
        CreateThreadResultView::Error { error_code: input.copy_error_code }
    } else if !spec_user_fn_valid(input) {
        CreateThreadResultView::Error { error_code: ERROR_CODE_INVALID_ARGUMENT() }
    } else if !spec_user_stack_valid(input) {
        CreateThreadResultView::Error { error_code: ERROR_CODE_INVALID_ARGUMENT() }
    } else if !spec_user_tda_valid(input) {
        CreateThreadResultView::Error { error_code: ERROR_CODE_INVALID_ARGUMENT() }
    } else {
        match pm_outcome {
            CreateThreadOutcomeView::CtOk { tid } => {
                CreateThreadResultView::Success { tid_value: tid }
            },
            CreateThreadOutcomeView::CtError { error_code } => {
                CreateThreadResultView::Error { error_code }
            },
        }
    }
}

/// Spec function: whether the result is success.
pub open spec fn spec_is_success(result: CreateThreadResultView) -> bool {
    matches!(result, CreateThreadResultView::Success { .. })
}

/// Spec function: whether the result is an error.
pub open spec fn spec_is_error(result: CreateThreadResultView) -> bool {
    matches!(result, CreateThreadResultView::Error { .. })
}

/// Spec function: whether the PM create_thread operation succeeded.
pub open spec fn spec_pm_create_thread_ok(outcome: CreateThreadOutcomeView) -> bool {
    matches!(outcome, CreateThreadOutcomeView::CtOk { .. })
}

/// Spec predicate: whether an error code is a valid positive error code.
pub open spec fn spec_is_valid_error_code(code: int) -> bool {
    code > 0
}

/// Spec predicate: whether an error code matches the ErrorCode enum domain.
///
/// # Description
///
/// Enumerates the actual discriminant values of the `ErrorCode` enum
/// (repr(i32)): NoSuchEntry=2, NoSuchProcess=3, OutOfMemory=12,
/// BadAddress=14, ResourceBusy=16, InvalidArgument=22.
pub open spec fn spec_is_error_code_value(code: int) -> bool {
    code == 2    // NoSuchEntry
    || code == 3    // NoSuchProcess
    || code == 12   // OutOfMemory
    || code == 14   // BadAddress
    || code == 16   // ResourceBusy
    || code == 22   // InvalidArgument
}

} // verus!
