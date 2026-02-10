// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Create Thread Kernel Call Verification Model
//!
//! Formal verification of the create_thread kernel call (`pm::kcall::create_thread`).
//!
//! ## Overview
//!
//! The `create_thread(pm, mm, args)` function creates a new thread in the calling
//! process. It implements a multi-step validation pipeline:
//! 1. Validate `thread_create_args` pointer lies in user space (`is_user_region`).
//! 2. Copy `ThreadCreateArgs` from user space to kernel space (`copy_from_user`).
//! 3. Validate `user_fn` address lies in user address space (`is_user_addr`).
//! 4. Validate `user_stack` lies in user address space with sufficient size.
//! 5. Validate `user_tda` (if present) lies in user address space.
//! 6. Call `ProcessManager::create_thread(mm, pid, &thread_create_args)`.
//!
//! On success, returns `KcallResult::Success(tid)`. On failure at any step,
//! returns `KcallResult::Error(error_code)`.
//!
//! ## Verified Properties
//!
//! - **Args address validation**: When `thread_create_args` does not lie in user
//!   space, the function returns `InvalidArgument` immediately
//!   (`lemma_args_addr_invalid_propagates`).
//! - **Copy error propagation**: When `copy_from_user` fails, the copy error
//!   code is returned; no further validation occurs
//!   (`lemma_copy_error_propagates`).
//! - **User function validation**: When `user_fn` is not in user address space,
//!   the function returns `InvalidArgument`
//!   (`lemma_user_fn_invalid_propagates`).
//! - **User stack validation**: When the user stack is not in user address space
//!   or is too small, the function returns `InvalidArgument`
//!   (`lemma_user_stack_invalid_propagates`).
//! - **User TDA validation**: When `user_tda` is present and not in user address
//!   space, the function returns `InvalidArgument`
//!   (`lemma_user_tda_invalid_propagates`).
//! - **Absent TDA always valid**: When `user_tda` is `None`, the TDA validation
//!   step is automatically satisfied (`lemma_absent_tda_always_valid`).
//! - **PM error propagation**: When all validations pass but
//!   `pm.create_thread` fails, the PM error code is returned
//!   (`lemma_pm_error_propagates`).
//! - **Success requires all steps**: The result is Success if and only if all
//!   five validation steps pass and `pm.create_thread` succeeds
//!   (`lemma_success_requires_all_steps`).
//! - **Result exhaustiveness**: Every input combination produces exactly one
//!   result: Success or Error. These are mutually exclusive
//!   (`lemma_result_exhaustive`).
//! - **Error code preservation**: Validation failures return InvalidArgument
//!   (except copy_from_user which preserves its own code);
//!   PM errors are preserved faithfully
//!   (`lemma_validation_error_codes`, `lemma_pm_error_propagates`).
//! - **Short-circuit behavior**: When any validation fails, the PM outcome
//!   is irrelevant (`lemma_short_circuit_on_validation_failure`).
//! - **Success implies valid TID**: If the result is Success, it carries
//!   a TID matching the PM outcome (`lemma_success_implies_valid_tid`).
//! - **Error code linkage**: `ERROR_CODE_INVALID_ARGUMENT()` is proven equal
//!   to `ErrorCode::InvalidArgument as int` (`lemma_error_code_matches`).
//!
//! ## Properties NOT Proven Here (Out of Scope)
//!
//! - "The created thread is schedulable" — thread scheduling invariant.
//! - "The thread's stack is properly mapped in virtual memory" — VMM invariant.
//! - "The thread's TID is unique" — TID allocator invariant.
//! - "Resources are cleaned up on thread creation failure" — resource management.
//!
//! These are internal PM and VMM invariants verified in their respective modules.
//!
//! ## Trust Boundaries
//!
//! - **T1: `Vmem::is_user_region(addr, size)`**. Checks whether a memory region
//!   lies within user address space. Modeled as `external_body` returning a bool.
//!   The VMM module verifies this implementation.
//! - **T2: `Vmem::is_user_addr(addr)`**. Checks whether an address lies within
//!   user address space. Modeled as `external_body` returning a bool.
//! - **T3: `pm::copy_from_user(pm, pid, dst, src)`**. Copies data from user space
//!   to kernel space. Modeled as `external_body` returning a fallible result.
//! - **T4: `ProcessManager::create_thread(mm, pid, args)`**. Creates a thread
//!   in the PM. Modeled as `external_body` returning Ok(tid) or Err(error).
//!
//! ## API Mapping
//!
//! | Original API                              | Verified Model                         | Notes           |
//! |-------------------------------------------|----------------------------------------|-----------------|
//! | `Vmem::is_user_region(addr, size)`        | `is_user_region(addr_valid)`           | external_body   |
//! | `Vmem::is_user_addr(addr)`                | `is_user_addr(addr_valid)`             | external_body   |
//! | `pm::copy_from_user(pm, pid, dst, src)`   | `copy_from_user(succeeded, error_code)`| external_body   |
//! | `pm.create_thread(mm, pid, args)`         | `pm_create_thread(…)`                  | external_body   |
//! | `pub fn create_thread(pm, mm, args)`      | `create_thread_model(input, …)`        | Fully verified  |
//!
//! Parameter abstraction: `pm: &mut ProcessManager` and `mm: &mut VirtMemoryManager`
//! are replaced by ghost state, and `args: &KcallArgs` is replaced by
//! `CreateThreadInputView`. This simplification focuses on the validation
//! dispatch logic.

use crate::libs::error::ErrorCode;
use vstd::prelude::*;

// Include specifications.
include!("create_thread.spec.rs");

// Include proofs.
include!("create_thread.proof.rs");

verus! {

//==================================================================================================
// Dependency Models (External Bodies)
//==================================================================================================

/// Model of the copy_from_user result for verification.
///
/// # Description
///
/// Represents the two possible outcomes from `pm::copy_from_user`:
/// - `CopyOk`: The copy succeeded; thread_create_args is now in kernel space.
/// - `CopyError`: The copy failed; carries the error code.
pub enum CopyFromUserResultModel {
    /// copy_from_user succeeded.
    CopyOk,
    /// copy_from_user failed with an error code.
    CopyError { error_code: i32 },
}

impl CopyFromUserResultModel {
    /// Spec function: whether the copy succeeded.
    pub open spec fn spec_succeeded(&self) -> bool {
        matches!(self, CopyFromUserResultModel::CopyOk)
    }

    /// Spec function: extract error code (meaningful only on failure).
    pub open spec fn spec_error_code(&self) -> int {
        match self {
            CopyFromUserResultModel::CopyOk => 0int,
            CopyFromUserResultModel::CopyError { error_code } => *error_code as int,
        }
    }
}

/// Model of the ThreadCreateArgs validation results.
///
/// # Description
///
/// After copy_from_user succeeds, the thread_create_args fields are
/// validated individually. This struct captures the validation results.
pub struct ThreadCreateArgsModel {
    /// Whether user_fn lies in user address space.
    pub user_fn_valid: bool,
    /// Whether user_stack region lies in user address space.
    pub user_stack_valid: bool,
    /// Whether user_stack_size >= USER_STACK_SIZE.
    pub user_stack_size_valid: bool,
    /// Whether user_tda is present.
    pub has_user_tda: bool,
    /// Whether user_tda (if present) lies in user address space.
    pub user_tda_valid: bool,
}

impl ThreadCreateArgsModel {
    /// Spec function: converts to the abstract ThreadCreateArgsView.
    pub open spec fn spec_view(&self) -> ThreadCreateArgsView {
        ThreadCreateArgsView {
            user_fn_valid: self.user_fn_valid,
            user_stack_valid: self.user_stack_valid,
            user_stack_size_valid: self.user_stack_size_valid,
            has_user_tda: self.has_user_tda,
            user_tda_valid: self.user_tda_valid,
        }
    }
}

/// Model of the ProcessManager::create_thread result.
///
/// # Description
///
/// Represents the two possible outcomes from `pm.create_thread(mm, pid, args)`:
/// - `CtOk`: Thread was created; carries the TID as i32.
/// - `CtError`: Thread creation failed; carries the error code.
pub enum CreateThreadResultModel {
    /// pm.create_thread returned Ok(tid).
    CtOk { tid: i32 },
    /// pm.create_thread returned Err(error).
    CtError { error_code: i32 },
}

impl CreateThreadResultModel {
    /// Spec function: converts to the abstract CreateThreadOutcomeView.
    pub open spec fn spec_view(&self) -> CreateThreadOutcomeView {
        match self {
            CreateThreadResultModel::CtOk { tid } => {
                CreateThreadOutcomeView::CtOk { tid: *tid as int }
            },
            CreateThreadResultModel::CtError { error_code } => {
                CreateThreadOutcomeView::CtError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the KcallResult for verification.
///
/// # Description
///
/// Represents the two possible outcomes from the kcall:
/// - `Success`: Thread was created; carries the TID value.
/// - `Error`: An error occurred; carries the error code.
pub enum KcallResultModel {
    /// KcallResult::Success — thread created with TID value.
    Success { tid_value: i32 },
    /// KcallResult::Error — error with code.
    Error { error_code: i32 },
}

impl KcallResultModel {
    /// Spec function: converts to the abstract CreateThreadResultView.
    pub open spec fn spec_view(&self) -> CreateThreadResultView {
        match self {
            KcallResultModel::Success { tid_value } => {
                CreateThreadResultView::Success { tid_value: *tid_value as int }
            },
            KcallResultModel::Error { error_code } => {
                CreateThreadResultView::Error { error_code: *error_code as int }
            },
        }
    }
}

//==================================================================================================
// External Body Functions (Trust Boundaries)
//==================================================================================================

/// Trust Boundary T1: Models `Vmem::is_user_region(addr, size)`.
///
/// # Description
///
/// Checks whether a memory region starting at `addr` with given `size`
/// lies entirely within user address space. The result is deterministic
/// for a given (addr, size) pair.
#[verifier::external_body]
pub fn is_user_region(valid: bool) -> (result: bool)
    ensures
        result == valid,
{
    unimplemented!()
}

/// Trust Boundary T2: Models `Vmem::is_user_addr(addr)`.
///
/// # Description
///
/// Checks whether an address lies within user address space.
#[verifier::external_body]
pub fn is_user_addr(valid: bool) -> (result: bool)
    ensures
        result == valid,
{
    unimplemented!()
}

/// Trust Boundary T3: Models `pm::copy_from_user(pm, pid, dst, src)`.
///
/// # Description
///
/// Copies data from user space to kernel space. Returns Ok on success or
/// Err with an error code on failure. The `succeeded` and `error_code`
/// parameters model the outcome deterministically.
#[verifier::external_body]
pub fn copy_from_user(succeeded: bool, error_code: i32) -> (result: CopyFromUserResultModel)
    ensures
        succeeded ==> matches!(result, CopyFromUserResultModel::CopyOk),
        !succeeded ==> (result matches CopyFromUserResultModel::CopyError { error_code: ec }
            && ec == error_code),
        result.spec_succeeded() == succeeded,
        !succeeded ==> result.spec_error_code() == error_code as int,
{
    unimplemented!()
}

/// Trust Boundary T4: Models `ProcessManager::create_thread(mm, pid, args)`.
///
/// # Description
///
/// Creates a new thread in the process identified by `pid`. Returns
/// Ok(tid) on success or Err(error) on failure.
///
/// Postconditions capture:
/// - The result is always one of the defined variants.
/// - On success, the TID is a valid positive value.
/// - On failure, the error code is a valid positive value.
#[verifier::external_body]
pub fn pm_create_thread() -> (result: CreateThreadResultModel)
    ensures
        matches!(result, CreateThreadResultModel::CtOk { .. } | CreateThreadResultModel::CtError { .. }),
        result matches CreateThreadResultModel::CtOk { tid } ==> tid >= 0i32,
        result.spec_view() matches CreateThreadOutcomeView::CtError { error_code }
            ==> spec_is_valid_error_code(error_code),
{
    unimplemented!()
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Verified exec model of the `create_thread(pm, mm, args)` kernel call.
///
/// # Description
///
/// This function mirrors the original `create_thread` control flow:
/// 1. Check if thread_create_args lies in user space → InvalidArgument on failure.
/// 2. Copy thread_create_args from user space → propagate error on failure.
/// 3. Check user_fn lies in user address space → InvalidArgument on failure.
/// 4. Check user_stack lies in user address space with sufficient size → InvalidArgument.
/// 5. Check user_tda (if present) lies in user address space → InvalidArgument.
/// 6. Call pm.create_thread → Success(tid) or Error.
///
/// # Parameters
///
/// - `args_addr_valid`: Whether the thread_create_args pointer lies in user space.
/// - `copy_succeeded`: Whether copy_from_user succeeded.
/// - `copy_error_code`: The error code from copy_from_user (if it failed).
/// - `thread_args`: The validation results for thread_create_args fields.
///
/// # Returns
///
/// A tuple of:
/// - `KcallResultModel`: The kcall result (Success with TID or Error).
/// - `Ghost<CreateThreadInputView>`: Ghost input view for postcondition exposure.
/// - `Ghost<CreateThreadOutcomeView>`: Ghost PM outcome for postcondition exposure.
pub fn create_thread_model(
    args_addr_valid: bool,
    copy_succeeded: bool,
    copy_error_code: i32,
    thread_args: &ThreadCreateArgsModel,
) -> (ret: (KcallResultModel, Ghost<CreateThreadInputView>, Ghost<CreateThreadOutcomeView>))
    ensures
        // Build the ghost input from parameters.
        ret.1@ == (CreateThreadInputView {
            args_addr_valid: args_addr_valid,
            copy_succeeded: copy_succeeded,
            copy_error_code: copy_error_code as int,
            thread_args: thread_args.spec_view(),
        }),
        // The result matches the spec pipeline.
        ret.0.spec_view() == spec_create_thread_result(ret.1@, ret.2@),
        // Args address error path: InvalidArgument returned immediately.
        !args_addr_valid
            ==> ret.0.spec_view() == (CreateThreadResultView::Error {
                    error_code: ERROR_CODE_INVALID_ARGUMENT()
                }),
        // Copy error path: copy error code propagated.
        args_addr_valid && !copy_succeeded
            ==> ret.0.spec_view() == (CreateThreadResultView::Error {
                    error_code: copy_error_code as int
                }),
        // Success path: all validations passed and PM succeeded.
        spec_is_success(ret.0.spec_view())
            ==> spec_all_validations_passed(ret.1@)
                && spec_pm_create_thread_ok(ret.2@),
        // Result is always Success or Error.
        spec_is_success(ret.0.spec_view()) || spec_is_error(ret.0.spec_view()),
        // Success and Error are mutually exclusive.
        !(spec_is_success(ret.0.spec_view()) && spec_is_error(ret.0.spec_view())),
{
    // Build the ghost input view.
    let ghost input_view: CreateThreadInputView = CreateThreadInputView {
        args_addr_valid: args_addr_valid,
        copy_succeeded: copy_succeeded,
        copy_error_code: copy_error_code as int,
        thread_args: thread_args.spec_view(),
    };

    // Step 1: Check if thread_create_args lies in user space.
    let addr_valid: bool = is_user_region(args_addr_valid);
    if !addr_valid {
        // Dummy PM outcome for the error path.
        let ghost pm_view: CreateThreadOutcomeView = CreateThreadOutcomeView::CtOk { tid: 0 };
        proof {
            lemma_args_addr_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: 22i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 2: Copy thread_create_args from user space.
    let copy_result: CopyFromUserResultModel = copy_from_user(copy_succeeded, copy_error_code);
    match copy_result {
        CopyFromUserResultModel::CopyError { error_code } => {
            let ghost pm_view: CreateThreadOutcomeView = CreateThreadOutcomeView::CtOk { tid: 0 };
            proof {
                lemma_copy_error_propagates(input_view, pm_view);
                lemma_result_exhaustive(input_view, pm_view);
            }
            return (
                KcallResultModel::Error { error_code },
                Ghost(input_view),
                Ghost(pm_view),
            );
        },
        CopyFromUserResultModel::CopyOk => {
            // Continue to validation steps.
        },
    }

    // Step 3: Check user_fn lies in user address space.
    let fn_valid: bool = is_user_addr(thread_args.user_fn_valid);
    if !fn_valid {
        let ghost pm_view: CreateThreadOutcomeView = CreateThreadOutcomeView::CtOk { tid: 0 };
        proof {
            lemma_user_fn_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: 22i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 4: Check user_stack lies in user address space with sufficient size.
    let stack_region_valid: bool = is_user_region(thread_args.user_stack_valid);
    if !stack_region_valid {
        let ghost pm_view: CreateThreadOutcomeView = CreateThreadOutcomeView::CtOk { tid: 0 };
        proof {
            lemma_user_stack_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: 22i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 4b: Check user_stack_size >= USER_STACK_SIZE.
    let stack_size_valid: bool = is_user_region(thread_args.user_stack_size_valid);
    if !stack_size_valid {
        let ghost pm_view: CreateThreadOutcomeView = CreateThreadOutcomeView::CtOk { tid: 0 };
        proof {
            lemma_user_stack_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: 22i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 5: Check user_tda (if present) lies in user address space.
    if thread_args.has_user_tda {
        let tda_valid: bool = is_user_addr(thread_args.user_tda_valid);
        if !tda_valid {
            let ghost pm_view: CreateThreadOutcomeView = CreateThreadOutcomeView::CtOk { tid: 0 };
            proof {
                lemma_user_tda_invalid_propagates(input_view, pm_view);
                lemma_result_exhaustive(input_view, pm_view);
            }
            return (
                KcallResultModel::Error { error_code: 22i32 },
                Ghost(input_view),
                Ghost(pm_view),
            );
        }
    } else {
        proof {
            lemma_absent_tda_always_valid(input_view);
        }
    }

    // Step 6: All validations passed. Call PM create_thread.
    let pm_result: CreateThreadResultModel = pm_create_thread();
    let ghost pm_view: CreateThreadOutcomeView = pm_result.spec_view();

    proof {
        lemma_result_exhaustive(input_view, pm_view);
    }

    match pm_result {
        CreateThreadResultModel::CtOk { tid } => {
            let tid_value: i32 = tid;
            (
                KcallResultModel::Success { tid_value },
                Ghost(input_view),
                Ghost(pm_view),
            )
        },
        CreateThreadResultModel::CtError { error_code } => {
            (
                KcallResultModel::Error { error_code },
                Ghost(input_view),
                Ghost(pm_view),
            )
        },
    }
}

} // verus!
