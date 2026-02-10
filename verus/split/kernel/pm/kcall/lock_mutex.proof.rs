// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Lock Mutex Kernel Call Proofs.
// Proof lemmas for the lock_mutex kcall verification model.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Proof Functions
//==================================================================================================

/// Proof: when both timeout params are MAX, the timeout is infinite.
///
/// # Description
///
/// The original code checks `timeout_s == usize::MAX && timeout_ns == usize::MAX`
/// and sets timeout to None (infinite wait). This lemma proves the spec correctly
/// models this as TimeoutView::Infinite.
pub proof fn lemma_max_max_is_infinite(timeout_s: nat, timeout_ns: nat)
    requires
        timeout_s == USIZE_MAX_X86_32(),
        timeout_ns == USIZE_MAX_X86_32(),
    ensures
        spec_is_infinite_timeout(timeout_s, timeout_ns),
        spec_parse_timeout(timeout_s, timeout_ns) == Some(TimeoutView::Infinite),
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
{
}

/// Proof: when timeout_ns >= NANOS_PER_SEC and not both MAX, the timeout is invalid.
///
/// # Description
///
/// SystemTime::new returns None when nanoseconds >= 1_000_000_000. This lemma
/// proves that the spec returns None (invalid) in this case.
pub proof fn lemma_invalid_nanos_returns_none(timeout_s: nat, timeout_ns: nat)
    requires
        !spec_is_infinite_timeout(timeout_s, timeout_ns),
        timeout_ns >= NANOS_PER_SEC(),
    ensures
        spec_parse_timeout(timeout_s, timeout_ns).is_none(),
        !spec_timeout_parsed_ok(timeout_s, timeout_ns),
{
}

/// Proof: when timeout is invalid, the result is InvalidTimeoutError.
///
/// # Description
///
/// When SystemTime::new returns None, the lock_mutex function returns
/// Err(SleepError::Generic(Error::new(ErrorCode::InvalidArgument, ...))).
/// This lemma proves the spec produces InvalidTimeoutError with the correct
/// error code, regardless of PM outcomes.
pub proof fn lemma_invalid_timeout_returns_error(
    timeout_s: nat,
    timeout_ns: nat,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    requires
        !spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_lock_mutex_result(timeout_s, timeout_ns, get_mutex_outcome, lock_outcome, put_guard_outcome)
            == LockMutexResultView::InvalidTimeoutError {
                error_code: ERROR_CODE_INVALID_ARGUMENT(),
            },
        spec_is_timeout_error(
            spec_lock_mutex_result(timeout_s, timeout_ns, get_mutex_outcome, lock_outcome, put_guard_outcome)
        ),
{
}

/// Proof: when get_mutex fails, its error propagates regardless of later steps.
///
/// # Description
///
/// If the timeout was parsed successfully but ProcessManager::get_mutex fails,
/// the function returns early with GetMutexError. The lock and put_guard
/// outcomes are irrelevant (pipeline short-circuit).
pub proof fn lemma_get_mutex_error_propagates(
    timeout_s: nat,
    timeout_ns: nat,
    error_code: int,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_lock_mutex_result(
            timeout_s, timeout_ns,
            GetMutexOutcomeView::GmError { error_code },
            lock_outcome,
            put_guard_outcome,
        ) == LockMutexResultView::GetMutexError { error_code },
        spec_is_get_mutex_error(
            spec_lock_mutex_result(
                timeout_s, timeout_ns,
                GetMutexOutcomeView::GmError { error_code },
                lock_outcome,
                put_guard_outcome,
            )
        ),
{
}

/// Proof: lock errors propagate unchanged through the pipeline.
///
/// # Description
///
/// When Mutex::lock fails, the SleepError is propagated unchanged via `?`.
/// This lemma proves that each lock error variant maps to the corresponding
/// result variant. The put_guard outcome is irrelevant.
pub proof fn lemma_lock_error_propagates(
    timeout_s: nat,
    timeout_ns: nat,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
        !matches!(lock_outcome, LockOutcomeView::LoOk),
    ensures
        spec_is_lock_error(
            spec_lock_mutex_result(
                timeout_s, timeout_ns,
                GetMutexOutcomeView::GmOk,
                lock_outcome,
                put_guard_outcome,
            )
        ),
        // TimedOut maps to LockTimedOut.
        matches!(lock_outcome, LockOutcomeView::LoTimedOut) ==>
            spec_lock_mutex_result(
                timeout_s, timeout_ns,
                GetMutexOutcomeView::GmOk,
                lock_outcome,
                put_guard_outcome,
            ) == LockMutexResultView::LockTimedOut,
        // Killed maps to LockKilled.
        matches!(lock_outcome, LockOutcomeView::LoKilled) ==>
            spec_lock_mutex_result(
                timeout_s, timeout_ns,
                GetMutexOutcomeView::GmOk,
                lock_outcome,
                put_guard_outcome,
            ) == LockMutexResultView::LockKilled,
{
    match lock_outcome {
        LockOutcomeView::LoTimedOut => {},
        LockOutcomeView::LoKilled => {},
        LockOutcomeView::LoGenericError { .. } => {},
        LockOutcomeView::LoOk => {},
    }
}

/// Proof: put_guard error propagates when all prior steps succeed.
///
/// # Description
///
/// When timeout is valid, get_mutex succeeds, and lock succeeds, but
/// put_mutex_guard fails, the error propagates as PutGuardError.
pub proof fn lemma_put_guard_error_propagates(
    timeout_s: nat,
    timeout_ns: nat,
    error_code: int,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_lock_mutex_result(
            timeout_s, timeout_ns,
            GetMutexOutcomeView::GmOk,
            LockOutcomeView::LoOk,
            PutGuardOutcomeView::PgError { error_code },
        ) == LockMutexResultView::PutGuardError { error_code },
        spec_is_put_guard_error(
            spec_lock_mutex_result(
                timeout_s, timeout_ns,
                GetMutexOutcomeView::GmOk,
                LockOutcomeView::LoOk,
                PutGuardOutcomeView::PgError { error_code },
            )
        ),
{
}

/// Proof: success only when all three pipeline steps succeed.
///
/// # Description
///
/// The result is Success if and only if:
/// 1. Timeout was parsed successfully.
/// 2. get_mutex returned GmOk.
/// 3. lock returned LoOk.
/// 4. put_mutex_guard returned PgOk.
pub proof fn lemma_success_requires_all_steps(
    timeout_s: nat,
    timeout_ns: nat,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    ensures
        spec_is_success(
            spec_lock_mutex_result(timeout_s, timeout_ns, get_mutex_outcome, lock_outcome, put_guard_outcome)
        ) <==> (
            spec_timeout_parsed_ok(timeout_s, timeout_ns)
            && matches!(get_mutex_outcome, GetMutexOutcomeView::GmOk)
            && matches!(lock_outcome, LockOutcomeView::LoOk)
            && matches!(put_guard_outcome, PutGuardOutcomeView::PgOk)
        ),
{
    // Unfold spec_parse_timeout to help Verus.
    let parsed: Option<TimeoutView> = spec_parse_timeout(timeout_s, timeout_ns);
    match parsed {
        None => {},
        Some(_) => {
            match get_mutex_outcome {
                GetMutexOutcomeView::GmError { .. } => {},
                GetMutexOutcomeView::GmOk => {
                    match lock_outcome {
                        LockOutcomeView::LoOk => {
                            match put_guard_outcome {
                                PutGuardOutcomeView::PgOk => {},
                                PutGuardOutcomeView::PgError { .. } => {},
                            }
                        },
                        LockOutcomeView::LoTimedOut => {},
                        LockOutcomeView::LoKilled => {},
                        LockOutcomeView::LoGenericError { .. } => {},
                    }
                },
            }
        },
    }
}

/// Proof: the result is always exactly one of the defined categories.
///
/// # Description
///
/// The lock_mutex result is exhaustive: every possible input combination
/// produces exactly one result category (success, timeout error, get_mutex
/// error, lock error, or put_guard error).
pub proof fn lemma_result_exhaustive(
    timeout_s: nat,
    timeout_ns: nat,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    ensures ({
        let result: LockMutexResultView = spec_lock_mutex_result(
            timeout_s, timeout_ns, get_mutex_outcome, lock_outcome, put_guard_outcome
        );
        // Exhaustive: every result falls into exactly one category.
        spec_is_success(result)
        || spec_is_timeout_error(result)
        || spec_is_get_mutex_error(result)
        || spec_is_lock_error(result)
        || spec_is_put_guard_error(result)
    }),
    ensures ({
        let result: LockMutexResultView = spec_lock_mutex_result(
            timeout_s, timeout_ns, get_mutex_outcome, lock_outcome, put_guard_outcome
        );
        // Mutual exclusion: success and error are disjoint.
        !(spec_is_success(result) && spec_is_error(result))
    }),
{
    let parsed: Option<TimeoutView> = spec_parse_timeout(timeout_s, timeout_ns);
    match parsed {
        None => {},
        Some(_) => {
            match get_mutex_outcome {
                GetMutexOutcomeView::GmError { .. } => {},
                GetMutexOutcomeView::GmOk => {
                    match lock_outcome {
                        LockOutcomeView::LoOk => {
                            match put_guard_outcome {
                                PutGuardOutcomeView::PgOk => {},
                                PutGuardOutcomeView::PgError { .. } => {},
                            }
                        },
                        LockOutcomeView::LoTimedOut => {},
                        LockOutcomeView::LoKilled => {},
                        LockOutcomeView::LoGenericError { .. } => {},
                    }
                },
            }
        },
    }
}

/// Proof: valid finite timeout produces well-formed SystemTime parameters.
///
/// # Description
///
/// When parse_timeout returns Finite, the nanoseconds component is guaranteed
/// to be less than NANOS_PER_SEC(), which is exactly the precondition for
/// SystemTime::new to return Some.
pub proof fn lemma_finite_timeout_wf(timeout_s: nat, timeout_ns: nat)
    requires
        spec_parse_timeout(timeout_s, timeout_ns) == Some(TimeoutView::Finite {
            seconds: timeout_s,
            nanoseconds: timeout_ns,
        }),
    ensures
        spec_finite_timeout_wf(timeout_s, timeout_ns),
        timeout_ns < NANOS_PER_SEC(),
{
}

/// Proof: the spec constant ERROR_CODE_INVALID_ARGUMENT matches ErrorCode::InvalidArgument.
///
/// # Description
///
/// Links the spec-level error code constant to the concrete `ErrorCode::InvalidArgument`
/// enum discriminant.
pub proof fn lemma_error_code_matches()
    ensures
        ERROR_CODE_INVALID_ARGUMENT() == ErrorCode::InvalidArgument as int,
{
    assert(ErrorCode::InvalidArgument as int == 22int);
}

/// Proof: pipeline ordering — later steps only matter if earlier steps succeed.
///
/// # Description
///
/// If get_mutex fails, the lock_outcome and put_guard_outcome do not affect
/// the result. Similarly, if lock fails, put_guard_outcome does not matter.
/// This proves the short-circuit behavior of the `?` operator.
pub proof fn lemma_pipeline_short_circuit(
    timeout_s: nat,
    timeout_ns: nat,
    gm1: GetMutexOutcomeView,
    lo1: LockOutcomeView,
    lo2: LockOutcomeView,
    pg1: PutGuardOutcomeView,
    pg2: PutGuardOutcomeView,
)
    requires
        !spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        // When timeout is invalid, all PM outcomes are irrelevant.
        spec_lock_mutex_result(timeout_s, timeout_ns, gm1, lo1, pg1)
            == spec_lock_mutex_result(timeout_s, timeout_ns, gm1, lo2, pg2),
{
}

/// Proof: when get_mutex fails, lock and put_guard outcomes are irrelevant.
pub proof fn lemma_get_mutex_short_circuit(
    timeout_s: nat,
    timeout_ns: nat,
    error_code: int,
    lo1: LockOutcomeView,
    lo2: LockOutcomeView,
    pg1: PutGuardOutcomeView,
    pg2: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_lock_mutex_result(
            timeout_s, timeout_ns,
            GetMutexOutcomeView::GmError { error_code },
            lo1, pg1,
        ) == spec_lock_mutex_result(
            timeout_s, timeout_ns,
            GetMutexOutcomeView::GmError { error_code },
            lo2, pg2,
        ),
{
}

/// Proof: when lock fails, put_guard outcome is irrelevant.
pub proof fn lemma_lock_short_circuit(
    timeout_s: nat,
    timeout_ns: nat,
    lock_outcome: LockOutcomeView,
    pg1: PutGuardOutcomeView,
    pg2: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
        !matches!(lock_outcome, LockOutcomeView::LoOk),
    ensures
        spec_lock_mutex_result(
            timeout_s, timeout_ns,
            GetMutexOutcomeView::GmOk,
            lock_outcome,
            pg1,
        ) == spec_lock_mutex_result(
            timeout_s, timeout_ns,
            GetMutexOutcomeView::GmOk,
            lock_outcome,
            pg2,
        ),
{
    match lock_outcome {
        LockOutcomeView::LoTimedOut => {},
        LockOutcomeView::LoKilled => {},
        LockOutcomeView::LoGenericError { .. } => {},
        LockOutcomeView::LoOk => {},
    }
}

} // verus!
