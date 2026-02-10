// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Terminate Kernel Call Proofs.
// Proof lemmas for the terminate kcall verification model.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Proof Functions
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

} // verus!
