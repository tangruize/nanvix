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
//! - **Error code linkage**: The spec constants `ERROR_CODE_INVALID_ARGUMENT()`
//!   and `ERROR_CODE_NO_SUCH_PROCESS()` are proven equal to their respective
//!   `ErrorCode` discriminants (`lemma_error_code_matches`,
//!   `lemma_no_such_process_error_code_matches`).
//! - **Success implies valid PID**: If the result is Success, then the PID was
//!   valid and the process existed (`lemma_success_implies_valid_pid`).
//! - **PID identity**: On successful parse, the parsed PID equals the input
//!   argument (`try_from_process_identifier` postcondition).
//! - **Kernel PID protection**: Terminating PID 0 (kernel process) always fails
//!   (`process_manager_terminate` postcondition + `lemma_kernel_pid_always_fails`).
//! - **Running PID protection**: Terminating the currently running process always
//!   fails with InvalidArgument (`process_manager_terminate` postcondition +
//!   `lemma_running_pid_returns_error`).
//! - **NoSuchProcess for missing PID**: Terminating a non-existent non-kernel PID
//!   produces `ErrorCode::NoSuchProcess` (3)
//!   (`process_manager_terminate` postcondition +
//!   `lemma_nonexistent_pid_returns_no_such_process`).
//! - **PM state preservation on error**: PM state is unchanged on any error path,
//!   proven from the pipeline structure (`terminate_model` postcondition +
//!   `lemma_state_unchanged_on_error`).
//! - **Terminability precondition**: Success implies the PID was terminatable
//!   in the pre-state: existed, not kernel, not running
//!   (`lemma_success_requires_terminatable`).
//! - **PM state well-formedness**: The `spec_pm_wf` invariant ensures the
//!   running PID is always in the process set and the kernel PID is tracked.
//!   This prevents inconsistent postconditions on the PM external_body
//!   (`lemma_wf_running_implies_exists`, `lemma_wf_prevents_inconsistency`).
//!   Well-formedness is required as a precondition and guaranteed as a
//!   postcondition of both `process_manager_terminate` and `terminate_model`.
//!
//! ## Properties NOT Proven Here (Out of Scope)
//!
//! - "PID is removed from process table after terminate" — The real PM may
//!   keep the PID alive (ready process with runnable threads is resumed).
//!   PID removal depends on process lifecycle state, not modeled here.
//! - "Resources held by the terminated process are freed" (resource management).
//! - "The terminated process's threads are cleaned up" (thread management).
//! - "Scheduler queues are consistent after terminate" (scheduler invariant).
//!
//! These are internal PM invariants verified in the ProcessManager module.
//!
//! ## Trust Boundaries
//!
//! - **T1: `ProcessIdentifier::try_from(u32)`**. Parses a raw u32 into a valid
//!   PID. Modeled as `external_body` returning a `PidParseResultModel`. The
//!   parsing logic is verified in the sys/pid module. Postconditions guarantee:
//!   identity (parsed PID == input), determinism (result determined by
//!   `spec_is_valid_pid`), and InvalidArgument on failure.
//! - **T2: `ProcessManager::terminate(pid)`**. Terminates the process. Modeled
//!   as `external_body` taking ghost PM pre-state and returning ghost post-state.
//!   Postconditions guarantee: kernel PID (0) rejection, running PID rejection,
//!   non-existent PID rejection (NoSuchProcess), PID existence requirement for
//!   success, and state preservation on error. PID removal is NOT claimed (the
//!   real PM may resume the process). Trusted postconditions are modeled from
//!   the PM implementation (mod.rs:1036-1078); the PM module's own verification
//!   covers the implementation side.
//!
//! ## Logging
//!
//! The original code logs with `error!("{error:?}")` on PID parse failure before
//! returning. This logging is not modeled because it has no functional effect on
//! the return value or PM state. The verification scope is limited to functional
//! correctness of the dispatch pipeline.
//!
//! ## API Mapping
//!
//! | Original API                              | Verified Model                      | Notes           |
//! |-------------------------------------------|-------------------------------------|-----------------|
//! | `ProcessIdentifier::try_from(args.arg0)`  | `try_from_process_identifier(arg0)` | external_body   |
//! | `pm.terminate(pid)`                       | `process_manager_terminate(pid, …)` | external_body   |
//! | `pub fn terminate(pm, args) -> KcallResult`| `terminate_model(arg0, …)`         | Fully verified  |

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
///
/// Postconditions capture:
/// - **Identity**: On success, the parsed PID equals the input `arg0`.
/// - **Determinism**: The result is determined by `spec_is_valid_pid(arg0)`.
/// - **Error code**: On failure, the error code is always InvalidArgument.
#[verifier::external_body]
pub fn try_from_process_identifier(arg0: u32) -> (result: PidParseResultModel)
    ensures
        // Identity: on success, parsed PID equals input.
        result.spec_view() matches PidParseOutcomeView::PidOk { pid }
            ==> pid == arg0 as nat,
        // Determinism: result is determined by the validity predicate.
        spec_is_valid_pid(arg0 as nat)
            ==> matches!(result, PidParseResultModel::PidOk { .. }),
        !spec_is_valid_pid(arg0 as nat)
            ==> matches!(result, PidParseResultModel::PidError { .. }),
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
/// Terminates the process identified by `pid`. Takes a ghost PM pre-state
/// and returns a ghost PM post-state alongside the result. Postconditions
/// capture state transition properties:
/// - **Kernel PID rejection**: PID 0 (kernel process) always fails with InvalidArgument.
/// - **Running PID rejection**: Running process always fails with InvalidArgument.
/// - **Non-existent PID rejection**: PID not in process set fails with NoSuchProcess.
/// - **State preservation on error**: PM state is unchanged on failure.
/// - **PID existence requirement**: Success requires PID to exist in pre-state.
/// - **Error validity**: Error codes are always valid positive values.
///
/// ## PID Removal NOT Claimed
///
/// The real `ProcessManager::terminate` does NOT necessarily remove the PID
/// from the process table on success. A ready process with runnable threads
/// is interrupted and then resumed back to the ready queue (PID stays).
/// Only processes with no runnable threads become zombies. Therefore, we
/// do NOT claim `!spec_pm_has_process(post, pid)` on success.
///
/// ## Trusted Postconditions
///
/// These postconditions are trusted assumptions modeled from the PM
/// implementation (src/kernel/src/pm/process/manager/mod.rs:1036-1078).
/// The PM module's own verification (in the `process_manager` verified
/// module) covers the implementation side of these contracts. The
/// `spec_pm_wf` precondition ensures structural consistency, preventing
/// contradictory postconditions (e.g., running + non-existent for the
/// same PID). This is a refinement contract: the PM module must maintain
/// well-formedness as an invariant, and this kcall module requires it.
#[verifier::external_body]
pub fn process_manager_terminate(
    pid: u32,
    Ghost(pm_pre): Ghost<ProcessManagerStateView>,
) -> (ret: (TerminateResultModel, Ghost<ProcessManagerStateView>))
    requires
        // The pre-state must be well-formed.
        spec_pm_wf(pm_pre),
    ensures
        // Kernel PID (0) always fails.
        pid as nat == KERNEL_PID()
            ==> matches!(ret.0, TerminateResultModel::TmError { .. }),
        // Kernel PID error code is InvalidArgument.
        pid as nat == KERNEL_PID()
            ==> (ret.0.spec_view() matches TerminateOutcomeView::TmError { error_code }
                && error_code == ERROR_CODE_INVALID_ARGUMENT()),
        // Running PID always fails.
        spec_is_running_process(pm_pre, pid as nat)
            ==> matches!(ret.0, TerminateResultModel::TmError { .. }),
        // Running PID error code is InvalidArgument.
        spec_is_running_process(pm_pre, pid as nat)
            ==> (ret.0.spec_view() matches TerminateOutcomeView::TmError { error_code }
                && error_code == ERROR_CODE_INVALID_ARGUMENT()),
        // Non-existent non-kernel PID returns NoSuchProcess.
        pid as nat != KERNEL_PID() && !spec_pm_has_process(pm_pre, pid as nat)
            ==> (ret.0.spec_view() matches TerminateOutcomeView::TmError { error_code }
                && error_code == ERROR_CODE_NO_SUCH_PROCESS()),
        // On success: PID existed in pre-state.
        ret.0.spec_view() == TerminateOutcomeView::TmOk
            ==> spec_pm_has_process(pm_pre, pid as nat),
        // Frame: on success, no new PIDs are created (subset).
        ret.0.spec_view() == TerminateOutcomeView::TmOk
            ==> ret.1@.process_set.subset_of(pm_pre.process_set),
        // Frame: on success, all PIDs other than the target are unchanged.
        ret.0.spec_view() == TerminateOutcomeView::TmOk
            ==> forall|p: nat| p != pid as nat ==>
                (spec_pm_has_process(pm_pre, p) <==> spec_pm_has_process(ret.1@, p)),
        // On error: state is unchanged.
        ret.0.spec_view() matches TerminateOutcomeView::TmError { .. }
            ==> ret.1@ == pm_pre,
        // PID must exist for success.
        !spec_pm_has_process(pm_pre, pid as nat)
            ==> matches!(ret.0, TerminateResultModel::TmError { .. }),
        // On failure, the error code is a valid positive error code.
        ret.0.spec_view() matches TerminateOutcomeView::TmError { error_code }
            ==> spec_is_valid_error_code(error_code),
        // The result is always one of the defined variants.
        matches!(ret.0, TerminateResultModel::TmOk | TerminateResultModel::TmError { .. }),
        // The post-state is well-formed.
        spec_pm_wf(ret.1@),
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
/// The model threads ghost PM state to enable proving state transition
/// properties: PID removal on success, state preservation on error.
///
/// # Parameters
///
/// - `arg0`: The raw u32 argument encoding the target process identifier.
/// - `Ghost(pm_pre)`: Ghost PM state before the call.
///
/// # Returns
///
/// A tuple of:
/// - `KcallResultModel`: The kcall result.
/// - `Ghost<PidParseOutcomeView>`: Ghost PID parse outcome.
/// - `Ghost<TerminateOutcomeView>`: Ghost terminate outcome.
/// - `Ghost<ProcessManagerStateView>`: Ghost PM post-state.
pub fn terminate_model(
    arg0: u32,
    Ghost(pm_pre): Ghost<ProcessManagerStateView>,
) -> (ret: (KcallResultModel, Ghost<PidParseOutcomeView>, Ghost<TerminateOutcomeView>, Ghost<ProcessManagerStateView>))
    requires
        // The pre-state must be well-formed.
        spec_pm_wf(pm_pre),
    ensures
        // The result matches the spec pipeline.
        ret.0.spec_view() == spec_terminate_result(ret.1@, ret.2@),
        // PID identity: on successful parse, parsed PID equals input.
        ret.1@ matches PidParseOutcomeView::PidOk { pid }
            ==> pid == arg0 as nat,
        // PID parse error path: error is returned immediately.
        !spec_pid_parsed_ok(ret.1@)
            ==> spec_is_error(ret.0.spec_view()),
        // PID parse error path: error code is InvalidArgument.
        !spec_pid_parsed_ok(ret.1@)
            ==> ret.0.spec_view() == (TerminateResultView::Error {
                    error_code: ERROR_CODE_INVALID_ARGUMENT()
                }),
        // PID parse error path: PM state is unchanged (PM was never called).
        !spec_pid_parsed_ok(ret.1@)
            ==> ret.3@ == pm_pre,
        // Terminate error path: PM state is unchanged.
        spec_pid_parsed_ok(ret.1@) && !spec_terminate_ok(ret.2@)
            ==> ret.3@ == pm_pre,
        // Success path: both steps succeeded.
        spec_is_success(ret.0.spec_view())
            ==> spec_pid_parsed_ok(ret.1@) && spec_terminate_ok(ret.2@),
        // Success path: PID existed in pre-state.
        spec_is_success(ret.0.spec_view())
            ==> spec_pm_has_process(pm_pre, arg0 as nat),
        // Success path: PID was terminatable in the pre-state.
        spec_is_success(ret.0.spec_view())
            ==> spec_terminate_possible(pm_pre, arg0 as nat),
        // Kernel PID: if arg0 == 0 and parses successfully, result is error.
        arg0 as nat == KERNEL_PID() && spec_pid_parsed_ok(ret.1@)
            ==> spec_is_error(ret.0.spec_view()),
        // Running PID: if arg0 is the running process and parses, result is error.
        spec_is_running_process(pm_pre, arg0 as nat) && spec_pid_parsed_ok(ret.1@)
            ==> spec_is_error(ret.0.spec_view()),
        // Result is always Success or Error.
        spec_is_success(ret.0.spec_view()) || spec_is_error(ret.0.spec_view()),
        // Success and Error are mutually exclusive.
        !(spec_is_success(ret.0.spec_view()) && spec_is_error(ret.0.spec_view())),
        // Post-state is well-formed.
        spec_pm_wf(ret.3@),
{
    // Step 1: Parse ProcessIdentifier from arg0.
    let pid_result: PidParseResultModel = try_from_process_identifier(arg0);

    // Capture the PID parse outcome as a ghost.
    let ghost pid_view: PidParseOutcomeView = pid_result.spec_view();

    match pid_result {
        PidParseResultModel::PidError { error_code } => {
            // Parse error: return immediately with the error code.
            // PM was never called, so state is unchanged.
            let ghost tm_view: TerminateOutcomeView = TerminateOutcomeView::TmOk;
            proof {
                lemma_pid_parse_error_propagates(error_code as int, tm_view);
                lemma_result_exhaustive(pid_view, tm_view);
            }
            (KcallResultModel::Error { error_code }, Ghost(pid_view), Ghost(tm_view), Ghost(pm_pre))
        },
        PidParseResultModel::PidOk { pid } => {
            // Step 2: Call ProcessManager::terminate(pid) with ghost state.
            let (tm_result, Ghost(pm_post)): (TerminateResultModel, Ghost<ProcessManagerStateView>) =
                process_manager_terminate(pid, Ghost(pm_pre));

            // Capture the terminate outcome as a ghost.
            let ghost tm_view: TerminateOutcomeView = tm_result.spec_view();

            proof {
                lemma_result_exhaustive(pid_view, tm_view);
            }

            match tm_result {
                TerminateResultModel::TmOk => {
                    (KcallResultModel::Ok, Ghost(pid_view), Ghost(tm_view), Ghost(pm_post))
                },
                TerminateResultModel::TmError { error_code } => {
                    (KcallResultModel::Error { error_code }, Ghost(pid_view), Ghost(tm_view), Ghost(pm_post))
                },
            }
        },
    }
}

} // verus!
