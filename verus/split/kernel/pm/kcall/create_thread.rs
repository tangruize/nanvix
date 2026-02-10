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
//! - **Argument identity tracking**: Ghost `pid` parameter is threaded through
//!   both `copy_from_user` and `pm_create_thread`, ensuring the same PID from
//!   `KcallArgs` is used in both operations. Ghost address parameters on
//!   `is_user_region` and `is_user_addr` record `arg0`, `args_size`,
//!   `user_fn_addr`, `user_stack_base_addr`, and `user_tda_addr` at each
//!   validation step. All ghost addresses are exposed in the `CreateThreadInputView`
//!   postcondition for cross-module composition.
//! - **Copy error validity**: The `copy_from_user` external body guarantees
//!   that error codes are valid positive values (`spec_is_valid_error_code`).
//!   The `spec_is_error_code_value` predicate enumerates the subset of ErrorCode
//!   variants present in the Verus verification model; the full kernel `ErrorCode`
//!   enum has additional variants. Module-level proofs in PM/VMM can strengthen
//!   the constraint for specific call sites.
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
//! ## Logging
//!
//! The original code logs with `error!()` before each error return. This logging
//! is not modeled because it has no functional effect on the return value. The
//! verification scope is limited to functional correctness of the validation
//! pipeline.
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
//!   Error codes are guaranteed valid (positive) by postcondition.
//! - **T4: `ProcessManager::create_thread(mm, pid, args)`**. Creates a thread
//!   in the PM. Modeled as `external_body` with ghost parameters for `pid` and
//!   `thread_create_args` to track argument identity through the pipeline.
//! - **T5: `args.arg0 as usize` cast**. The original performs
//!   `VirtualAddress::from_raw_value(args.arg0 as usize)` which casts a u32
//!   field to usize. On Nanvix's x86-32 target, usize is 32 bits, so this cast
//!   is an identity. This assumption is trusted as part of the KcallArgs
//!   abstraction and is not modeled.
//! - **T6: `<i32>::from(tid).into()` conversion**. The original success path
//!   performs `KcallResult::Success(<i32>::from(tid).into())` which converts
//!   `ThreadIdentifier → i32 → KcallResult`. The `pm_create_thread` external
//!   body postcondition guarantees `tid >= 0`, ensuring the conversion is the
//!   correct i32 representation. The `ThreadIdentifier → i32` conversion is
//!   verified in the `tid` module.
//!
//! ### Abstraction Correctness
//!
//! The validation oracles (`is_user_region`, `is_user_addr`) are modeled as
//! boolean identity functions with ghost address parameters. The ghost parameters
//! record which concrete address/size was validated at each call site, enabling
//! postconditions to link validation results to specific addresses:
//! - Step 1: `is_user_region(valid, Ghost(arg0), Ghost(args_size))` — validates
//!   the `ThreadCreateArgs` pointer region.
//! - Step 3: `is_user_addr(valid, Ghost(user_fn_addr))` — validates user_fn.
//! - Step 4: `is_user_region(valid, Ghost(user_stack_base_addr), Ghost(user_stack_size))`
//!   — validates user_stack region.
//! - Step 5: `is_user_addr(valid, Ghost(user_tda_addr))` — validates user_tda.
//!
//! The `CreateThreadInputView` carries all ghost addresses (`arg0`, `args_size`,
//! `user_fn_addr`, `user_stack_base_addr`, `user_tda_addr`), so postconditions
//! can relate validated addresses to the input view.
//!
//! The **mapping from concrete types (`VirtualAddress`, `KcallArgs`,
//! `ThreadCreateArgs`) to boolean inputs remains trusted**. The verification
//! proves that the control-flow dispatch logic is correct given boolean inputs
//! AND that the correct addresses are threaded through each validation step.
//! However, it does NOT prove that the booleans faithfully represent the
//! concrete `Vmem::is_user_region`/`Vmem::is_user_addr` results.
//!
//! This is an intentional design choice matching the project's per-module
//! verification approach:
//! - The VMM module verifies `Vmem::is_user_region` and `Vmem::is_user_addr`.
//! - The PM module verifies `copy_from_user`.
//! - This kcall module verifies the dispatch pipeline and address threading.
//!
//! Full end-to-end soundness requires composing these module-level proofs.
//! A future refinement could add linking lemmas connecting concrete types to
//! the boolean abstraction by importing VMM spec functions.
//!
//! ## API Mapping
//!
//! | Original API                              | Verified Model                         | Notes           |
//! |-------------------------------------------|----------------------------------------|-----------------|
//! | `Vmem::is_user_region(addr, size)`        | `is_user_region(valid, Ghost(addr), Ghost(size))` | external_body |
//! | `Vmem::is_user_addr(addr)`                | `is_user_addr(valid, Ghost(addr))`     | external_body   |
//! | `thread_create_args.user_stack_size < ..`  | Direct `u32` comparison                | Fully verified  |
//! | `pm::copy_from_user(pm, pid, dst, src)`   | `copy_from_user(succeeded, error_code, ghost_pid)` | external_body |
//! | `pm.create_thread(mm, pid, args)`         | `pm_create_thread(ghost_pid, ghost_args)` | external_body |
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
/// validated individually. This struct captures the validation results
/// along with the concrete addresses from the copied structure.
///
/// ## Concrete Address Fields
///
/// `user_fn_addr`, `user_stack_base_addr`, and `user_tda_addr` are
/// the concrete addresses extracted from the copied `ThreadCreateArgs`.
/// These link the validation booleans to the actual data from user space,
/// enabling postconditions to verify that the addresses validated by
/// `is_user_addr`/`is_user_region` are the same as those in the copied args.
///
/// ## Omitted Fields
///
/// `user_fn_arg0` and `user_fn_arg1` are not modeled because `create_thread`
/// does not validate them — they are passed through to `pm.create_thread`
/// unchanged. Argument passthrough verification is out of scope for this
/// module; it would require PM-level ghost state tracking.
///
/// ## Architecture Dependency
///
/// `user_stack_size` uses `u32` which matches `usize` on the x86-32 target.
/// If Nanvix targets a 64-bit architecture, this field and the
/// `USER_STACK_SIZE()` spec constant must be updated.
pub struct ThreadCreateArgsModel {
    /// The user_fn address (concrete value from copied args).
    pub user_fn_addr: u32,
    /// Whether user_fn lies in user address space.
    pub user_fn_valid: bool,
    /// The user_stack_base address (concrete value from copied args).
    pub user_stack_base_addr: u32,
    /// Whether user_stack region lies in user address space.
    pub user_stack_valid: bool,
    /// The user stack size in bytes.
    pub user_stack_size: u32,
    /// Whether user_tda is present.
    pub has_user_tda: bool,
    /// The user_tda address (concrete value from copied args; 0 if absent).
    pub user_tda_addr: u32,
    /// Whether user_tda (if present) lies in user address space.
    pub user_tda_valid: bool,
}

impl ThreadCreateArgsModel {
    /// Spec function: converts to the abstract ThreadCreateArgsView.
    pub open spec fn spec_view(&self) -> ThreadCreateArgsView {
        ThreadCreateArgsView {
            user_fn_addr: self.user_fn_addr as nat,
            user_fn_valid: self.user_fn_valid,
            user_stack_base_addr: self.user_stack_base_addr as nat,
            user_stack_valid: self.user_stack_valid,
            user_stack_size: self.user_stack_size as nat,
            has_user_tda: self.has_user_tda,
            user_tda_addr: self.user_tda_addr as nat,
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
/// lies entirely within user address space. The boolean parameter is a
/// pre-computed result from the concrete `Vmem::is_user_region` call.
/// Ghost parameters record which address and size were validated,
/// enabling postconditions to link validation results to concrete values.
#[verifier::external_body]
pub fn is_user_region(
    valid: bool,
    Ghost(ghost_addr): Ghost<nat>,
    Ghost(ghost_size): Ghost<nat>,
) -> (result: bool)
    ensures
        result == valid,
{
    unimplemented!()
}

/// Trust Boundary T2: Models `Vmem::is_user_addr(addr)`.
///
/// # Description
///
/// Checks whether an address lies within user address space. The boolean
/// parameter is a pre-computed result from the concrete `Vmem::is_user_addr`
/// call. Ghost parameter records which address was validated.
#[verifier::external_body]
pub fn is_user_addr(
    valid: bool,
    Ghost(ghost_addr): Ghost<nat>,
) -> (result: bool)
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
///
/// Error codes from `copy_from_user` are guaranteed valid (positive) because
/// the original returns `error.code` which is an `ErrorCode` enum value.
///
/// On success, the copied `ThreadCreateArgs` structure produces concrete
/// addresses (user_fn, user_stack_base, user_tda) that are subsequently
/// validated by `is_user_addr`/`is_user_region`. The ghost `args_view`
/// parameter tracks these addresses through the pipeline.
#[verifier::external_body]
pub fn copy_from_user(
    succeeded: bool,
    error_code: i32,
    Ghost(ghost_pid): Ghost<nat>,
    Ghost(ghost_args_view): Ghost<ThreadCreateArgsView>,
) -> (result: CopyFromUserResultModel)
    ensures
        succeeded ==> matches!(result, CopyFromUserResultModel::CopyOk),
        !succeeded ==> (result matches CopyFromUserResultModel::CopyError { error_code: ec }
            && ec == error_code),
        result.spec_succeeded() == succeeded,
        !succeeded ==> result.spec_error_code() == error_code as int,
        // Error codes are always valid positive values (ErrorCode enum discriminants).
        !succeeded ==> spec_is_valid_error_code(error_code as int),
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
/// Ghost parameters track argument identity through the pipeline:
/// - `ghost_pid`: The process identifier passed to create_thread.
/// - `ghost_args`: The thread creation args passed to create_thread.
///
/// This ensures that the correct arguments are forwarded from the
/// validated inputs to the PM call. The postconditions link the ghost
/// arguments to the result.
///
/// The TID on success is guaranteed `>= 0` because `ThreadIdentifier`
/// values are non-negative. The `<i32>::from(tid)` conversion in the
/// original is lossless for valid TIDs, verified in the `tid` module.
///
/// Postconditions capture:
/// - The result is always one of the defined variants.
/// - On success, the TID is a valid non-negative value.
/// - On failure, the error code is a valid positive value.
#[verifier::external_body]
pub fn pm_create_thread(
    Ghost(ghost_pid): Ghost<nat>,
    Ghost(ghost_args): Ghost<ThreadCreateArgsView>,
) -> (result: CreateThreadResultModel)
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
/// - `user_stack_size_min`: The minimum user stack size (USER_STACK_SIZE from config).
/// - `Ghost(ghost_pid)`: Ghost PID for argument identity tracking.
/// - `Ghost(ghost_arg0)`: Ghost arg0 (raw address) for argument identity tracking.
/// - `Ghost(ghost_args_size)`: Ghost size_of::<ThreadCreateArgs>() for address linkage.
/// - `Ghost(ghost_user_fn_addr)`: Ghost user_fn address for validation linkage.
/// - `Ghost(ghost_user_stack_base_addr)`: Ghost user_stack_base address for validation linkage.
/// - `Ghost(ghost_user_tda_addr)`: Ghost user_tda address for validation linkage.
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
    user_stack_size_min: u32,
    Ghost(ghost_pid): Ghost<nat>,
    Ghost(ghost_arg0): Ghost<nat>,
    Ghost(ghost_args_size): Ghost<nat>,
    Ghost(ghost_user_fn_addr): Ghost<nat>,
    Ghost(ghost_user_stack_base_addr): Ghost<nat>,
    Ghost(ghost_user_tda_addr): Ghost<nat>,
) -> (ret: (KcallResultModel, Ghost<CreateThreadInputView>, Ghost<CreateThreadOutcomeView>))
    requires
        // The minimum stack size parameter matches the spec constant.
        user_stack_size_min as nat == USER_STACK_SIZE(),
        // Copy error code must be valid when copy fails.
        !copy_succeeded ==> spec_is_valid_error_code(copy_error_code as int),
        // The args_size parameter matches the ThreadCreateArgs struct size.
        ghost_args_size == THREAD_CREATE_ARGS_SIZE(),
        // Ghost addresses must match the concrete addresses in thread_args.
        // This ensures validation booleans correspond to the copied args.
        ghost_user_fn_addr == thread_args.user_fn_addr as nat,
        ghost_user_stack_base_addr == thread_args.user_stack_base_addr as nat,
        ghost_user_tda_addr == thread_args.user_tda_addr as nat,
    ensures
        // Build the ghost input from parameters.
        ret.1@ == (CreateThreadInputView {
            pid: ghost_pid,
            arg0: ghost_arg0,
            args_size: ghost_args_size,
            user_fn_addr: ghost_user_fn_addr,
            user_stack_base_addr: ghost_user_stack_base_addr,
            user_tda_addr: ghost_user_tda_addr,
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
        // Ghost addresses match thread_args concrete addresses (data-flow linkage).
        ret.1@.user_fn_addr == ret.1@.thread_args.user_fn_addr,
        ret.1@.user_stack_base_addr == ret.1@.thread_args.user_stack_base_addr,
        ret.1@.user_tda_addr == ret.1@.thread_args.user_tda_addr,
        // Args size matches the spec constant.
        ret.1@.args_size == THREAD_CREATE_ARGS_SIZE(),
{
    // Build the ghost input view.
    let ghost input_view: CreateThreadInputView = CreateThreadInputView {
        pid: ghost_pid,
        arg0: ghost_arg0,
        args_size: ghost_args_size,
        user_fn_addr: ghost_user_fn_addr,
        user_stack_base_addr: ghost_user_stack_base_addr,
        user_tda_addr: ghost_user_tda_addr,
        args_addr_valid: args_addr_valid,
        copy_succeeded: copy_succeeded,
        copy_error_code: copy_error_code as int,
        thread_args: thread_args.spec_view(),
    };

    // Step 1: Check if thread_create_args lies in user space.
    let addr_valid: bool = is_user_region(args_addr_valid, Ghost(ghost_arg0), Ghost(ghost_args_size));
    if !addr_valid {
        let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
        proof {
            lemma_args_addr_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: ErrorCode::InvalidArgument as i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 2: Copy thread_create_args from user space.
    let copy_result: CopyFromUserResultModel = copy_from_user(
        copy_succeeded,
        copy_error_code,
        Ghost(ghost_pid),
        Ghost(thread_args.spec_view()),
    );
    match copy_result {
        CopyFromUserResultModel::CopyError { error_code } => {
            let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
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
    let fn_valid: bool = is_user_addr(thread_args.user_fn_valid, Ghost(ghost_user_fn_addr));
    if !fn_valid {
        let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
        proof {
            lemma_user_fn_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: ErrorCode::InvalidArgument as i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 4: Check user_stack lies in user address space.
    let stack_region_valid: bool = is_user_region(
        thread_args.user_stack_valid,
        Ghost(ghost_user_stack_base_addr),
        Ghost(thread_args.user_stack_size as nat),
    );
    if !stack_region_valid {
        let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
        proof {
            lemma_user_stack_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: ErrorCode::InvalidArgument as i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 4b: Check user_stack_size >= USER_STACK_SIZE (concrete numeric comparison).
    if thread_args.user_stack_size < user_stack_size_min {
        let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
        proof {
            lemma_user_stack_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: ErrorCode::InvalidArgument as i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 5: Check user_tda (if present) lies in user address space.
    if thread_args.has_user_tda {
        let tda_valid: bool = is_user_addr(thread_args.user_tda_valid, Ghost(ghost_user_tda_addr));
        if !tda_valid {
            let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
            proof {
                lemma_user_tda_invalid_propagates(input_view, pm_view);
                lemma_result_exhaustive(input_view, pm_view);
            }
            return (
                KcallResultModel::Error { error_code: ErrorCode::InvalidArgument as i32 },
                Ghost(input_view),
                Ghost(pm_view),
            );
        }
    } else {
        proof {
            lemma_absent_tda_always_valid(input_view);
        }
    }

    // Step 6: All validations passed. Call PM create_thread with ghost argument identity.
    let pm_result: CreateThreadResultModel =
        pm_create_thread(Ghost(ghost_pid), Ghost(thread_args.spec_view()));
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
