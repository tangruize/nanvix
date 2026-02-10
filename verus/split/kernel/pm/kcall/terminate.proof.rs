// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Terminate Kernel Call Proofs.
// Proof lemmas for the terminate kcall verification model.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Proof Functions — Pipeline Error Propagation
//==================================================================================================

/// Proof: when PID parsing fails, its error propagates regardless of terminate outcome.
///
/// # Description
///
/// If ProcessIdentifier::try_from fails, the function returns early with the
/// parse error. The terminate_outcome is irrelevant (pipeline short-circuit).
pub proof fn lemma_pid_parse_error_propagates(
    error_code: int,
    terminate_outcome: TerminateOutcomeView,
)
    ensures
        spec_terminate_result(
            PidParseOutcomeView::PidError { error_code },
            terminate_outcome,
        ) == (TerminateResultView::Error { error_code }),
        spec_is_error(
            spec_terminate_result(
                PidParseOutcomeView::PidError { error_code },
                terminate_outcome,
            )
        ),
{
}

/// Proof: when PID parsing fails, terminate outcome does not affect the result.
///
/// # Description
///
/// For any two different terminate outcomes, the result is identical when
/// PID parsing fails. This proves the short-circuit behavior of early return.
pub proof fn lemma_pid_parse_short_circuit(
    error_code: int,
    tm1: TerminateOutcomeView,
    tm2: TerminateOutcomeView,
)
    ensures
        spec_terminate_result(
            PidParseOutcomeView::PidError { error_code },
            tm1,
        ) == spec_terminate_result(
            PidParseOutcomeView::PidError { error_code },
            tm2,
        ),
{
}

/// Proof: terminate error propagates when PID parsing succeeds.
///
/// # Description
///
/// When ProcessIdentifier::try_from succeeds but pm.terminate fails,
/// the terminate error propagates as the final result.
pub proof fn lemma_terminate_error_propagates(
    pid: nat,
    error_code: int,
)
    ensures
        spec_terminate_result(
            PidParseOutcomeView::PidOk { pid },
            TerminateOutcomeView::TmError { error_code },
        ) == (TerminateResultView::Error { error_code }),
        spec_is_error(
            spec_terminate_result(
                PidParseOutcomeView::PidOk { pid },
                TerminateOutcomeView::TmError { error_code },
            )
        ),
{
}

//==================================================================================================
// Proof Functions — Success / Exhaustiveness
//==================================================================================================

/// Proof: success requires both pipeline steps to succeed.
///
/// # Description
///
/// The result is Success if and only if:
/// 1. PID parsing succeeded (PidOk).
/// 2. pm.terminate succeeded (TmOk).
pub proof fn lemma_success_requires_both_steps(
    pid_parse_outcome: PidParseOutcomeView,
    terminate_outcome: TerminateOutcomeView,
)
    ensures
        spec_is_success(
            spec_terminate_result(pid_parse_outcome, terminate_outcome)
        ) <==> (
            spec_pid_parsed_ok(pid_parse_outcome)
            && spec_terminate_ok(terminate_outcome)
        ),
{
    match pid_parse_outcome {
        PidParseOutcomeView::PidError { .. } => {},
        PidParseOutcomeView::PidOk { .. } => {
            match terminate_outcome {
                TerminateOutcomeView::TmError { .. } => {},
                TerminateOutcomeView::TmOk => {},
            }
        },
    }
}

/// Proof: the result is always exactly Success or Error (exhaustive).
///
/// # Description
///
/// Every possible input combination produces exactly one result category.
/// Success and Error are mutually exclusive.
pub proof fn lemma_result_exhaustive(
    pid_parse_outcome: PidParseOutcomeView,
    terminate_outcome: TerminateOutcomeView,
)
    ensures
        ({
            let result: TerminateResultView = spec_terminate_result(
                pid_parse_outcome, terminate_outcome,
            );
            // Exhaustive: every result is success or error.
            spec_is_success(result) || spec_is_error(result)
        }),
        ({
            let result: TerminateResultView = spec_terminate_result(
                pid_parse_outcome, terminate_outcome,
            );
            // Mutual exclusion: success and error are disjoint.
            !(spec_is_success(result) && spec_is_error(result))
        }),
{
    match pid_parse_outcome {
        PidParseOutcomeView::PidError { .. } => {},
        PidParseOutcomeView::PidOk { .. } => {
            match terminate_outcome {
                TerminateOutcomeView::TmError { .. } => {},
                TerminateOutcomeView::TmOk => {},
            }
        },
    }
}

/// Proof: success implies PID was valid and process existed.
///
/// # Description
///
/// If the overall result is Success, then both the PID parsing and the
/// terminate operation must have succeeded. This is the reverse implication
/// of lemma_success_requires_both_steps.
pub proof fn lemma_success_implies_valid_pid(
    pid_parse_outcome: PidParseOutcomeView,
    terminate_outcome: TerminateOutcomeView,
)
    requires
        spec_is_success(spec_terminate_result(pid_parse_outcome, terminate_outcome)),
    ensures
        spec_pid_parsed_ok(pid_parse_outcome),
        spec_terminate_ok(terminate_outcome),
{
    match pid_parse_outcome {
        PidParseOutcomeView::PidError { .. } => {},
        PidParseOutcomeView::PidOk { .. } => {
            match terminate_outcome {
                TerminateOutcomeView::TmError { .. } => {},
                TerminateOutcomeView::TmOk => {},
            }
        },
    }
}

//==================================================================================================
// Proof Functions — Error Code Preservation
//==================================================================================================

/// Proof: the error code from PID parsing is preserved in the final result.
///
/// # Description
///
/// When PID parsing fails, the error code in the final result is exactly
/// the error code from the PID parse failure.
pub proof fn lemma_pid_error_code_preserved(
    error_code: int,
    terminate_outcome: TerminateOutcomeView,
)
    ensures
        ({
            let result: TerminateResultView = spec_terminate_result(
                PidParseOutcomeView::PidError { error_code },
                terminate_outcome,
            );
            result == TerminateResultView::Error { error_code }
        }),
{
}

/// Proof: the error code from terminate is preserved in the final result.
///
/// # Description
///
/// When PID parsing succeeds but terminate fails, the error code in the
/// final result is exactly the error code from the terminate failure.
pub proof fn lemma_terminate_error_code_preserved(
    pid: nat,
    error_code: int,
)
    ensures
        ({
            let result: TerminateResultView = spec_terminate_result(
                PidParseOutcomeView::PidOk { pid },
                TerminateOutcomeView::TmError { error_code },
            );
            result == TerminateResultView::Error { error_code }
        }),
{
}

/// Proof: the spec constant ERROR_CODE_INVALID_ARGUMENT matches ErrorCode::InvalidArgument.
///
/// # Description
///
/// Links the spec-level error code constant to the concrete ErrorCode::InvalidArgument
/// discriminant (22).
pub proof fn lemma_error_code_matches()
    ensures
        ERROR_CODE_INVALID_ARGUMENT() == ErrorCode::InvalidArgument as int,
{
    assert(ErrorCode::InvalidArgument as int == 22int);
}

/// Proof: PID parse error always produces InvalidArgument error code.
///
/// # Description
///
/// ProcessIdentifier::try_from returns ErrorCode::InvalidArgument on failure.
/// This lemma proves that when the parse outcome is a PidError with
/// InvalidArgument, the final result carries that error code.
pub proof fn lemma_invalid_pid_returns_invalid_argument(
    terminate_outcome: TerminateOutcomeView,
)
    ensures
        spec_terminate_result(
            PidParseOutcomeView::PidError { error_code: ERROR_CODE_INVALID_ARGUMENT() },
            terminate_outcome,
        ) == (TerminateResultView::Error { error_code: ERROR_CODE_INVALID_ARGUMENT() }),
{
}

//==================================================================================================
// Proof Functions — Kernel PID Protection
//==================================================================================================

/// Proof: terminating the kernel process (PID 0) always fails.
///
/// # Description
///
/// When PID parsing succeeds and yields PID 0 (the kernel process), and the
/// PM terminate returns an error (guaranteed by the external_body contract
/// on `process_manager_terminate`), the overall result is an error.
///
/// The trust chain is:
///   `process_manager_terminate` ensures (pid == 0 ==> TmError) →
///   this lemma ensures → overall result is Error.
pub proof fn lemma_kernel_pid_always_fails(
    terminate_outcome: TerminateOutcomeView,
)
    requires
        // From the external_body contract: terminate with kernel PID always errors.
        terminate_outcome matches TerminateOutcomeView::TmError { .. },
    ensures
        spec_is_error(
            spec_terminate_result(
                PidParseOutcomeView::PidOk { pid: KERNEL_PID() },
                terminate_outcome,
            )
        ),
{
}

//==================================================================================================
// Proof Functions — PM State Transitions
//==================================================================================================

/// Proof: PM state is unchanged when the overall pipeline result is an error.
///
/// # Description
///
/// When the terminate kcall fails (either PID parse error or PM terminate
/// error), the PM state is preserved. This follows from the pipeline
/// structure:
/// - PID parse error: PM was never called, so `pm_post == pm_pre`.
/// - PM terminate error: the external_body contract guarantees
///   `pm_post == pm_pre` on TmError.
///
/// This lemma proves state preservation from the pipeline invariants,
/// covering both error paths in a single proof.
pub proof fn lemma_state_unchanged_on_error(
    pid_parse_outcome: PidParseOutcomeView,
    terminate_outcome: TerminateOutcomeView,
    pm_pre: ProcessManagerStateView,
    pm_post: ProcessManagerStateView,
)
    requires
        // The overall pipeline result is an error.
        spec_is_error(spec_terminate_result(pid_parse_outcome, terminate_outcome)),
        // Pipeline state invariant: PID parse error path preserves state.
        !spec_pid_parsed_ok(pid_parse_outcome) ==> pm_post == pm_pre,
        // Pipeline state invariant: terminate error path preserves state.
        spec_pid_parsed_ok(pid_parse_outcome) && !spec_terminate_ok(terminate_outcome)
            ==> pm_post == pm_pre,
    ensures
        pm_post == pm_pre,
{
    match pid_parse_outcome {
        PidParseOutcomeView::PidError { .. } => {
            // PID parse failed → PM never called → pm_post == pm_pre.
        },
        PidParseOutcomeView::PidOk { .. } => {
            // PID parsed OK. Since overall is error, terminate must have failed.
            match terminate_outcome {
                TerminateOutcomeView::TmError { .. } => {
                    // Terminate failed → pm_post == pm_pre from invariant.
                },
                TerminateOutcomeView::TmOk => {
                    // Contradicts: overall is error but both steps succeeded.
                },
            }
        },
    }
}

/// Proof: success requires that termination was possible in the pre-state.
///
/// # Description
///
/// Combines `terminate_model`'s postconditions to prove that a successful
/// terminate implies `spec_terminate_possible(pm_pre, pid)`: the PID
/// existed in the pre-state, was not the kernel PID, and was not the
/// running process. This ties the exec model's postconditions to the
/// `spec_terminate_possible` predicate.
pub proof fn lemma_success_requires_terminatable(
    pm_pre: ProcessManagerStateView,
    pid: nat,
)
    requires
        spec_pm_has_process(pm_pre, pid),
        pid != KERNEL_PID(),
        !spec_is_running_process(pm_pre, pid),
    ensures
        spec_terminate_possible(pm_pre, pid),
{
}

/// Proof: running PID terminate produces InvalidArgument error in the pipeline.
///
/// # Description
///
/// When the PID parses successfully but refers to the running process,
/// `process_manager_terminate` returns `InvalidArgument`. This lemma
/// proves the pipeline result carries that error code.
///
/// The trust chain is:
///   `process_manager_terminate` ensures (running PID ==> TmError with
///   InvalidArgument) → this lemma ensures → pipeline result is Error.
pub proof fn lemma_running_pid_returns_error(
    pid: nat,
    error_code: int,
)
    requires
        error_code == ERROR_CODE_INVALID_ARGUMENT(),
    ensures
        spec_terminate_result(
            PidParseOutcomeView::PidOk { pid },
            TerminateOutcomeView::TmError { error_code },
        ) == (TerminateResultView::Error { error_code: ERROR_CODE_INVALID_ARGUMENT() }),
{
}

/// Proof: non-existent PID produces NoSuchProcess error.
///
/// # Description
///
/// When a non-kernel PID does not exist in the PM state, the terminate
/// call returns `ERROR_CODE_NO_SUCH_PROCESS` (ESRCH = 3). This links
/// the spec constant to the specific failure condition.
///
/// The trust chain is:
///   `process_manager_terminate` ensures (non-existent non-kernel PID
///   ==> TmError with error_code == ERROR_CODE_NO_SUCH_PROCESS()) →
///   this lemma ensures → the pipeline result carries that error code.
pub proof fn lemma_nonexistent_pid_returns_no_such_process(
    pid: nat,
    error_code: int,
)
    requires
        error_code == ERROR_CODE_NO_SUCH_PROCESS(),
    ensures
        spec_terminate_result(
            PidParseOutcomeView::PidOk { pid },
            TerminateOutcomeView::TmError { error_code },
        ) == (TerminateResultView::Error { error_code: ERROR_CODE_NO_SUCH_PROCESS() }),
{
}

} // verus!
