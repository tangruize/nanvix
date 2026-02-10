// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Wait Condition Kernel Call Proofs.
// Proof lemmas for the wait_cond kcall verification model.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Proof Functions
//==================================================================================================

/// Proof: when both timeout params are MAX, the timeout is infinite.
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
pub proof fn lemma_invalid_timeout_returns_error(
    timeout_s: nat,
    timeout_ns: nat,
    take_guard_outcome: TakeMutexGuardOutcomeView,
    get_cond_outcome: GetCondOutcomeView,
    cond_wait_outcome: CondWaitOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    requires
        !spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_wait_cond_result(
            timeout_s, timeout_ns,
            take_guard_outcome, get_cond_outcome, cond_wait_outcome,
            put_cond_outcome, get_mutex_outcome, lock_outcome, put_guard_outcome,
        ) == (WaitCondResultView::InvalidTimeoutError {
            error_code: ERROR_CODE_INVALID_ARGUMENT(),
        }),
        spec_is_timeout_error(
            spec_wait_cond_result(
                timeout_s, timeout_ns,
                take_guard_outcome, get_cond_outcome, cond_wait_outcome,
                put_cond_outcome, get_mutex_outcome, lock_outcome, put_guard_outcome,
            )
        ),
{
}

/// Proof: when take_mutex_guard fails, its error propagates regardless of later steps.
pub proof fn lemma_take_guard_error_propagates(
    timeout_s: nat,
    timeout_ns: nat,
    error_code: int,
    get_cond_outcome: GetCondOutcomeView,
    cond_wait_outcome: CondWaitOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_wait_cond_result(
            timeout_s, timeout_ns,
            TakeMutexGuardOutcomeView::TmgError { error_code },
            get_cond_outcome, cond_wait_outcome,
            put_cond_outcome, get_mutex_outcome, lock_outcome, put_guard_outcome,
        ) == (WaitCondResultView::TakeMutexGuardError { error_code }),
        spec_is_take_guard_error(
            spec_wait_cond_result(
                timeout_s, timeout_ns,
                TakeMutexGuardOutcomeView::TmgError { error_code },
                get_cond_outcome, cond_wait_outcome,
                put_cond_outcome, get_mutex_outcome, lock_outcome, put_guard_outcome,
            )
        ),
{
}

/// Proof: when get_cond fails, its error propagates.
pub proof fn lemma_get_cond_error_propagates(
    timeout_s: nat,
    timeout_ns: nat,
    error_code: int,
    cond_wait_outcome: CondWaitOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_wait_cond_result(
            timeout_s, timeout_ns,
            TakeMutexGuardOutcomeView::TmgOk,
            GetCondOutcomeView::GcError { error_code },
            cond_wait_outcome,
            put_cond_outcome, get_mutex_outcome, lock_outcome, put_guard_outcome,
        ) == (WaitCondResultView::GetCondError { error_code }),
        spec_is_get_cond_error(
            spec_wait_cond_result(
                timeout_s, timeout_ns,
                TakeMutexGuardOutcomeView::TmgOk,
                GetCondOutcomeView::GcError { error_code },
                cond_wait_outcome,
                put_cond_outcome, get_mutex_outcome, lock_outcome, put_guard_outcome,
            )
        ),
{
}

/// Proof: when put_cond fails, its error propagates regardless of mutex reacquisition.
pub proof fn lemma_put_cond_error_propagates(
    timeout_s: nat,
    timeout_ns: nat,
    cond_wait_outcome: CondWaitOutcomeView,
    error_code: int,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_wait_cond_result(
            timeout_s, timeout_ns,
            TakeMutexGuardOutcomeView::TmgOk,
            GetCondOutcomeView::GcOk,
            cond_wait_outcome,
            PutCondOutcomeView::PcError { error_code },
            get_mutex_outcome, lock_outcome, put_guard_outcome,
        ) == (WaitCondResultView::PutCondError { error_code }),
        spec_is_put_cond_error(
            spec_wait_cond_result(
                timeout_s, timeout_ns,
                TakeMutexGuardOutcomeView::TmgOk,
                GetCondOutcomeView::GcOk,
                cond_wait_outcome,
                PutCondOutcomeView::PcError { error_code },
                get_mutex_outcome, lock_outcome, put_guard_outcome,
            )
        ),
{
}

/// Proof: when get_mutex fails during reacquisition, its error propagates.
pub proof fn lemma_get_mutex_error_propagates(
    timeout_s: nat,
    timeout_ns: nat,
    cond_wait_outcome: CondWaitOutcomeView,
    error_code: int,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_wait_cond_result(
            timeout_s, timeout_ns,
            TakeMutexGuardOutcomeView::TmgOk,
            GetCondOutcomeView::GcOk,
            cond_wait_outcome,
            PutCondOutcomeView::PcOk,
            GetMutexOutcomeView::GmError { error_code },
            lock_outcome, put_guard_outcome,
        ) == (WaitCondResultView::GetMutexError { error_code }),
        spec_is_get_mutex_error(
            spec_wait_cond_result(
                timeout_s, timeout_ns,
                TakeMutexGuardOutcomeView::TmgOk,
                GetCondOutcomeView::GcOk,
                cond_wait_outcome,
                PutCondOutcomeView::PcOk,
                GetMutexOutcomeView::GmError { error_code },
                lock_outcome, put_guard_outcome,
            )
        ),
{
}

/// Proof: lock errors during mutex reacquisition propagate.
pub proof fn lemma_lock_error_propagates(
    timeout_s: nat,
    timeout_ns: nat,
    cond_wait_outcome: CondWaitOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
        !matches!(lock_outcome, LockOutcomeView::LoOk),
    ensures
        spec_is_lock_error(
            spec_wait_cond_result(
                timeout_s, timeout_ns,
                TakeMutexGuardOutcomeView::TmgOk,
                GetCondOutcomeView::GcOk,
                cond_wait_outcome,
                PutCondOutcomeView::PcOk,
                GetMutexOutcomeView::GmOk,
                lock_outcome, put_guard_outcome,
            )
        ),
{
    match lock_outcome {
        LockOutcomeView::LoTimedOut => {},
        LockOutcomeView::LoKilled => {},
        LockOutcomeView::LoGenericError { .. } => {},
        LockOutcomeView::LoOk => {},
    }
}

/// Proof: put_guard error during reacquisition propagates.
pub proof fn lemma_put_guard_error_propagates(
    timeout_s: nat,
    timeout_ns: nat,
    cond_wait_outcome: CondWaitOutcomeView,
    error_code: int,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_wait_cond_result(
            timeout_s, timeout_ns,
            TakeMutexGuardOutcomeView::TmgOk,
            GetCondOutcomeView::GcOk,
            cond_wait_outcome,
            PutCondOutcomeView::PcOk,
            GetMutexOutcomeView::GmOk,
            LockOutcomeView::LoOk,
            PutGuardOutcomeView::PgError { error_code },
        ) == (WaitCondResultView::PutGuardError { error_code }),
        spec_is_put_guard_error(
            spec_wait_cond_result(
                timeout_s, timeout_ns,
                TakeMutexGuardOutcomeView::TmgOk,
                GetCondOutcomeView::GcOk,
                cond_wait_outcome,
                PutCondOutcomeView::PcOk,
                GetMutexOutcomeView::GmOk,
                LockOutcomeView::LoOk,
                PutGuardOutcomeView::PgError { error_code },
            )
        ),
{
}

/// Proof: success only when ALL pipeline steps succeed AND cond.wait returns Ok.
pub proof fn lemma_success_requires_all_steps(
    timeout_s: nat,
    timeout_ns: nat,
    take_guard_outcome: TakeMutexGuardOutcomeView,
    get_cond_outcome: GetCondOutcomeView,
    cond_wait_outcome: CondWaitOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    ensures
        spec_is_success(
            spec_wait_cond_result(
                timeout_s, timeout_ns,
                take_guard_outcome, get_cond_outcome, cond_wait_outcome,
                put_cond_outcome, get_mutex_outcome, lock_outcome, put_guard_outcome,
            )
        ) <==> (
            spec_timeout_parsed_ok(timeout_s, timeout_ns)
            && matches!(take_guard_outcome, TakeMutexGuardOutcomeView::TmgOk)
            && matches!(get_cond_outcome, GetCondOutcomeView::GcOk)
            && matches!(cond_wait_outcome, CondWaitOutcomeView::CwOk)
            && matches!(put_cond_outcome, PutCondOutcomeView::PcOk)
            && matches!(get_mutex_outcome, GetMutexOutcomeView::GmOk)
            && matches!(lock_outcome, LockOutcomeView::LoOk)
            && matches!(put_guard_outcome, PutGuardOutcomeView::PgOk)
        ),
{
    let parsed: Option<TimeoutView> = spec_parse_timeout(timeout_s, timeout_ns);
    match parsed {
        None => {},
        Some(_) => {
            match take_guard_outcome {
                TakeMutexGuardOutcomeView::TmgError { .. } => {},
                TakeMutexGuardOutcomeView::TmgOk => {
                    match get_cond_outcome {
                        GetCondOutcomeView::GcError { .. } => {},
                        GetCondOutcomeView::GcOk => {
                            match put_cond_outcome {
                                PutCondOutcomeView::PcError { .. } => {},
                                PutCondOutcomeView::PcOk => {
                                    match get_mutex_outcome {
                                        GetMutexOutcomeView::GmError { .. } => {},
                                        GetMutexOutcomeView::GmOk => {
                                            match lock_outcome {
                                                LockOutcomeView::LoOk => {
                                                    match put_guard_outcome {
                                                        PutGuardOutcomeView::PgOk => {
                                                            match cond_wait_outcome {
                                                                CondWaitOutcomeView::CwOk => {},
                                                                CondWaitOutcomeView::CwTimedOut => {},
                                                                CondWaitOutcomeView::CwKilled => {},
                                                                CondWaitOutcomeView::CwGenericError { .. } => {},
                                                            }
                                                        },
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
                        },
                    }
                },
            }
        },
    }
}

/// Proof: the result is always exactly one of the defined categories.
pub proof fn lemma_result_exhaustive(
    timeout_s: nat,
    timeout_ns: nat,
    take_guard_outcome: TakeMutexGuardOutcomeView,
    get_cond_outcome: GetCondOutcomeView,
    cond_wait_outcome: CondWaitOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    ensures
        ({
            let result: WaitCondResultView = spec_wait_cond_result(
                timeout_s, timeout_ns,
                take_guard_outcome, get_cond_outcome, cond_wait_outcome,
                put_cond_outcome, get_mutex_outcome, lock_outcome, put_guard_outcome,
            );
            spec_is_success(result)
            || spec_is_timeout_error(result)
            || spec_is_take_guard_error(result)
            || spec_is_get_cond_error(result)
            || spec_is_cond_wait_error(result)
            || spec_is_put_cond_error(result)
            || spec_is_get_mutex_error(result)
            || spec_is_lock_error(result)
            || spec_is_put_guard_error(result)
        }),
        ({
            let result: WaitCondResultView = spec_wait_cond_result(
                timeout_s, timeout_ns,
                take_guard_outcome, get_cond_outcome, cond_wait_outcome,
                put_cond_outcome, get_mutex_outcome, lock_outcome, put_guard_outcome,
            );
            !(spec_is_success(result) && spec_is_error(result))
        }),
{
    let parsed: Option<TimeoutView> = spec_parse_timeout(timeout_s, timeout_ns);
    match parsed {
        None => {},
        Some(_) => {
            match take_guard_outcome {
                TakeMutexGuardOutcomeView::TmgError { .. } => {},
                TakeMutexGuardOutcomeView::TmgOk => {
                    match get_cond_outcome {
                        GetCondOutcomeView::GcError { .. } => {},
                        GetCondOutcomeView::GcOk => {
                            match put_cond_outcome {
                                PutCondOutcomeView::PcError { .. } => {},
                                PutCondOutcomeView::PcOk => {
                                    match get_mutex_outcome {
                                        GetMutexOutcomeView::GmError { .. } => {},
                                        GetMutexOutcomeView::GmOk => {
                                            match lock_outcome {
                                                LockOutcomeView::LoOk => {
                                                    match put_guard_outcome {
                                                        PutGuardOutcomeView::PgOk => {
                                                            match cond_wait_outcome {
                                                                CondWaitOutcomeView::CwOk => {},
                                                                CondWaitOutcomeView::CwTimedOut => {},
                                                                CondWaitOutcomeView::CwKilled => {},
                                                                CondWaitOutcomeView::CwGenericError { .. } => {},
                                                            }
                                                        },
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
                        },
                    }
                },
            }
        },
    }
}

/// Proof: the spec constant ERROR_CODE_INVALID_ARGUMENT matches ErrorCode::InvalidArgument.
pub proof fn lemma_error_code_matches()
    ensures
        ERROR_CODE_INVALID_ARGUMENT() == ErrorCode::InvalidArgument as int,
{
    assert(ErrorCode::InvalidArgument as int == 22int);
}

/// Proof: architecture guard — USIZE_BITS is 32 and USIZE_MAX matches u32::MAX.
pub proof fn lemma_architecture_guard()
    ensures
        USIZE_BITS() == 32,
        USIZE_MAX_X86_32() == u32::MAX as nat,
        USIZE_MAX_X86_32() == 4294967295nat,
{
}

/// Proof: valid non-MAX inputs produce a finite timeout.
pub proof fn lemma_valid_non_max_is_finite(timeout_s: nat, timeout_ns: nat)
    requires
        !spec_is_infinite_timeout(timeout_s, timeout_ns),
        timeout_ns < NANOS_PER_SEC(),
    ensures
        spec_parse_timeout(timeout_s, timeout_ns) == Some(TimeoutView::Finite {
            seconds: timeout_s,
            nanoseconds: timeout_ns,
        }),
        spec_is_finite_timeout(timeout_s, timeout_ns),
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
{
}

/// Proof: pipeline short-circuit — when timeout is invalid, all PM outcomes are irrelevant.
pub proof fn lemma_pipeline_short_circuit_timeout(
    timeout_s: nat,
    timeout_ns: nat,
    tg1: TakeMutexGuardOutcomeView,
    gc1: GetCondOutcomeView,
    cw1: CondWaitOutcomeView,
    pc1: PutCondOutcomeView,
    gm1: GetMutexOutcomeView,
    lo1: LockOutcomeView,
    pg1: PutGuardOutcomeView,
    tg2: TakeMutexGuardOutcomeView,
    gc2: GetCondOutcomeView,
    cw2: CondWaitOutcomeView,
    pc2: PutCondOutcomeView,
    gm2: GetMutexOutcomeView,
    lo2: LockOutcomeView,
    pg2: PutGuardOutcomeView,
)
    requires
        !spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_wait_cond_result(timeout_s, timeout_ns, tg1, gc1, cw1, pc1, gm1, lo1, pg1)
            == spec_wait_cond_result(timeout_s, timeout_ns, tg2, gc2, cw2, pc2, gm2, lo2, pg2),
{
}

/// Proof: when take_mutex_guard fails, all subsequent outcomes are irrelevant.
pub proof fn lemma_take_guard_short_circuit(
    timeout_s: nat,
    timeout_ns: nat,
    error_code: int,
    gc1: GetCondOutcomeView,
    cw1: CondWaitOutcomeView,
    pc1: PutCondOutcomeView,
    gm1: GetMutexOutcomeView,
    lo1: LockOutcomeView,
    pg1: PutGuardOutcomeView,
    gc2: GetCondOutcomeView,
    cw2: CondWaitOutcomeView,
    pc2: PutCondOutcomeView,
    gm2: GetMutexOutcomeView,
    lo2: LockOutcomeView,
    pg2: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        spec_wait_cond_result(
            timeout_s, timeout_ns,
            TakeMutexGuardOutcomeView::TmgError { error_code },
            gc1, cw1, pc1, gm1, lo1, pg1,
        ) == spec_wait_cond_result(
            timeout_s, timeout_ns,
            TakeMutexGuardOutcomeView::TmgError { error_code },
            gc2, cw2, pc2, gm2, lo2, pg2,
        ),
{
}

/// Proof: the safety preconditions predicate is well-formed.
pub proof fn lemma_safety_preconditions_well_formed(pid: nat, tid: nat)
    ensures
        spec_wait_cond_safety_preconditions(pid, tid) ==> spec_caller_is_not_kernel_process(pid),
        spec_wait_cond_safety_preconditions(pid, tid) ==> spec_caller_holds_no_resources(tid),
        spec_wait_cond_safety_preconditions(pid, tid) ==> spec_caller_no_pm_reference(),
{
}

/// Proof: the cond.wait result is preserved when all subsequent steps succeed.
///
/// # Description
///
/// When get_cond, put_cond, and the entire mutex reacquisition pipeline succeed,
/// the final result exactly reflects the cond.wait outcome:
/// - CwOk → Success
/// - CwTimedOut → CondWaitTimedOut
/// - CwKilled → CondWaitKilled
/// - CwGenericError → CondWaitGenericError
pub proof fn lemma_cond_wait_result_preserved(
    timeout_s: nat,
    timeout_ns: nat,
    cond_wait_outcome: CondWaitOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
    ensures
        ({
            let result: WaitCondResultView = spec_wait_cond_result(
                timeout_s, timeout_ns,
                TakeMutexGuardOutcomeView::TmgOk,
                GetCondOutcomeView::GcOk,
                cond_wait_outcome,
                PutCondOutcomeView::PcOk,
                GetMutexOutcomeView::GmOk,
                LockOutcomeView::LoOk,
                PutGuardOutcomeView::PgOk,
            );
            match cond_wait_outcome {
                CondWaitOutcomeView::CwOk => result == WaitCondResultView::Success,
                CondWaitOutcomeView::CwTimedOut => result == WaitCondResultView::CondWaitTimedOut,
                CondWaitOutcomeView::CwKilled => result == WaitCondResultView::CondWaitKilled,
                CondWaitOutcomeView::CwGenericError { error_code } => {
                    result == WaitCondResultView::CondWaitGenericError { error_code }
                },
            }
        }),
{
    match cond_wait_outcome {
        CondWaitOutcomeView::CwOk => {},
        CondWaitOutcomeView::CwTimedOut => {},
        CondWaitOutcomeView::CwKilled => {},
        CondWaitOutcomeView::CwGenericError { .. } => {},
    }
}

/// Proof: mutex reacquisition uses infinite wait (None timeout), so TimedOut
/// cannot occur in the lock step during reacquisition.
///
/// # Description
///
/// In the original code, `mutex.lock(None)` is called for reacquisition. With
/// no timeout, TimedOut is impossible. When the lock outcome is valid for
/// an infinite timeout, LockTimedOut cannot appear in the final result.
pub proof fn lemma_reacquisition_no_timed_out(
    timeout_s: nat,
    timeout_ns: nat,
    cond_wait_outcome: CondWaitOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
)
    requires
        spec_timeout_parsed_ok(timeout_s, timeout_ns),
        // Lock outcome valid for infinite wait (no timeout → TimedOut impossible).
        !matches!(lock_outcome, LockOutcomeView::LoTimedOut),
    ensures
        !matches!(
            spec_wait_cond_result(
                timeout_s, timeout_ns,
                TakeMutexGuardOutcomeView::TmgOk,
                GetCondOutcomeView::GcOk,
                cond_wait_outcome,
                PutCondOutcomeView::PcOk,
                GetMutexOutcomeView::GmOk,
                lock_outcome,
                put_guard_outcome,
            ),
            WaitCondResultView::LockTimedOut
        ),
{
    match lock_outcome {
        LockOutcomeView::LoOk => {
            match put_guard_outcome {
                PutGuardOutcomeView::PgOk => {
                    match cond_wait_outcome {
                        CondWaitOutcomeView::CwOk => {},
                        CondWaitOutcomeView::CwTimedOut => {},
                        CondWaitOutcomeView::CwKilled => {},
                        CondWaitOutcomeView::CwGenericError { .. } => {},
                    }
                },
                PutGuardOutcomeView::PgError { .. } => {},
            }
        },
        LockOutcomeView::LoKilled => {},
        LockOutcomeView::LoGenericError { .. } => {},
        LockOutcomeView::LoTimedOut => {},
    }
}

} // verus!
