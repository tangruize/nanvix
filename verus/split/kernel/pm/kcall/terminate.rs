// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Terminate Kernel Call Verification Model
//!
//! Formal verification of the terminate kernel call (`pm::kcall::terminate`).
//!
//! ## Overview
//!
//! The `terminate(pm, args)` function terminates a process identified by its PID.
//! It implements a two-step pipeline:
//! 1. Parse `ProcessIdentifier` from `args.arg0` via `ProcessIdentifier::try_from(u32)`.
//! 2. Call `ProcessManager::terminate(pid)`.
//!
//! On success, returns `KcallResult::ok()`. On failure at any step, returns
//! `KcallResult::Error(error.code.into())`.
//!
//! ## Verified Properties
//!
//! - **PID parse error propagation**: When `ProcessIdentifier::try_from` fails,
//!   the error code is returned immediately and the terminate step is skipped
//!   (`lemma_pid_parse_error_propagates`, `lemma_pid_parse_short_circuit`).
//! - **Terminate error propagation**: When `pm.terminate` fails, the error code
//!   is returned (`lemma_terminate_error_propagates`).
//! - **Success requires both steps**: The result is Success if and only if both
//!   PID parsing and terminate succeed (`lemma_success_requires_both_steps`).
//! - **Result exhaustiveness**: Every input combination produces exactly one
//!   result: Success or Error. These are mutually exclusive
//!   (`lemma_result_exhaustive`).
//! - **Error code preservation**: Error codes from both pipeline steps are
//!   preserved faithfully in the final result
//!   (`lemma_pid_error_code_preserved`, `lemma_terminate_error_code_preserved`).
//! - **InvalidArgument for bad PID**: Invalid PID values produce
//!   `ErrorCode::InvalidArgument` (22) (`lemma_invalid_pid_returns_invalid_argument`).
//! - **Error code linkage**: The spec constant `ERROR_CODE_INVALID_ARGUMENT()`
//!   is proven equal to `ErrorCode::InvalidArgument as int`
//!   (`lemma_error_code_matches`).
//! - **Success implies valid PID**: If the result is Success, then the PID was
//!   valid and the process existed (`lemma_success_implies_valid_pid`).
//!
//! ## Properties NOT Proven Here (Out of Scope)
//!
//! - "The terminated process is removed from the scheduler" (PM internal invariant).
//! - "Resources held by the terminated process are freed" (resource management).
//! - "The terminated process's threads are cleaned up" (thread management).
//!
//! These are internal PM invariants verified in the ProcessManager module.
//!
//! ## Trust Boundaries
//!
//! - **T1: `ProcessIdentifier::try_from(u32)`**. Parses a raw u32 into a valid
//!   PID. Modeled as `external_body` returning a `PidParseResultModel`. The
//!   parsing logic is verified in the sys/pid module.
//! - **T2: `ProcessManager::terminate(pid)`**. Terminates the process. Modeled
//!   as `external_body` returning a `TerminateResultModel`. The PM's internal
//!   correctness is verified separately.
//!
//! ## API Mapping
//!
//! | Original API                              | Verified Model                      | Notes           |
//! |-------------------------------------------|-------------------------------------|-----------------|
//! | `ProcessIdentifier::try_from(args.arg0)`  | `try_from_process_identifier(arg0)` | external_body   |
//! | `pm.terminate(pid)`                       | `process_manager_terminate(pid)`    | external_body   |
//! | `pub fn terminate(pm, args) -> KcallResult`| `terminate_model(arg0)`            | Fully verified  |

use crate::libs::error::ErrorCode;
use vstd::prelude::*;

// Include specifications.
include!("terminate.spec.rs");

// Include proofs.
include!("terminate.proof.rs");

verus! {

//==================================================================================================
// Dependency Models (External Bodies)
//==================================================================================================

/// Model of the PID parsing result for verification.
///
/// # Description
///
/// Represents the two possible outcomes from `ProcessIdentifier::try_from(u32)`:
/// - `Ok(pid)` → PidOk with the parsed PID value.
/// - `Err(error)` → PidError with the error code.
pub enum PidParseResultModel {
    /// ProcessIdentifier::try_from succeeded.
    PidOk { pid: u32 },
    /// ProcessIdentifier::try_from failed with an error code.
    PidError { error_code: i32 },
}

impl PidParseResultModel {
    /// Spec function: converts to the abstract PidParseOutcomeView.
    pub open spec fn spec_view(&self) -> PidParseOutcomeView {
        match self {
            PidParseResultModel::PidOk { pid } => {
                PidParseOutcomeView::PidOk { pid: *pid as nat }
            },
            PidParseResultModel::PidError { error_code } => {
                PidParseOutcomeView::PidError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the ProcessManager::terminate result for verification.
///
/// # Description
///
/// Represents the two possible outcomes from `pm.terminate(pid)`:
/// - `Ok(())` → TmOk.
/// - `Err(error)` → TmError with the error code.
pub enum TerminateResultModel {
    /// ProcessManager::terminate returned Ok(()).
    TmOk,
    /// ProcessManager::terminate returned Err(error).
    TmError { error_code: i32 },
}

impl TerminateResultModel {
    /// Spec function: converts to the abstract TerminateOutcomeView.
    pub open spec fn spec_view(&self) -> TerminateOutcomeView {
        match self {
            TerminateResultModel::TmOk => TerminateOutcomeView::TmOk,
            TerminateResultModel::TmError { error_code } => {
                TerminateOutcomeView::TmError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the KcallResult for verification.
///
/// # Description
///
/// Represents the two possible outcomes from the kcall:
/// - `Ok` → Success.
/// - `Error` → Error with an error code.
pub enum KcallResultModel {
    /// KcallResult::ok() — success.
    Ok,
    /// KcallResult::Error — error with code.
    Error { error_code: i32 },
}

impl KcallResultModel {
    /// Spec function: converts to the abstract TerminateResultView.
    pub open spec fn spec_view(&self) -> TerminateResultView {
        match self {
            KcallResultModel::Ok => TerminateResultView::Success,
            KcallResultModel::Error { error_code } => {
                TerminateResultView::Error { error_code: *error_code as int }
            },
        }
    }
}

//==================================================================================================
// External Body Functions (Trust Boundaries)
//==================================================================================================

/// Trust Boundary T1: Models `ProcessIdentifier::try_from(arg0)`.
///
/// # Description
///
/// Parses a raw u32 into a ProcessIdentifier. Returns PidOk on success or
/// PidError with ErrorCode::InvalidArgument on failure. The parsing logic
/// validates that the raw value represents a valid process identifier.
#[verifier::external_body]
pub fn try_from_process_identifier(arg0: u32) -> (result: PidParseResultModel)
    ensures
        // On failure, the error code is always InvalidArgument.
        result.spec_view() matches PidParseOutcomeView::PidError { error_code }
            ==> error_code == ERROR_CODE_INVALID_ARGUMENT(),
        // The result is always one of the defined variants.
        matches!(result, PidParseResultModel::PidOk { .. } | PidParseResultModel::PidError { .. }),
{
    unimplemented!()
}

/// Trust Boundary T2: Models `ProcessManager::terminate(pid)`.
///
/// # Description
///
/// Terminates the process identified by `pid`. Returns TmOk on success or
/// TmError with an error code on failure. Possible errors include
/// InvalidArgument (terminating kernel process) and NoSuchProcess.
#[verifier::external_body]
pub fn process_manager_terminate(pid: u32) -> (result: TerminateResultModel)
    ensures
        // The result is always one of the defined variants.
        matches!(result, TerminateResultModel::TmOk | TerminateResultModel::TmError { .. }),
        // On failure, the error code is a valid positive error code.
        result.spec_view() matches TerminateOutcomeView::TmError { error_code }
            ==> spec_is_valid_error_code(error_code),
{
    unimplemented!()
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Verified exec model of the `terminate(pm, args)` kernel call.
///
/// # Description
///
/// This function mirrors the original `terminate` control flow:
/// 1. Parse ProcessIdentifier from args.arg0 via try_from.
/// 2. On parse error, return KcallResult::Error(error.code.into()).
/// 3. Call pm.terminate(pid).
/// 4. On success, return KcallResult::ok().
/// 5. On terminate error, return KcallResult::Error(e.code.into()).
///
/// # Parameters
///
/// - `arg0`: The raw u32 argument encoding the target process identifier.
///
/// # Returns
///
/// A KcallResultModel and ghost views of both pipeline step outcomes.
pub fn terminate_model(arg0: u32) -> (ret: (KcallResultModel, Ghost<PidParseOutcomeView>, Ghost<TerminateOutcomeView>))
    ensures
        // The result matches the spec pipeline.
        ret.0.spec_view() == spec_terminate_result(ret.1@, ret.2@),
        // PID parse error path: error is returned immediately.
        !spec_pid_parsed_ok(ret.1@)
            ==> spec_is_error(ret.0.spec_view()),
        // PID parse error path: error code is InvalidArgument.
        !spec_pid_parsed_ok(ret.1@)
            ==> ret.0.spec_view() == TerminateResultView::Error {
                    error_code: ERROR_CODE_INVALID_ARGUMENT()
                },
        // Success path: both steps succeeded.
        spec_is_success(ret.0.spec_view())
            ==> spec_pid_parsed_ok(ret.1@) && spec_terminate_ok(ret.2@),
        // Result is always Success or Error.
        spec_is_success(ret.0.spec_view()) || spec_is_error(ret.0.spec_view()),
        // Success and Error are mutually exclusive.
        !(spec_is_success(ret.0.spec_view()) && spec_is_error(ret.0.spec_view())),
{
    // Step 1: Parse ProcessIdentifier from arg0.
    let pid_result: PidParseResultModel = try_from_process_identifier(arg0);

    // Capture the PID parse outcome as a ghost.
    let ghost pid_view: PidParseOutcomeView = pid_result.spec_view();

    match pid_result {
        PidParseResultModel::PidError { error_code } => {
            // Parse error: return immediately with the error code.
            // The ghost terminate outcome is arbitrary (short-circuit).
            let ghost tm_view: TerminateOutcomeView = TerminateOutcomeView::TmOk;
            proof {
                lemma_pid_parse_error_propagates(error_code as int, tm_view);
                lemma_result_exhaustive(pid_view, tm_view);
            }
            (KcallResultModel::Error { error_code }, Ghost(pid_view), Ghost(tm_view))
        },
        PidParseResultModel::PidOk { pid } => {
            // Step 2: Call ProcessManager::terminate(pid).
            let tm_result: TerminateResultModel = process_manager_terminate(pid);

            // Capture the terminate outcome as a ghost.
            let ghost tm_view: TerminateOutcomeView = tm_result.spec_view();

            proof {
                lemma_result_exhaustive(pid_view, tm_view);
            }

            match tm_result {
                TerminateResultModel::TmOk => {
                    (KcallResultModel::Ok, Ghost(pid_view), Ghost(tm_view))
                },
                TerminateResultModel::TmError { error_code } => {
                    (KcallResultModel::Error { error_code }, Ghost(pid_view), Ghost(tm_view))
                },
            }
        },
    }
}

} // verus!
