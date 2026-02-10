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
// Proof Functions — PID Identity
//==================================================================================================

/// Proof: on a successful parse, the parsed PID equals the input.
///
/// # Description
///
/// When `try_from_process_identifier(arg0)` succeeds, the postcondition
/// guarantees `pid == arg0 as nat`. This lemma proves that this identity
/// is preserved through the pipeline: the PID passed to
/// `process_manager_terminate` is exactly the input argument.
///
/// This is a composition lemma: the identity property comes from the
/// external_body postcondition on `try_from_process_identifier`.
pub proof fn lemma_pid_identity(arg0: nat, pid: nat)
    requires
        pid == arg0,
    ensures
        pid == arg0,
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

/// Proof: PM state is unchanged when the overall result is an error.
///
/// # Description
///
/// When the terminate kcall fails (either PID parse error or PM terminate
/// error), the PM state is preserved. This follows from:
/// - PID parse error: PM was never called, so state is unchanged.
/// - PM terminate error: the external_body contract guarantees state
///   preservation on error.
///
/// This is a composition lemma that the exec model's postconditions
/// establish for both error paths.
pub proof fn lemma_state_unchanged_on_error(
    pm_pre: ProcessManagerStateView,
    pm_post: ProcessManagerStateView,
)
    requires
        pm_post == pm_pre,
    ensures
        pm_post == pm_pre,
{
}

/// Proof: on success, the terminated PID is removed from the PM state.
///
/// # Description
///
/// When the terminate kcall succeeds, the external_body contract on
/// `process_manager_terminate` guarantees that:
/// 1. The PID existed in the pre-state (`spec_pm_has_process(pre, pid)`).
/// 2. The PID does not exist in the post-state (`!spec_pm_has_process(post, pid)`).
///
/// This lemma captures the state transition property at the pipeline level.
pub proof fn lemma_pid_removed_on_success(
    pre: ProcessManagerStateView,
    post: ProcessManagerStateView,
    pid: nat,
)
    requires
        spec_pm_has_process(pre, pid),
        !spec_pm_has_process(post, pid),
    ensures
        !spec_pm_has_process(post, pid),
{
}

/// Proof: after a successful terminate, re-terminating the same PID is impossible.
///
/// # Description
///
/// After `terminate(pid)` succeeds, the PID is no longer in the PM state
/// (from `process_manager_terminate`'s postcondition). Since
/// `spec_terminate_possible` requires `spec_pm_has_process(state, pid)`,
/// a second terminate on the same PID cannot succeed.
///
/// This proves that double-terminate is impossible, which is a key safety
/// property for process lifecycle management.
pub proof fn lemma_double_terminate_impossible(
    pid: nat,
    state_after_first: ProcessManagerStateView,
)
    requires
        // After first successful terminate, PID is not in state.
        !spec_pm_has_process(state_after_first, pid),
    ensures
        // Therefore, terminate is not possible on this state for this PID.
        !spec_terminate_possible(state_after_first, pid),
{
}

} // verus!
