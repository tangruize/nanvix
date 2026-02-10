// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Join Thread Kernel Call Specification.
// Defines View types, spec constants, and spec functions for the join_thread
// kernel call verification model.
//
// ## Verification Model
//
// The join_thread kcall implements a 3-step pipeline:
//   1. Parse ThreadIdentifier from arg0 via ThreadIdentifier::try_from(u32).
//   2. Call ProcessManager::join_thread(pid, tid) — blocks until target exits.
//   3. Copy the ExitStatus to user space via copy_to_user.
//
// This spec models:
// - TID parsing as a fallible conversion with two outcomes (Ok or Error).
// - The join operation as a blocking call returning ExitStatus or SleepError.
// - The copy_to_user operation as a fallible step with error propagation.
// - The overall result as a sequential composition with short-circuit on error.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Spec Constants
//==================================================================================================

/// The ErrorCode value for InvalidArgument (repr(i32) = 22).
/// Used when ThreadIdentifier::try_from fails on an invalid raw TID value.
pub open spec fn ERROR_CODE_INVALID_ARGUMENT() -> int {
    22
}

/// The ExitStatus value representing success (0).
/// Corresponds to ExitStatus::ok() which returns ExitStatus(0).
pub open spec fn EXIT_STATUS_OK() -> int {
    0
}

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of the TID parsing outcome.
///
/// # Description
///
/// Models the result of `ThreadIdentifier::try_from(arg0)`:
/// - `TidOk`: The raw u32 was a valid TID.
/// - `TidError`: The raw u32 was invalid; carries the error code.
#[verifier::ext_equal]
pub enum TidParseOutcomeView {
    /// ThreadIdentifier::try_from succeeded.
    TidOk { tid: nat },
    /// ThreadIdentifier::try_from failed with an error code.
    TidError { error_code: int },
}

/// Abstract view of the ProcessManager::join_thread outcome.
///
/// # Description
///
/// Models the result of `ProcessManager::join_thread(pid, tid)`:
/// - `JtOk`: The thread was successfully joined; carries the exit status.
/// - `JtInterruptedKilled`: The joining thread was killed while waiting.
/// - `JtError`: A generic error occurred; carries the error code.
///
/// Note: join_thread uses the `?` operator, so SleepError is propagated
/// directly. SleepError has two variants:
/// - Interrupted(InterruptReason) — modeled as JtInterruptedKilled.
/// - Generic(Error) — modeled as JtError.
#[verifier::ext_equal]
pub enum JoinThreadOutcomeView {
    /// ProcessManager::join_thread returned Ok(exit_status).
    JtOk { exit_status: int },
    /// ProcessManager::join_thread returned Err(SleepError::Interrupted(Killed)).
    JtInterruptedKilled,
    /// ProcessManager::join_thread returned Err(SleepError::Generic(error)).
    JtError { error_code: int },
}

/// Abstract view of the copy_to_user outcome.
///
/// # Description
///
/// Models the result of `pm::copy_to_user(pm, pid, retval, &status)`:
/// - `CopyOk`: The copy succeeded.
/// - `CopyError`: The copy failed; carries the error code.
///   Error is wrapped in SleepError::Generic by `.map_err(SleepError::Generic)`.
#[verifier::ext_equal]
pub enum CopyToUserOutcomeView {
    /// copy_to_user returned Ok(()).
    CopyOk,
    /// copy_to_user returned Err(error).
    CopyError { error_code: int },
}

/// Abstract view of the join_thread kcall's final result.
///
/// # Description
///
/// The join_thread function returns `Result<ExitStatus, SleepError>`:
/// - `Success`: All steps succeeded; returns Ok(ExitStatus::ok()).
/// - `GenericError`: TID parse failed, join_thread had generic error, or copy_to_user failed.
/// - `InterruptedKilled`: The joining thread was killed while blocking in join_thread.
#[verifier::ext_equal]
pub enum JoinThreadResultView {
    /// All steps succeeded. Returns Ok(ExitStatus::ok()).
    Success { exit_status: int },
    /// A generic error occurred at some pipeline step.
    GenericError { error_code: int },
    /// The joining thread was killed while waiting (SleepError::Interrupted(Killed)).
    InterruptedKilled,
}

//==================================================================================================
// Spec Predicates
//==================================================================================================

/// Whether a raw u32 value is a valid ThreadIdentifier.
///
/// # Description
///
/// Abstract predicate that characterizes which raw values pass
/// `ThreadIdentifier::try_from(u32)`. The tid module provides the concrete
/// interpretation. This module uses it to make the try_from external_body
/// deterministic.
///
/// The actual `ThreadIdentifier` wraps an `i32` internally, so `try_from(u32)`
/// succeeds iff the u32 value fits in a non-negative i32 (i.e., `raw <= i32::MAX`).
/// The axiom `axiom_valid_tid_range` in the proof file establishes this bound.
/// TID representation is modeled at the kcall interface level (u32) rather than
/// the internal representation (i32), since the kcall only sees the u32 form.
pub uninterp spec fn spec_is_valid_tid(raw: nat) -> bool;

/// Uninterpreted predicate: whether user memory at the given address for
/// the given process has been written with the specified value.
///
/// # Description
///
/// Models the abstract postcondition of `copy_to_user`: on success, the
/// user-space memory at `retval_addr` for process `pid` contains `value`.
/// The concrete memory model is outside this module's scope; this predicate
/// establishes the proof obligation for the memory management module.
pub uninterp spec fn spec_user_mem_written(pid: nat, retval_addr: nat, value: int) -> bool;

/// Whether the TID was parsed successfully.
pub open spec fn spec_tid_parsed_ok(outcome: TidParseOutcomeView) -> bool {
    matches!(outcome, TidParseOutcomeView::TidOk { .. })
}

/// Whether the join_thread operation succeeded.
pub open spec fn spec_join_ok(outcome: JoinThreadOutcomeView) -> bool {
    matches!(outcome, JoinThreadOutcomeView::JtOk { .. })
}

/// Whether the copy_to_user operation succeeded.
pub open spec fn spec_copy_ok(outcome: CopyToUserOutcomeView) -> bool {
    matches!(outcome, CopyToUserOutcomeView::CopyOk)
}

/// Whether the result is success.
pub open spec fn spec_is_success(result: JoinThreadResultView) -> bool {
    matches!(result, JoinThreadResultView::Success { .. })
}

/// Whether the result is a generic error.
pub open spec fn spec_is_generic_error(result: JoinThreadResultView) -> bool {
    matches!(result, JoinThreadResultView::GenericError { .. })
}

/// Whether the result is an interrupted-killed error.
pub open spec fn spec_is_interrupted_killed(result: JoinThreadResultView) -> bool {
    matches!(result, JoinThreadResultView::InterruptedKilled)
}

/// Whether the result is any kind of error (generic or interrupted-killed).
pub open spec fn spec_is_error(result: JoinThreadResultView) -> bool {
    spec_is_generic_error(result) || spec_is_interrupted_killed(result)
}

/// Spec predicate: whether an error code is a valid positive error code.
pub open spec fn spec_is_valid_error_code(code: int) -> bool {
    code > 0
}

/// Spec predicate: whether an error code matches a known ErrorCode variant.
///
/// # Description
///
/// Enumerates the discriminant values of the `ErrorCode` variants present
/// in the Verus verification model (`verus/split/libs/error/lib.rs`):
/// NoSuchEntry=2, NoSuchProcess=3, OutOfMemory=12, BadAddress=14,
/// ResourceBusy=16, InvalidArgument=22.
///
/// NOTE: The full kernel `ErrorCode` enum defines additional variants.
/// This predicate covers the subset used in the Verus model. External body
/// postconditions use the broader `spec_is_valid_error_code(code > 0)` to
/// avoid unsound over-constraint.
pub open spec fn spec_is_known_error_code(code: int) -> bool {
    code == 2     // NoSuchEntry
    || code == 3  // NoSuchProcess
    || code == 12 // OutOfMemory
    || code == 14 // BadAddress
    || code == 16 // ResourceBusy
    || code == 22 // InvalidArgument
}

/// Spec predicate: whether the calling process is a user process (not kernel).
///
/// # Description
///
/// Safety precondition: the original function documents that this function
/// panics if the kernel process tries to sleep. The calling process must
/// not be the kernel process (PID 0).
pub uninterp spec fn spec_is_user_process(pid: nat) -> bool;

/// Spec predicate: whether the process manager is initialized.
///
/// # Description
///
/// Safety precondition: the original function requires that the process
/// manager is initialized and access to it is synchronized before calling.
/// This subsumes both initialization and synchronization requirements.
pub uninterp spec fn spec_pm_initialized() -> bool;

/// Spec predicate: whether the memory manager is initialized.
///
/// # Description
///
/// Safety precondition: the original function requires that the memory
/// manager is initialized and access to it is synchronized before calling.
/// This subsumes both initialization and synchronization requirements.
pub uninterp spec fn spec_mm_initialized() -> bool;

/// Spec predicate: whether the caller holds no resources.
///
/// # Description
///
/// Safety precondition: the original function documents that it must be
/// invoked without holding any resources, since `join_thread` blocks
/// the calling thread until the target thread exits. Holding resources
/// (e.g., locks, borrowed references) while blocking could cause deadlocks.
pub uninterp spec fn spec_no_resources_held() -> bool;

//==================================================================================================
// Spec Functions
//==================================================================================================

/// Spec function: models the complete join_thread kcall pipeline.
///
/// # Description
///
/// The join_thread function executes a sequential pipeline:
/// 1. Parse TID from arg0 → GenericError(error_code) on failure.
/// 2. ProcessManager::join_thread(pid, tid) → propagate SleepError on failure.
/// 3. copy_to_user(pm, pid, retval, &status) → GenericError on failure.
/// 4. Return Ok(ExitStatus::ok()) on success.
///
/// Each step only executes if the previous step succeeded (short-circuit).
pub open spec fn spec_join_thread_result(
    tid_parse_outcome: TidParseOutcomeView,
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
) -> JoinThreadResultView {
    match tid_parse_outcome {
        TidParseOutcomeView::TidError { error_code } => {
            // Step 1 failed: return Err(SleepError::Generic(error)).
            JoinThreadResultView::GenericError { error_code }
        },
        TidParseOutcomeView::TidOk { .. } => {
            // Step 1 passed. Execute step 2.
            match join_outcome {
                JoinThreadOutcomeView::JtError { error_code } => {
                    // Step 2 failed with generic error.
                    JoinThreadResultView::GenericError { error_code }
                },
                JoinThreadOutcomeView::JtInterruptedKilled => {
                    // Step 2 failed: thread was killed while joining.
                    JoinThreadResultView::InterruptedKilled
                },
                JoinThreadOutcomeView::JtOk { .. } => {
                    // Step 2 passed. Execute step 3 (copy_to_user).
                    match copy_outcome {
                        CopyToUserOutcomeView::CopyError { error_code } => {
                            // Step 3 failed: copy error wrapped in SleepError::Generic.
                            JoinThreadResultView::GenericError { error_code }
                        },
                        CopyToUserOutcomeView::CopyOk => {
                            // All steps passed. Return Ok(ExitStatus::ok()).
                            JoinThreadResultView::Success { exit_status: EXIT_STATUS_OK() }
                        },
                    }
                },
            }
        },
    }
}

/// Whether all pipeline steps passed (precondition for success).
pub open spec fn spec_all_steps_passed(
    tid_parse_outcome: TidParseOutcomeView,
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
) -> bool {
    spec_tid_parsed_ok(tid_parse_outcome)
    && spec_join_ok(join_outcome)
    && spec_copy_ok(copy_outcome)
}

/// Extract the exit status from a join outcome (defaults to 0 on non-Ok).
pub open spec fn spec_join_exit_status(outcome: JoinThreadOutcomeView) -> int {
    match outcome {
        JoinThreadOutcomeView::JtOk { exit_status } => exit_status,
        _ => 0int,
    }
}

/// Extract the error code from a TID parse failure.
pub open spec fn spec_tid_error_code(outcome: TidParseOutcomeView) -> int
    recommends !spec_tid_parsed_ok(outcome),
{
    match outcome {
        TidParseOutcomeView::TidError { error_code } => error_code,
        TidParseOutcomeView::TidOk { .. } => 0int,
    }
}

} // verus!
