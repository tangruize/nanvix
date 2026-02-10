// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Terminate Kernel Call Specification.
// Defines View types, spec constants, and spec functions for the terminate
// kernel call verification model.
//
// ## Verification Model
//
// The terminate kcall implements a two-step pipeline:
//   1. Parse ProcessIdentifier from raw u32 argument via try_from.
//   2. Call ProcessManager::terminate(pid).
//
// This spec models:
// - PID parsing as a fallible conversion with two outcomes (Ok or Error).
// - The terminate operation as a fallible call with two outcomes (Ok or Error).
// - The overall result as a sequential composition with short-circuit on error.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Spec Constants
//==================================================================================================

/// The ErrorCode value for InvalidArgument (repr(i32) = 22).
/// Used when ProcessIdentifier::try_from fails on an invalid raw PID value.
pub open spec fn ERROR_CODE_INVALID_ARGUMENT() -> int {
    22
}

/// Maximum value for usize on x86-32 (used for ABI boundary reasoning).
pub open spec fn USIZE_MAX_X86_32() -> nat {
    u32::MAX as nat
}

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of the PID parsing outcome.
///
/// # Description
///
/// Models the result of `ProcessIdentifier::try_from(arg0)`:
/// - `PidOk`: The raw u32 was a valid PID.
/// - `PidError`: The raw u32 was invalid; carries the error code.
#[verifier::ext_equal]
pub enum PidParseOutcomeView {
    /// ProcessIdentifier::try_from succeeded.
    PidOk { pid: nat },
    /// ProcessIdentifier::try_from failed with an error code.
    PidError { error_code: int },
}

/// Abstract view of the ProcessManager::terminate outcome.
///
/// # Description
///
/// Models the result of `pm.terminate(pid)`:
/// - `TmOk`: The process was successfully terminated.
/// - `TmError`: The terminate call failed; carries the error code.
#[verifier::ext_equal]
pub enum TerminateOutcomeView {
    /// ProcessManager::terminate returned Ok(()).
    TmOk,
    /// ProcessManager::terminate returned Err(error).
    TmError { error_code: int },
}

/// Abstract view of the terminate kcall's final result.
///
/// # Description
///
/// The terminate function returns KcallResult, which is either:
/// - Success: PID was parsed and process was terminated.
/// - Error: Either PID parsing failed or pm.terminate failed.
#[verifier::ext_equal]
pub enum TerminateResultView {
    /// Both steps succeeded: process was terminated.
    Success,
    /// An error occurred (either PID parse error or terminate error).
    Error { error_code: int },
}

//==================================================================================================
// Spec Functions
//==================================================================================================

/// Spec function: models the complete terminate kcall pipeline.
///
/// # Description
///
/// The terminate function executes a sequential pipeline:
/// 1. Parse PID from raw arg → Error on failure.
/// 2. pm.terminate(pid) → Error on failure.
/// 3. Both succeeded → Success.
///
/// Each step only executes if the previous step succeeded (short-circuit).
pub open spec fn spec_terminate_result(
    pid_parse_outcome: PidParseOutcomeView,
    terminate_outcome: TerminateOutcomeView,
) -> TerminateResultView {
    match pid_parse_outcome {
        PidParseOutcomeView::PidError { error_code } => {
            TerminateResultView::Error { error_code }
        },
        PidParseOutcomeView::PidOk { .. } => {
            match terminate_outcome {
                TerminateOutcomeView::TmError { error_code } => {
                    TerminateResultView::Error { error_code }
                },
                TerminateOutcomeView::TmOk => {
                    TerminateResultView::Success
                },
            }
        },
    }
}

/// Spec function: whether the result is success.
pub open spec fn spec_is_success(result: TerminateResultView) -> bool {
    matches!(result, TerminateResultView::Success)
}

/// Spec function: whether the result is an error.
pub open spec fn spec_is_error(result: TerminateResultView) -> bool {
    matches!(result, TerminateResultView::Error { .. })
}

/// Spec function: whether the PID was parsed successfully.
pub open spec fn spec_pid_parsed_ok(outcome: PidParseOutcomeView) -> bool {
    matches!(outcome, PidParseOutcomeView::PidOk { .. })
}

/// Spec function: whether the terminate operation succeeded.
pub open spec fn spec_terminate_ok(outcome: TerminateOutcomeView) -> bool {
    matches!(outcome, TerminateOutcomeView::TmOk)
}

/// Spec function: extract the error code from a PID parse failure.
pub open spec fn spec_pid_error_code(outcome: PidParseOutcomeView) -> int
    recommends !spec_pid_parsed_ok(outcome),
{
    match outcome {
        PidParseOutcomeView::PidError { error_code } => error_code,
        PidParseOutcomeView::PidOk { .. } => 0int,
    }
}

/// Spec function: extract the error code from a terminate failure.
pub open spec fn spec_terminate_error_code(outcome: TerminateOutcomeView) -> int
    recommends !spec_terminate_ok(outcome),
{
    match outcome {
        TerminateOutcomeView::TmError { error_code } => error_code,
        TerminateOutcomeView::TmOk => 0int,
    }
}

/// Spec predicate: whether an error code is a valid positive error code.
///
/// # Description
///
/// All POSIX errno values are positive integers. This predicate captures
/// the essential invariant for error codes returned by kernel functions.
pub open spec fn spec_is_valid_error_code(code: int) -> bool {
    code > 0
}

} // verus!
