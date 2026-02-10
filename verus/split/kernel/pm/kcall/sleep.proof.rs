// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Sleep Kernel Call Proofs.
// Proof lemmas for the sleep kcall verification model.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Proof Functions
//==================================================================================================

/// Proof: Duration::new always produces a well-formed Duration.
///
/// # Description
///
/// For any (seconds, nanoseconds) input, the resulting DurationView has
/// nanoseconds < NANOS_PER_SEC(). This follows from the modulus operation.
pub proof fn lemma_duration_new_wf(seconds: nat, nanoseconds: nat)
    ensures
        spec_duration_wf(spec_duration_new(seconds, nanoseconds)),
        spec_duration_new_wf(seconds, nanoseconds),
{
    assert(NANOS_PER_SEC() > 0);
    assert(nanoseconds % NANOS_PER_SEC() < NANOS_PER_SEC());
}

/// Proof: when checked_add succeeds, the alarm time is well-formed.
///
/// # Description
///
/// If both the source SystemTime and Duration are well-formed, and the
/// checked addition succeeds, the resulting alarm SystemTime is also
/// well-formed (nanoseconds < NANOS_PER_SEC()).
pub proof fn lemma_alarm_wf(now: SystemTimeView, timeout: DurationView)
    requires
        spec_system_time_wf(now),
        spec_duration_wf(timeout),
        spec_checked_add_succeeds(now, timeout),
    ensures
        spec_alarm_wf(now, timeout),
        spec_system_time_wf(spec_compute_alarm(now, timeout)),
{
    let total_nanos: nat = now.nanoseconds + timeout.nanoseconds;
    if total_nanos >= NANOS_PER_SEC() {
        assert(now.nanoseconds < NANOS_PER_SEC());
        assert(timeout.nanoseconds < NANOS_PER_SEC());
        assert(total_nanos < 2 * NANOS_PER_SEC());
        assert(total_nanos - NANOS_PER_SEC() < NANOS_PER_SEC());
    } else {
        assert(total_nanos < NANOS_PER_SEC());
    }
}

/// Proof: invalid argument error when checked_add fails.
///
/// # Description
///
/// When the timeout addition overflows, the sleep function returns
/// GenericError with InvalidArgument error code (22). This is independent
/// of the ProcessManager result.
pub proof fn lemma_overflow_returns_invalid_argument(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
    pm_result: PmSleepResultView,
)
    requires
        !spec_sleep_success_condition(now, seconds, nanoseconds),
    ensures
        spec_sleep_result(now, seconds, nanoseconds, pm_result) ==
            (SleepResultView::GenericError { error_code: ERROR_CODE_INVALID_ARGUMENT() }),
{
}

/// Proof: TimedOut interruption is treated as success.
///
/// # Description
///
/// When the timeout addition succeeds and ProcessManager::sleep returns
/// PmTimedOut, the sleep kcall treats it as success (Ok(())).
/// This matches the original: `Err(SleepError::Interrupted(InterruptReason::TimedOut)) => Ok(())`.
pub proof fn lemma_timed_out_is_success(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
)
    requires
        spec_sleep_success_condition(now, seconds, nanoseconds),
    ensures
        spec_is_success(
            spec_sleep_result(now, seconds, nanoseconds, PmSleepResultView::PmTimedOut)
        ),
{
}

/// Proof: Killed interruption is propagated as an error.
///
/// # Description
///
/// When the timeout addition succeeds and ProcessManager::sleep returns
/// PmKilled, the sleep kcall propagates it as KilledError.
/// This matches the original: `Err(error) => Err(error)` catching Interrupted(Killed).
pub proof fn lemma_killed_is_error(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
)
    requires
        spec_sleep_success_condition(now, seconds, nanoseconds),
    ensures
        spec_is_killed(
            spec_sleep_result(now, seconds, nanoseconds, PmSleepResultView::PmKilled)
        ),
        spec_is_error(
            spec_sleep_result(now, seconds, nanoseconds, PmSleepResultView::PmKilled)
        ),
        !spec_is_success(
            spec_sleep_result(now, seconds, nanoseconds, PmSleepResultView::PmKilled)
        ),
{
}

/// Proof: PM success maps to sleep success.
///
/// # Description
///
/// When the timeout addition succeeds and ProcessManager::sleep returns Ok,
/// the sleep kcall returns success.
pub proof fn lemma_pm_success_is_success(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
)
    requires
        spec_sleep_success_condition(now, seconds, nanoseconds),
    ensures
        spec_is_success(
            spec_sleep_result(now, seconds, nanoseconds, PmSleepResultView::PmOk)
        ),
{
}

/// Proof: PM generic error propagates through unchanged.
///
/// # Description
///
/// When ProcessManager::sleep returns a generic error, it is propagated
/// through the sleep kcall with the same error code.
pub proof fn lemma_pm_error_propagates(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
    error_code: int,
)
    requires
        spec_sleep_success_condition(now, seconds, nanoseconds),
    ensures
        spec_sleep_result(now, seconds, nanoseconds, PmSleepResultView::PmGenericError { error_code }) ==
            (SleepResultView::GenericError { error_code }),
        spec_is_generic_error(
            spec_sleep_result(now, seconds, nanoseconds, PmSleepResultView::PmGenericError { error_code })
        ),
{
}

/// Proof: the sleep result is always exactly one of Success, KilledError, or GenericError.
///
/// # Description
///
/// The result of spec_sleep_result is exhaustive: every possible (now, seconds, nanoseconds,
/// pm_result) combination yields exactly one of the three result categories. No result
/// goes unclassified, and the categories are mutually exclusive.
pub proof fn lemma_sleep_result_exhaustive(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
    pm_result: PmSleepResultView,
)
    ensures
        // Exhaustive: every result is success, killed, or generic error.
        spec_is_success(spec_sleep_result(now, seconds, nanoseconds, pm_result))
        || spec_is_killed(spec_sleep_result(now, seconds, nanoseconds, pm_result))
        || spec_is_generic_error(spec_sleep_result(now, seconds, nanoseconds, pm_result)),
        // Mutual exclusion: success and error are disjoint.
        !(spec_is_success(spec_sleep_result(now, seconds, nanoseconds, pm_result))
          && spec_is_error(spec_sleep_result(now, seconds, nanoseconds, pm_result))),
        // Mutual exclusion: killed and generic error are disjoint.
        !(spec_is_killed(spec_sleep_result(now, seconds, nanoseconds, pm_result))
          && spec_is_generic_error(spec_sleep_result(now, seconds, nanoseconds, pm_result))),
{
    let timeout: DurationView = spec_duration_new(seconds, nanoseconds);
    if !spec_checked_add_succeeds(now, timeout) {
        assert(spec_is_generic_error(spec_sleep_result(now, seconds, nanoseconds, pm_result)));
    } else {
        match pm_result {
            PmSleepResultView::PmOk => {
                assert(spec_is_success(spec_sleep_result(now, seconds, nanoseconds, pm_result)));
            },
            PmSleepResultView::PmTimedOut => {
                assert(spec_is_success(spec_sleep_result(now, seconds, nanoseconds, pm_result)));
            },
            PmSleepResultView::PmKilled => {
                assert(spec_is_killed(spec_sleep_result(now, seconds, nanoseconds, pm_result)));
            },
            PmSleepResultView::PmGenericError { .. } => {
                assert(spec_is_generic_error(spec_sleep_result(now, seconds, nanoseconds, pm_result)));
            },
        }
    }
}

/// Proof: zero duration always succeeds checked_add (if now is valid).
///
/// # Description
///
/// Sleeping for zero seconds and zero nanoseconds always produces a valid
/// alarm time equal to the current time.
pub proof fn lemma_zero_duration_always_valid(now: SystemTimeView)
    requires
        spec_system_time_wf(now),
        now.seconds <= u64::MAX as nat,
    ensures
        spec_sleep_success_condition(now, 0, 0),
{
    let timeout: DurationView = spec_duration_new(0, 0);
    assert(timeout.seconds == 0nat);
    assert(timeout.nanoseconds == 0nat);
    assert(spec_checked_add_succeeds(now, timeout));
}

/// Proof: TimedOut and Ok are the only PM results that produce success.
///
/// # Description
///
/// When checked_add succeeds, only PmOk and PmTimedOut produce Success.
/// PmKilled produces KilledError, PmGenericError produces GenericError.
/// This proves the result classification is exact.
pub proof fn lemma_success_only_from_ok_or_timed_out(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
    pm_result: PmSleepResultView,
)
    requires
        spec_sleep_success_condition(now, seconds, nanoseconds),
    ensures
        spec_is_success(spec_sleep_result(now, seconds, nanoseconds, pm_result))
            <==> matches!(pm_result, PmSleepResultView::PmOk | PmSleepResultView::PmTimedOut),
{
}

/// Proof: the spec constant ERROR_CODE_INVALID_ARGUMENT matches ErrorCode::InvalidArgument.
///
/// # Description
///
/// Links the spec-level error code constant to the concrete `ErrorCode::InvalidArgument`
/// enum discriminant. This ensures the spec cannot silently drift if the error code
/// mapping changes.
pub proof fn lemma_error_code_matches()
    ensures
        ERROR_CODE_INVALID_ARGUMENT() == ErrorCode::InvalidArgument as int,
{
    assert(ErrorCode::InvalidArgument as int == 22int);
}

} // verus!
