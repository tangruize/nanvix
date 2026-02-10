// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Join Thread Kernel Call Proofs.
// Proof lemmas for the join_thread kcall verification model.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Proof Functions — Pipeline Error Propagation
//==================================================================================================

/// Proof: when TID parsing fails, its error propagates regardless of other outcomes.
///
/// # Description
///
/// If ThreadIdentifier::try_from fails, the function returns early with
/// SleepError::Generic(error). The join and copy outcomes are irrelevant.
pub proof fn lemma_tid_parse_error_propagates(
    error_code: int,
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
)
    ensures
        spec_join_thread_result(
            TidParseOutcomeView::TidError { error_code },
            join_outcome,
            copy_outcome,
        ) == (JoinThreadResultView::GenericError { error_code }),
        spec_is_error(
            spec_join_thread_result(
                TidParseOutcomeView::TidError { error_code },
                join_outcome,
                copy_outcome,
            )
        ),
{
}

/// Proof: when TID parsing fails, join and copy outcomes do not affect the result.
///
/// # Description
///
/// For any two different (join, copy) outcome pairs, the result is identical
/// when TID parsing fails. This proves the short-circuit behavior.
pub proof fn lemma_tid_parse_short_circuit(
    error_code: int,
    jt1: JoinThreadOutcomeView,
    cp1: CopyToUserOutcomeView,
    jt2: JoinThreadOutcomeView,
    cp2: CopyToUserOutcomeView,
)
    ensures
        spec_join_thread_result(
            TidParseOutcomeView::TidError { error_code },
            jt1,
            cp1,
        ) == spec_join_thread_result(
            TidParseOutcomeView::TidError { error_code },
            jt2,
            cp2,
        ),
{
}

/// Proof: when join_thread returns a generic error, the error propagates.
///
/// # Description
///
/// When TID parsing succeeds but ProcessManager::join_thread fails with
/// SleepError::Generic(error), that error propagates as the final result.
pub proof fn lemma_join_generic_error_propagates(
    tid: nat,
    error_code: int,
    copy_outcome: CopyToUserOutcomeView,
)
    ensures
        spec_join_thread_result(
            TidParseOutcomeView::TidOk { tid },
            JoinThreadOutcomeView::JtError { error_code },
            copy_outcome,
        ) == (JoinThreadResultView::GenericError { error_code }),
        spec_is_error(
            spec_join_thread_result(
                TidParseOutcomeView::TidOk { tid },
                JoinThreadOutcomeView::JtError { error_code },
                copy_outcome,
            )
        ),
{
}

/// Proof: when join_thread is interrupted (killed), the error propagates.
///
/// # Description
///
/// When TID parsing succeeds but ProcessManager::join_thread returns
/// SleepError::Interrupted(Killed), the result is InterruptedKilled.
pub proof fn lemma_join_killed_propagates(
    tid: nat,
    copy_outcome: CopyToUserOutcomeView,
)
    ensures
        spec_join_thread_result(
            TidParseOutcomeView::TidOk { tid },
            JoinThreadOutcomeView::JtInterruptedKilled,
            copy_outcome,
        ) == JoinThreadResultView::InterruptedKilled,
        spec_is_interrupted_killed(
            spec_join_thread_result(
                TidParseOutcomeView::TidOk { tid },
                JoinThreadOutcomeView::JtInterruptedKilled,
                copy_outcome,
            )
        ),
{
}

/// Proof: when join_thread returns an error, the copy outcome is irrelevant.
///
/// # Description
///
/// If join_thread fails (either generic error or killed), the copy outcome
/// does not affect the result. This proves short-circuit at step 2.
pub proof fn lemma_join_error_short_circuits_copy(
    tid: nat,
    join_outcome: JoinThreadOutcomeView,
    cp1: CopyToUserOutcomeView,
    cp2: CopyToUserOutcomeView,
)
    requires
        !spec_join_ok(join_outcome),
    ensures
        spec_join_thread_result(
            TidParseOutcomeView::TidOk { tid },
            join_outcome,
            cp1,
        ) == spec_join_thread_result(
            TidParseOutcomeView::TidOk { tid },
            join_outcome,
            cp2,
        ),
{
}

/// Proof: when copy_to_user fails, the error is wrapped in GenericError.
///
/// # Description
///
/// When TID parsing and join_thread succeed but copy_to_user fails,
/// the copy error is wrapped in SleepError::Generic and returned.
pub proof fn lemma_copy_error_propagates(
    tid: nat,
    exit_status: int,
    error_code: int,
)
    ensures
        spec_join_thread_result(
            TidParseOutcomeView::TidOk { tid },
            JoinThreadOutcomeView::JtOk { exit_status },
            CopyToUserOutcomeView::CopyError { error_code },
        ) == (JoinThreadResultView::GenericError { error_code }),
        spec_is_error(
            spec_join_thread_result(
                TidParseOutcomeView::TidOk { tid },
                JoinThreadOutcomeView::JtOk { exit_status },
                CopyToUserOutcomeView::CopyError { error_code },
            )
        ),
{
}

//==================================================================================================
// Proof Functions — Success / Exhaustiveness
//==================================================================================================

/// Proof: success requires all three pipeline steps to succeed.
///
/// # Description
///
/// The result is Success if and only if:
/// 1. TID parsing succeeded.
/// 2. ProcessManager::join_thread succeeded.
/// 3. copy_to_user succeeded.
pub proof fn lemma_success_requires_all_steps(
    tid_parse_outcome: TidParseOutcomeView,
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
)
    ensures
        spec_is_success(
            spec_join_thread_result(tid_parse_outcome, join_outcome, copy_outcome)
        ) <==> spec_all_steps_passed(tid_parse_outcome, join_outcome, copy_outcome),
{
    match tid_parse_outcome {
        TidParseOutcomeView::TidError { .. } => {},
        TidParseOutcomeView::TidOk { .. } => {
            match join_outcome {
                JoinThreadOutcomeView::JtError { .. } => {},
                JoinThreadOutcomeView::JtInterruptedKilled => {},
                JoinThreadOutcomeView::JtOk { .. } => {
                    match copy_outcome {
                        CopyToUserOutcomeView::CopyError { .. } => {},
                        CopyToUserOutcomeView::CopyOk => {},
                    }
                },
            }
        },
    }
}

/// Proof: the result is always exactly one of Success, GenericError, or InterruptedKilled.
///
/// # Description
///
/// Every possible input combination produces exactly one result category.
/// The categories are mutually exclusive and exhaustive.
pub proof fn lemma_result_exhaustive(
    tid_parse_outcome: TidParseOutcomeView,
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
)
    ensures
        ({
            let result: JoinThreadResultView = spec_join_thread_result(
                tid_parse_outcome, join_outcome, copy_outcome,
            );
            // Exhaustive: every result is success or error.
            spec_is_success(result) || spec_is_error(result)
        }),
        ({
            let result: JoinThreadResultView = spec_join_thread_result(
                tid_parse_outcome, join_outcome, copy_outcome,
            );
            // Mutual exclusion: success and error are disjoint.
            !(spec_is_success(result) && spec_is_error(result))
        }),
        ({
            let result: JoinThreadResultView = spec_join_thread_result(
                tid_parse_outcome, join_outcome, copy_outcome,
            );
            // Mutual exclusion: generic error and interrupted-killed are disjoint.
            !(spec_is_generic_error(result) && spec_is_interrupted_killed(result))
        }),
{
    match tid_parse_outcome {
        TidParseOutcomeView::TidError { .. } => {},
        TidParseOutcomeView::TidOk { .. } => {
            match join_outcome {
                JoinThreadOutcomeView::JtError { .. } => {},
                JoinThreadOutcomeView::JtInterruptedKilled => {},
                JoinThreadOutcomeView::JtOk { .. } => {
                    match copy_outcome {
                        CopyToUserOutcomeView::CopyError { .. } => {},
                        CopyToUserOutcomeView::CopyOk => {},
                    }
                },
            }
        },
    }
}

/// Proof: success implies the result carries ExitStatus::ok().
///
/// # Description
///
/// On the success path, the function returns Ok(ExitStatus::ok()),
/// which has exit status value 0.
pub proof fn lemma_success_implies_ok_status(
    tid_parse_outcome: TidParseOutcomeView,
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
)
    requires
        spec_is_success(
            spec_join_thread_result(tid_parse_outcome, join_outcome, copy_outcome)
        ),
    ensures
        spec_join_thread_result(tid_parse_outcome, join_outcome, copy_outcome)
            == (JoinThreadResultView::Success { exit_status: EXIT_STATUS_OK() }),
{
    match tid_parse_outcome {
        TidParseOutcomeView::TidError { .. } => {},
        TidParseOutcomeView::TidOk { .. } => {
            match join_outcome {
                JoinThreadOutcomeView::JtError { .. } => {},
                JoinThreadOutcomeView::JtInterruptedKilled => {},
                JoinThreadOutcomeView::JtOk { .. } => {
                    match copy_outcome {
                        CopyToUserOutcomeView::CopyError { .. } => {},
                        CopyToUserOutcomeView::CopyOk => {},
                    }
                },
            }
        },
    }
}

//==================================================================================================
// Proof Functions — Error Code Preservation
//==================================================================================================

/// Proof: the spec constant ERROR_CODE_INVALID_ARGUMENT matches
/// ErrorCode::InvalidArgument.
pub proof fn lemma_error_code_matches()
    ensures
        ERROR_CODE_INVALID_ARGUMENT() == ErrorCode::InvalidArgument as int,
{
    assert(ErrorCode::InvalidArgument as int == 22int);
}

/// Proof: TID parse error always produces InvalidArgument error code.
///
/// # Description
///
/// ThreadIdentifier::try_from returns ErrorCode::InvalidArgument on failure.
/// Combined with the external_body postcondition, the final result carries
/// InvalidArgument.
pub proof fn lemma_invalid_tid_returns_invalid_argument(
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
)
    ensures
        spec_join_thread_result(
            TidParseOutcomeView::TidError { error_code: ERROR_CODE_INVALID_ARGUMENT() },
            join_outcome,
            copy_outcome,
        ) == (JoinThreadResultView::GenericError { error_code: ERROR_CODE_INVALID_ARGUMENT() }),
{
}

/// Proof: the error code from TID parsing is preserved in the final result.
pub proof fn lemma_tid_error_code_preserved(
    error_code: int,
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
)
    ensures
        ({
            let result: JoinThreadResultView = spec_join_thread_result(
                TidParseOutcomeView::TidError { error_code },
                join_outcome,
                copy_outcome,
            );
            result == JoinThreadResultView::GenericError { error_code }
        }),
{
}

/// Proof: the error code from join_thread generic error is preserved.
pub proof fn lemma_join_error_code_preserved(
    tid: nat,
    error_code: int,
    copy_outcome: CopyToUserOutcomeView,
)
    ensures
        ({
            let result: JoinThreadResultView = spec_join_thread_result(
                TidParseOutcomeView::TidOk { tid },
                JoinThreadOutcomeView::JtError { error_code },
                copy_outcome,
            );
            result == JoinThreadResultView::GenericError { error_code }
        }),
{
}

/// Proof: the error code from copy_to_user is preserved.
pub proof fn lemma_copy_error_code_preserved(
    tid: nat,
    exit_status: int,
    error_code: int,
)
    ensures
        ({
            let result: JoinThreadResultView = spec_join_thread_result(
                TidParseOutcomeView::TidOk { tid },
                JoinThreadOutcomeView::JtOk { exit_status },
                CopyToUserOutcomeView::CopyError { error_code },
            );
            result == JoinThreadResultView::GenericError { error_code }
        }),
{
}

//==================================================================================================
// Proof Functions — Validation Failure Generalization
//==================================================================================================

/// Proof: any pipeline failure produces an error result.
///
/// # Description
///
/// If any of the three pipeline steps fail, the result is always an error.
pub proof fn lemma_any_failure_is_error(
    tid_parse_outcome: TidParseOutcomeView,
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
)
    requires
        !spec_all_steps_passed(tid_parse_outcome, join_outcome, copy_outcome),
    ensures
        spec_is_error(
            spec_join_thread_result(tid_parse_outcome, join_outcome, copy_outcome)
        ),
{
    match tid_parse_outcome {
        TidParseOutcomeView::TidError { .. } => {},
        TidParseOutcomeView::TidOk { .. } => {
            match join_outcome {
                JoinThreadOutcomeView::JtError { .. } => {},
                JoinThreadOutcomeView::JtInterruptedKilled => {},
                JoinThreadOutcomeView::JtOk { .. } => {
                    match copy_outcome {
                        CopyToUserOutcomeView::CopyError { .. } => {},
                        CopyToUserOutcomeView::CopyOk => {},
                    }
                },
            }
        },
    }
}

/// Proof: InterruptedKilled can only come from the join step.
///
/// # Description
///
/// The only way to get an InterruptedKilled result is when the TID
/// parsing succeeds and join_thread returns Interrupted(Killed).
pub proof fn lemma_killed_only_from_join(
    tid_parse_outcome: TidParseOutcomeView,
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
)
    ensures
        spec_is_interrupted_killed(
            spec_join_thread_result(tid_parse_outcome, join_outcome, copy_outcome)
        ) <==> (
            spec_tid_parsed_ok(tid_parse_outcome)
            && join_outcome == JoinThreadOutcomeView::JtInterruptedKilled
        ),
{
    match tid_parse_outcome {
        TidParseOutcomeView::TidError { .. } => {},
        TidParseOutcomeView::TidOk { .. } => {
            match join_outcome {
                JoinThreadOutcomeView::JtError { .. } => {},
                JoinThreadOutcomeView::JtInterruptedKilled => {},
                JoinThreadOutcomeView::JtOk { .. } => {
                    match copy_outcome {
                        CopyToUserOutcomeView::CopyError { .. } => {},
                        CopyToUserOutcomeView::CopyOk => {},
                    }
                },
            }
        },
    }
}

/// Proof: on success, the join outcome is JtOk and its exit status is written
/// to user memory via spec_user_mem_written.
///
/// # Description
///
/// Establishes that on the success path, spec_join_exit_status correctly
/// extracts the exit status from JtOk, linking the join outcome to the
/// copy_to_user postcondition.
pub proof fn lemma_success_join_exit_status(
    tid_parse_outcome: TidParseOutcomeView,
    join_outcome: JoinThreadOutcomeView,
    copy_outcome: CopyToUserOutcomeView,
)
    requires
        spec_is_success(
            spec_join_thread_result(tid_parse_outcome, join_outcome, copy_outcome)
        ),
    ensures
        spec_join_ok(join_outcome),
        join_outcome matches JoinThreadOutcomeView::JtOk { exit_status }
            ==> spec_join_exit_status(join_outcome) == exit_status,
{
    match tid_parse_outcome {
        TidParseOutcomeView::TidError { .. } => {},
        TidParseOutcomeView::TidOk { .. } => {
            match join_outcome {
                JoinThreadOutcomeView::JtError { .. } => {},
                JoinThreadOutcomeView::JtInterruptedKilled => {},
                JoinThreadOutcomeView::JtOk { .. } => {
                    match copy_outcome {
                        CopyToUserOutcomeView::CopyError { .. } => {},
                        CopyToUserOutcomeView::CopyOk => {},
                    }
                },
            }
        },
    }
}

/// Proof: ExitStatus::ok() has value 0.
///
/// # Description
///
/// Documents that EXIT_STATUS_OK() == 0, matching ExitStatus::ok() = ExitStatus(0).
pub proof fn lemma_exit_status_ok_is_zero()
    ensures
        EXIT_STATUS_OK() == 0int,
{
}

//==================================================================================================
// Proof Functions — TID Validity
//==================================================================================================

/// Axiom: spec_is_valid_tid(raw) <==> raw <= i32::MAX.
///
/// # Description
///
/// The actual `ThreadIdentifier` wraps an `i32` internally. `try_from(u32)`
/// succeeds iff the u32 value fits in a non-negative i32, i.e.,
/// `raw <= 2147483647` (i32::MAX).
///
/// ## Trust Boundary (Deliberate)
///
/// This axiom is an `external_body` assumption that serves as a trust boundary
/// with the TID module. It should be discharged by verification of the
/// `ThreadIdentifier::try_from(u32)` implementation in the TID module
/// (`verus/split/kernel/pm/kcall/tid.rs` or the corresponding tid module).
/// The TID module's verified `try_from` postcondition should establish that
/// `try_from(raw)` succeeds iff `raw <= i32::MAX`, which is exactly what
/// this axiom states. Until the TID module formally exports this fact,
/// it remains a documented trust dependency.
#[verifier::external_body]
pub proof fn axiom_valid_tid_range(raw: nat)
    ensures
        spec_is_valid_tid(raw) <==> raw <= 2147483647nat,
{
}

//==================================================================================================
// Proof Functions — TimedOut Exclusion
//==================================================================================================

/// Axiom: `ProcessManager::join_thread` uses `wait(None)`, excluding TimedOut.
///
/// # Description
///
/// The `ProcessManager::join_thread` implementation calls
/// `join_cond.wait(None)?` (see `src/kernel/src/pm/process/manager/unsafe.rs:402`).
/// The `None` alarm argument means no timeout is set, so `Condvar::wait` can
/// only be interrupted by `Killed`, never by `TimedOut`.
///
/// ## Trust Boundary (Deliberate)
///
/// This axiom is an `external_body` assumption that serves as a trust boundary
/// with the PM module. It should be discharged by verification of the
/// `ProcessManager::join_thread` implementation confirming it passes `None`
/// to `Condvar::wait`. If the PM implementation were changed to pass
/// `Some(alarm)` to `wait`, this axiom would become invalid and need removal.
#[verifier::external_body]
pub proof fn axiom_join_wait_excludes_timeout()
    ensures
        spec_join_wait_excludes_timeout(),
{
}

//==================================================================================================
// Proof Functions — Error Code Domain
//==================================================================================================

/// Proof: known error codes are valid positive error codes.
///
/// # Description
///
/// Links `spec_is_known_error_code` (the verified ErrorCode subset) to
/// `spec_is_valid_error_code` (code > 0). All ErrorCode values in the
/// subset (2, 3, 12, 14, 16, 22) are positive.
pub proof fn lemma_known_error_code_implies_valid(code: int)
    requires
        spec_is_known_error_code(code),
    ensures
        spec_is_valid_error_code(code),
{
}

/// Proof: InvalidArgument is a known error code.
pub proof fn lemma_invalid_argument_is_known()
    ensures
        spec_is_known_error_code(ERROR_CODE_INVALID_ARGUMENT()),
{
}

} // verus!
