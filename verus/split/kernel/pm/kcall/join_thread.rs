// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Join Thread Kernel Call Verification Model
//!
//! Formal verification of the join thread kernel call (`pm::kcall::join_thread`).
//!
//! ## Overview
//!
//! The `join_thread(pid, arg0, arg1)` function joins a thread, blocking until
//! the target thread exits. It implements a 3-step pipeline:
//! 1. Parse `ThreadIdentifier` from `arg0` via `ThreadIdentifier::try_from(u32)`.
//! 2. Call `ProcessManager::join_thread(pid, tid)` — blocks until the target
//!    thread exits, returning its `ExitStatus`.
//! 3. Copy the `ExitStatus` to user space via `copy_to_user` at the address
//!    given by `arg1`.
//!
//! On success, returns `Ok(ExitStatus::ok())`. On failure at any step, returns
//! `Err(SleepError)` — either `Generic(error)` or `Interrupted(Killed)`.
//!
//! ## Verified Properties
//!
//! - **TID parse error propagation**: When `ThreadIdentifier::try_from` fails,
//!   the error code is wrapped in `SleepError::Generic` and returned immediately.
//!   The join and copy steps are skipped (`lemma_tid_parse_error_propagates`,
//!   `lemma_tid_parse_short_circuit`).
//! - **Join generic error propagation**: When `ProcessManager::join_thread` fails
//!   with `SleepError::Generic`, the error propagates as GenericError
//!   (`lemma_join_generic_error_propagates`).
//! - **Join killed propagation**: When `ProcessManager::join_thread` fails with
//!   `SleepError::Interrupted(Killed)`, the result is InterruptedKilled
//!   (`lemma_join_killed_propagates`).
//! - **Copy error propagation**: When `copy_to_user` fails, the error is wrapped
//!   in `SleepError::Generic` and returned (`lemma_copy_error_propagates`).
//! - **Success requires all steps**: The result is Success if and only if all
//!   three pipeline steps succeed (`lemma_success_requires_all_steps`).
//! - **Result exhaustiveness**: Every input combination produces exactly one
//!   result: Success, GenericError, or InterruptedKilled. These are mutually
//!   exclusive (`lemma_result_exhaustive`).
//! - **Success implies ExitStatus::ok()**: On success, the returned exit status
//!   is always `ExitStatus::ok()` (value 0) (`lemma_success_implies_ok_status`).
//! - **Error code preservation**: Error codes from all pipeline steps are
//!   preserved faithfully (`lemma_tid_error_code_preserved`,
//!   `lemma_join_error_code_preserved`, `lemma_copy_error_code_preserved`).
//! - **InvalidArgument for bad TID**: Invalid TID values produce
//!   `ErrorCode::InvalidArgument` (22) (`lemma_invalid_tid_returns_invalid_argument`).
//! - **Error code linkage**: The spec constant `ERROR_CODE_INVALID_ARGUMENT()`
//!   is proven equal to `ErrorCode::InvalidArgument as int`
//!   (`lemma_error_code_matches`).
//! - **Killed only from join**: InterruptedKilled can only originate from the
//!   join_thread step (`lemma_killed_only_from_join`).
//! - **Short-circuit at join**: When join_thread fails, the copy outcome is
//!   irrelevant (`lemma_join_error_short_circuits_copy`).
//! - **Any failure is error**: If any step fails, the overall result is an error
//!   (`lemma_any_failure_is_error`).
//! - **TID identity**: On successful parse, the parsed TID equals the input
//!   argument (`try_from_thread_identifier` postcondition).
//!
//! ## Properties NOT Proven Here (Out of Scope)
//!
//! - "ProcessManager::join_thread eventually returns" (liveness / scheduler).
//! - "The joined thread's exit status is correct" (PM internal invariant).
//! - "copy_to_user writes the correct bytes" (memory management correctness).
//! - "The user-space pointer arg1 is valid" (validated by copy_to_user internally).
//!
//! ## Trust Boundaries
//!
//! - **T1: `ThreadIdentifier::try_from(u32)`**. Parses a raw u32 into a valid
//!   TID. Modeled as `external_body`. Postconditions guarantee identity,
//!   determinism, and InvalidArgument on failure.
//! - **T2: `ProcessManager::join_thread(pid, tid)`**. Blocks until the target
//!   thread exits and returns its ExitStatus. Modeled as `external_body`.
//!   Can return Ok(ExitStatus), Err(Interrupted(Killed)), or Err(Generic(error)).
//! - **T3: `pm::copy_to_user(pm, pid, retval, &status)`**. Copies the
//!   ExitStatus to user space. Modeled as `external_body`. Can return
//!   Ok(()) or Err(error).
//!
//! ## Logging
//!
//! The original code logs with `error!("{error:?}")` on TID parse failure.
//! This logging is not modeled as it has no functional effect.
//!
//! ## API Mapping
//!
//! | Original API                                         | Verified Model                          | Notes          |
//! |------------------------------------------------------|-----------------------------------------|----------------|
//! | `ThreadIdentifier::try_from(arg0)`                   | `try_from_thread_identifier(arg0)`      | external_body  |
//! | `ProcessManager::join_thread(pid, tid)`               | `process_manager_join_thread(pid, tid)` | external_body  |
//! | `pm::copy_to_user(pm, pid, retval, &status)`         | `copy_to_user_exit_status(…)`           | external_body  |
//! | `pub unsafe fn join_thread(pid, arg0, arg1)`         | `join_thread_model(arg0, …)`            | Fully verified  |

use crate::libs::error::ErrorCode;
use vstd::prelude::*;

// Include specifications.
include!("join_thread.spec.rs");

// Include proofs.
include!("join_thread.proof.rs");

verus! {

//==================================================================================================
// Dependency Models (External Bodies)
//==================================================================================================

/// Model of the TID parsing result for verification.
///
/// # Description
///
/// Represents the two possible outcomes from `ThreadIdentifier::try_from(u32)`:
/// - `TidOk`: The raw u32 was a valid TID.
/// - `TidError`: The raw u32 was invalid; carries the error code.
pub enum TidParseResultModel {
    /// ThreadIdentifier::try_from succeeded.
    TidOk { tid: u32 },
    /// ThreadIdentifier::try_from failed with an error code.
    TidError { error_code: i32 },
}

impl TidParseResultModel {
    /// Spec function: converts to the abstract TidParseOutcomeView.
    pub open spec fn spec_view(&self) -> TidParseOutcomeView {
        match self {
            TidParseResultModel::TidOk { tid } => {
                TidParseOutcomeView::TidOk { tid: *tid as nat }
            },
            TidParseResultModel::TidError { error_code } => {
                TidParseOutcomeView::TidError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the ProcessManager::join_thread result for verification.
///
/// # Description
///
/// Represents the three possible outcomes from `ProcessManager::join_thread(pid, tid)`:
/// - `JtOk`: Thread joined successfully; carries the exit status value.
/// - `JtInterruptedKilled`: The joining thread was killed while waiting.
/// - `JtError`: A generic error occurred; carries the error code.
pub enum JoinThreadResultModel {
    /// ProcessManager::join_thread returned Ok(exit_status).
    JtOk { exit_status: u32 },
    /// ProcessManager::join_thread returned Err(SleepError::Interrupted(Killed)).
    JtInterruptedKilled,
    /// ProcessManager::join_thread returned Err(SleepError::Generic(error)).
    JtError { error_code: i32 },
}

impl JoinThreadResultModel {
    /// Spec function: converts to the abstract JoinThreadOutcomeView.
    pub open spec fn spec_view(&self) -> JoinThreadOutcomeView {
        match self {
            JoinThreadResultModel::JtOk { exit_status } => {
                JoinThreadOutcomeView::JtOk { exit_status: *exit_status as int }
            },
            JoinThreadResultModel::JtInterruptedKilled => {
                JoinThreadOutcomeView::JtInterruptedKilled
            },
            JoinThreadResultModel::JtError { error_code } => {
                JoinThreadOutcomeView::JtError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the copy_to_user result for verification.
///
/// # Description
///
/// Represents the two possible outcomes from `pm::copy_to_user`:
/// - `CopyOk`: The copy succeeded.
/// - `CopyError`: The copy failed; carries the error code.
pub enum CopyToUserResultModel {
    /// copy_to_user returned Ok(()).
    CopyOk,
    /// copy_to_user returned Err(error).
    CopyError { error_code: i32 },
}

impl CopyToUserResultModel {
    /// Spec function: converts to the abstract CopyToUserOutcomeView.
    pub open spec fn spec_view(&self) -> CopyToUserOutcomeView {
        match self {
            CopyToUserResultModel::CopyOk => CopyToUserOutcomeView::CopyOk,
            CopyToUserResultModel::CopyError { error_code } => {
                CopyToUserOutcomeView::CopyError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the kcall final result for verification.
///
/// # Description
///
/// Represents the final result: Ok(ExitStatus), Err(SleepError::Generic),
/// or Err(SleepError::Interrupted(Killed)).
pub enum JoinThreadKcallResultModel {
    /// Ok(ExitStatus::ok()).
    Ok { exit_status: u32 },
    /// Err(SleepError::Generic(error)).
    GenericError { error_code: i32 },
    /// Err(SleepError::Interrupted(Killed)).
    InterruptedKilled,
}

impl JoinThreadKcallResultModel {
    /// Spec function: converts to the abstract JoinThreadResultView.
    pub open spec fn spec_view(&self) -> JoinThreadResultView {
        match self {
            JoinThreadKcallResultModel::Ok { exit_status } => {
                JoinThreadResultView::Success { exit_status: *exit_status as int }
            },
            JoinThreadKcallResultModel::GenericError { error_code } => {
                JoinThreadResultView::GenericError { error_code: *error_code as int }
            },
            JoinThreadKcallResultModel::InterruptedKilled => {
                JoinThreadResultView::InterruptedKilled
            },
        }
    }
}

//==================================================================================================
// External Body Functions (Trust Boundaries)
//==================================================================================================

/// Trust Boundary T1: Models `ThreadIdentifier::try_from(arg0)`.
///
/// # Description
///
/// Parses a raw u32 into a ThreadIdentifier. Returns TidOk on success or
/// TidError with ErrorCode::InvalidArgument on failure.
///
/// Postconditions capture:
/// - **Identity**: On success, the parsed TID equals the input `arg0`.
/// - **Determinism**: The result is determined by `spec_is_valid_tid(arg0)`.
/// - **Error code**: On failure, the error code is always InvalidArgument.
#[verifier::external_body]
pub fn try_from_thread_identifier(arg0: u32) -> (result: TidParseResultModel)
    ensures
        // Identity: on success, parsed TID equals input.
        result.spec_view() matches TidParseOutcomeView::TidOk { tid }
            ==> tid == arg0 as nat,
        // Determinism: result is determined by the validity predicate.
        spec_is_valid_tid(arg0 as nat)
            ==> matches!(result, TidParseResultModel::TidOk { .. }),
        !spec_is_valid_tid(arg0 as nat)
            ==> matches!(result, TidParseResultModel::TidError { .. }),
        // On failure, the error code is always InvalidArgument.
        result.spec_view() matches TidParseOutcomeView::TidError { error_code }
            ==> error_code == ERROR_CODE_INVALID_ARGUMENT(),
        // The result is always one of the defined variants.
        matches!(result, TidParseResultModel::TidOk { .. } | TidParseResultModel::TidError { .. }),
{
    unimplemented!()
}

/// Trust Boundary T2: Models `ProcessManager::join_thread(pid, tid)`.
///
/// # Description
///
/// Blocks until the target thread (identified by tid in process pid) exits,
/// then returns its ExitStatus. This is a blocking call that may:
/// - Succeed with the target thread's exit status.
/// - Fail with SleepError::Interrupted(Killed) if the joining thread is killed.
/// - Fail with SleepError::Generic(error) for other errors (e.g., no such thread).
///
/// This kcall module does NOT verify:
/// - That join_thread eventually returns (liveness).
/// - That the exit status is correct (PM internal invariant).
/// - That the thread exists (PM validates this).
#[verifier::external_body]
pub fn process_manager_join_thread(pid: u32, tid: u32) -> (result: JoinThreadResultModel)
    ensures
        // The result is always one of the defined variants.
        matches!(result, JoinThreadResultModel::JtOk { .. }
            | JoinThreadResultModel::JtInterruptedKilled
            | JoinThreadResultModel::JtError { .. }),
        // On generic error, the error code is a valid positive value.
        result.spec_view() matches JoinThreadOutcomeView::JtError { error_code }
            ==> spec_is_valid_error_code(error_code),
{
    unimplemented!()
}

/// Trust Boundary T3: Models `pm::copy_to_user(pm, pid, retval, &status)`.
///
/// # Description
///
/// Copies the ExitStatus to user space at the address pointed to by retval.
/// Returns Ok(()) on success or Err(error) on failure.
///
/// The error is wrapped in SleepError::Generic by the caller via
/// `.map_err(SleepError::Generic)`.
#[verifier::external_body]
pub fn copy_to_user_exit_status(pid: u32, retval_addr: u32, exit_status: u32) -> (result: CopyToUserResultModel)
    ensures
        // The result is always one of the defined variants.
        matches!(result, CopyToUserResultModel::CopyOk | CopyToUserResultModel::CopyError { .. }),
        // On error, the error code is a valid positive value.
        result.spec_view() matches CopyToUserOutcomeView::CopyError { error_code }
            ==> spec_is_valid_error_code(error_code),
{
    unimplemented!()
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Verified exec model of the `join_thread(pid, arg0, arg1)` kernel call.
///
/// # Description
///
/// This function mirrors the original `join_thread` control flow:
/// 1. Parse ThreadIdentifier from arg0 via try_from.
/// 2. Call ProcessManager::join_thread(pid, tid) — blocks until target exits.
/// 3. Copy exit status to user space via copy_to_user at arg1 address.
/// 4. Return Ok(ExitStatus::ok()) on success.
///
/// # Parameters
///
/// - `pid`: Process identifier (passed through from kcall args).
/// - `arg0`: Raw u32 encoding the target thread identifier.
/// - `arg1`: Raw u32 encoding the user-space address for the exit status.
///
/// # Returns
///
/// A tuple of:
/// - `JoinThreadKcallResultModel`: The kcall result.
/// - `Ghost<TidParseOutcomeView>`: Ghost TID parse outcome.
/// - `Ghost<JoinThreadOutcomeView>`: Ghost join outcome.
/// - `Ghost<CopyToUserOutcomeView>`: Ghost copy outcome.
pub fn join_thread_model(
    pid: u32,
    arg0: u32,
    arg1: u32,
) -> (ret: (JoinThreadKcallResultModel, Ghost<TidParseOutcomeView>, Ghost<JoinThreadOutcomeView>, Ghost<CopyToUserOutcomeView>))
    ensures
        // The result matches the spec pipeline.
        ret.0.spec_view() == spec_join_thread_result(ret.1@, ret.2@, ret.3@),
        // TID identity: on successful parse, parsed TID equals input.
        ret.1@ matches TidParseOutcomeView::TidOk { tid }
            ==> tid == arg0 as nat,
        // TID parse error path: error is returned immediately.
        !spec_tid_parsed_ok(ret.1@)
            ==> spec_is_error(ret.0.spec_view()),
        // TID parse error path: error code is InvalidArgument.
        !spec_tid_parsed_ok(ret.1@)
            ==> ret.0.spec_view() == (JoinThreadResultView::GenericError {
                    error_code: ERROR_CODE_INVALID_ARGUMENT()
                }),
        // Join error path: error is propagated.
        spec_tid_parsed_ok(ret.1@) && !spec_join_ok(ret.2@)
            ==> spec_is_error(ret.0.spec_view()),
        // Copy error path: error is wrapped in GenericError.
        spec_tid_parsed_ok(ret.1@) && spec_join_ok(ret.2@) && !spec_copy_ok(ret.3@)
            ==> spec_is_generic_error(ret.0.spec_view()),
        // Success path: all steps succeeded.
        spec_is_success(ret.0.spec_view())
            ==> spec_all_steps_passed(ret.1@, ret.2@, ret.3@),
        // Success path: result carries ExitStatus::ok().
        spec_is_success(ret.0.spec_view())
            ==> ret.0.spec_view() == (JoinThreadResultView::Success {
                    exit_status: EXIT_STATUS_OK()
                }),
        // Result is always one of the three categories.
        spec_is_success(ret.0.spec_view()) || spec_is_error(ret.0.spec_view()),
        // Success and error are mutually exclusive.
        !(spec_is_success(ret.0.spec_view()) && spec_is_error(ret.0.spec_view())),
{
    // Step 1: Parse ThreadIdentifier from arg0.
    let tid_result: TidParseResultModel = try_from_thread_identifier(arg0);

    // Capture the TID parse outcome as a ghost.
    let ghost tid_view: TidParseOutcomeView = tid_result.spec_view();

    match tid_result {
        TidParseResultModel::TidError { error_code } => {
            // Parse error: return Err(SleepError::Generic(error)).
            // Dummy join and copy outcomes: irrelevant due to short-circuit.
            let ghost jt_view: JoinThreadOutcomeView = JoinThreadOutcomeView::JtOk { exit_status: 0 };
            let ghost cp_view: CopyToUserOutcomeView = CopyToUserOutcomeView::CopyOk;
            proof {
                lemma_tid_parse_error_propagates(error_code as int, jt_view, cp_view);
                lemma_result_exhaustive(tid_view, jt_view, cp_view);
            }
            (JoinThreadKcallResultModel::GenericError { error_code }, Ghost(tid_view), Ghost(jt_view), Ghost(cp_view))
        },
        TidParseResultModel::TidOk { tid } => {
            // Step 2: Call ProcessManager::join_thread(pid, tid).
            let jt_result: JoinThreadResultModel = process_manager_join_thread(pid, tid);

            // Capture the join outcome as a ghost.
            let ghost jt_view: JoinThreadOutcomeView = jt_result.spec_view();

            match jt_result {
                JoinThreadResultModel::JtError { error_code } => {
                    // Join failed with generic error: propagate.
                    let ghost cp_view: CopyToUserOutcomeView = CopyToUserOutcomeView::CopyOk;
                    proof {
                        lemma_join_generic_error_propagates(tid as nat, error_code as int, cp_view);
                        lemma_result_exhaustive(tid_view, jt_view, cp_view);
                    }
                    (JoinThreadKcallResultModel::GenericError { error_code }, Ghost(tid_view), Ghost(jt_view), Ghost(cp_view))
                },
                JoinThreadResultModel::JtInterruptedKilled => {
                    // Join failed: thread was killed.
                    let ghost cp_view: CopyToUserOutcomeView = CopyToUserOutcomeView::CopyOk;
                    proof {
                        lemma_join_killed_propagates(tid as nat, cp_view);
                        lemma_result_exhaustive(tid_view, jt_view, cp_view);
                    }
                    (JoinThreadKcallResultModel::InterruptedKilled, Ghost(tid_view), Ghost(jt_view), Ghost(cp_view))
                },
                JoinThreadResultModel::JtOk { exit_status } => {
                    // Step 3: Copy exit status to user space.
                    let cp_result: CopyToUserResultModel = copy_to_user_exit_status(pid, arg1, exit_status);

                    // Capture the copy outcome as a ghost.
                    let ghost cp_view: CopyToUserOutcomeView = cp_result.spec_view();

                    match cp_result {
                        CopyToUserResultModel::CopyError { error_code } => {
                            // Copy failed: wrap in SleepError::Generic.
                            proof {
                                lemma_copy_error_propagates(tid as nat, exit_status as int, error_code as int);
                                lemma_result_exhaustive(tid_view, jt_view, cp_view);
                            }
                            (JoinThreadKcallResultModel::GenericError { error_code }, Ghost(tid_view), Ghost(jt_view), Ghost(cp_view))
                        },
                        CopyToUserResultModel::CopyOk => {
                            // All steps succeeded. Return Ok(ExitStatus::ok()).
                            proof {
                                lemma_result_exhaustive(tid_view, jt_view, cp_view);
                            }
                            (JoinThreadKcallResultModel::Ok { exit_status: 0u32 }, Ghost(tid_view), Ghost(jt_view), Ghost(cp_view))
                        },
                    }
                },
            }
        },
    }
}

} // verus!
