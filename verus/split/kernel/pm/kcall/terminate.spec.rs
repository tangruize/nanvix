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
// - ProcessManager state transitions via ghost state threading.

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

/// The kernel process PID (always 0).
///
/// # Description
///
/// The kernel process cannot be terminated. Any attempt to terminate PID 0
/// must return an error.
pub open spec fn KERNEL_PID() -> nat {
    0
}

/// The ErrorCode value for NoSuchProcess (repr(i32) = 3, ESRCH).
pub open spec fn ERROR_CODE_NO_SUCH_PROCESS() -> int {
    3
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

/// Abstract view of the ProcessManager state.
///
/// # Description
///
/// Models the ProcessManager's process table as a set of tracked PIDs,
/// a terminatable subset, and the currently running process (if any).
/// This concrete representation (vs. an empty struct) ensures that
/// pre-state and post-state can be genuinely distinct, making
/// state-transition postconditions satisfiable and non-vacuous.
///
/// - `process_set`: All PIDs known to the PM (ready, suspended,
///   interrupted, zombie, or running).
/// - `terminatable_set`: The subset of PIDs that `pm.terminate` will
///   accept (ready or suspended processes). Interrupted and zombie
///   PIDs are in `process_set` but NOT in `terminatable_set`; the
///   real PM returns `NoSuchProcess` for them because they are not
///   found in the ready/suspended queues.
/// - `running_pid`: The currently executing process (if any). The PM
///   rejects terminate requests for the running process even though
///   it is conceptually "ready".
#[verifier::ext_equal]
pub struct ProcessManagerStateView {
    /// The set of process identifiers tracked by the process manager.
    pub process_set: Set<nat>,
    /// The subset of PIDs for which `pm.terminate` will accept (ready/suspended).
    pub terminatable_set: Set<nat>,
    /// The PID of the currently running process, if any.
    pub running_pid: Option<nat>,
}

//==================================================================================================
// Spec Predicates
//==================================================================================================

/// Whether the process manager state contains a process with the given PID.
///
/// # Description
///
/// Concrete predicate over PM state using set membership. This replaces
/// the previous uninterpreted version to ensure that pre-state and
/// post-state with different process sets are provably distinct, making
/// success-path postconditions satisfiable (not vacuously true).
pub open spec fn spec_pm_has_process(state: ProcessManagerStateView, pid: nat) -> bool {
    state.process_set.contains(pid)
}

/// Whether the given PID is the currently running process.
///
/// # Description
///
/// The PM tracks which process is currently running. Terminating the
/// running process is rejected with InvalidArgument. This predicate
/// checks whether a given PID matches the running process.
pub open spec fn spec_is_running_process(state: ProcessManagerStateView, pid: nat) -> bool {
    state.running_pid == Some(pid)
}

/// Well-formedness predicate for ProcessManager state.
///
/// # Description
///
/// Ensures structural consistency of the PM state:
/// - If a running process exists, its PID must be in the process set.
/// - The kernel PID (0) is always in the process set (it is never removed).
/// - The terminatable set is a subset of the process set.
/// - The kernel PID is never terminatable (it is rejected before queue lookup).
/// - The running PID is never terminatable (it is rejected before queue lookup).
///
/// These invariants prevent inconsistent postconditions in
/// `process_manager_terminate`. Without them, postconditions for different
/// rejection reasons could fire simultaneously.
pub open spec fn spec_pm_wf(state: ProcessManagerStateView) -> bool {
    // Running process must be in the process set.
    (state.running_pid matches Some(pid) ==> state.process_set.contains(pid))
    // Kernel PID is always tracked.
    && state.process_set.contains(KERNEL_PID())
    // Terminatable set is a subset of the process set.
    && state.terminatable_set.subset_of(state.process_set)
    // Kernel PID is never terminatable.
    && !state.terminatable_set.contains(KERNEL_PID())
    // Running PID is never in the terminatable set.
    && (state.running_pid matches Some(pid) ==> !state.terminatable_set.contains(pid))
}

/// Whether a raw u32 value is a valid ProcessIdentifier.
///
/// # Description
///
/// Abstract predicate that characterizes which raw values pass
/// `ProcessIdentifier::try_from`. The pid module provides the concrete
/// interpretation. This module uses it to make the try_from external_body
/// deterministic: for a given input, the parse result is determined by
/// this predicate.
pub uninterp spec fn spec_is_valid_pid(raw: nat) -> bool;

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

/// Spec predicate: whether a terminate operation can succeed on a given state and PID.
///
/// # Description
///
/// A terminate can succeed only if the PID is in the terminatable set
/// (ready or suspended processes). Under `spec_pm_wf`, this implies:
/// 1. The PID exists in the process set (`terminatable_set ⊆ process_set`).
/// 2. The PID is not the kernel process (`KERNEL_PID ∉ terminatable_set`).
/// 3. The PID is not the running process (`running ∉ terminatable_set`).
///
/// This models the three rejection checks in `ProcessManager::terminate`
/// plus the queue-lookup requirement (only ready/suspended are found).
pub open spec fn spec_terminate_possible(state: ProcessManagerStateView, pid: nat) -> bool {
    state.terminatable_set.contains(pid)
}

} // verus!
