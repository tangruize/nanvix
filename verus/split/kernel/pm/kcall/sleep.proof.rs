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
    // The nanoseconds field is `nanoseconds % NANOS_PER_SEC()`, which is < NANOS_PER_SEC().
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
    // Case split on whether nanoseconds overflow.
    if total_nanos >= NANOS_PER_SEC() {
        // Carry case: new_nanos = total_nanos - NANOS_PER_SEC().
        // Since now.nanoseconds < NANOS_PER_SEC() and timeout.nanoseconds < NANOS_PER_SEC(),
        // total_nanos < 2 * NANOS_PER_SEC(), so new_nanos < NANOS_PER_SEC().
        assert(now.nanoseconds < NANOS_PER_SEC());
        assert(timeout.nanoseconds < NANOS_PER_SEC());
        assert(total_nanos < 2 * NANOS_PER_SEC());
        assert(total_nanos - NANOS_PER_SEC() < NANOS_PER_SEC());
    } else {
        // No carry: new_nanos = total_nanos < NANOS_PER_SEC().
        assert(total_nanos < NANOS_PER_SEC());
    }
}

/// Proof: invalid argument error when checked_add fails.
///
/// # Description
///
/// When the timeout addition overflows, the sleep function returns
/// GenericError with InvalidArgument error code. This is independent
/// of the ProcessManager result.
pub proof fn lemma_overflow_returns_invalid_argument(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
    pm_result: SleepResultView,
)
    requires
        !spec_sleep_success_condition(now, seconds, nanoseconds),
    ensures
        spec_sleep_result(now, seconds, nanoseconds, pm_result) ==
            (SleepResultView::GenericError { error_code: ERROR_CODE_INVALID_ARGUMENT() }),
{
    // Follows directly from the spec_sleep_result definition:
    // when !spec_checked_add_succeeds, the result is GenericError.
}

/// Proof: TimedOut interruption is treated as success.
///
/// # Description
///
/// When the timeout addition succeeds and ProcessManager::sleep returns
/// Interrupted (TimedOut), the sleep kcall treats it as success (Ok(())).
pub proof fn lemma_timed_out_is_success(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
)
    requires
        spec_sleep_success_condition(now, seconds, nanoseconds),
    ensures
        spec_is_success(
            spec_sleep_result(now, seconds, nanoseconds, SleepResultView::Interrupted)
        ),
{
    // Follows from spec_sleep_result: Interrupted maps to Success.
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
            spec_sleep_result(now, seconds, nanoseconds, SleepResultView::Success)
        ),
{
    // Follows from spec_sleep_result: Success maps to Success.
}

/// Proof: PM generic error propagates through.
///
/// # Description
///
/// When ProcessManager::sleep returns a generic error, it is propagated
/// through the sleep kcall unchanged.
pub proof fn lemma_pm_error_propagates(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
    error_code: int,
)
    requires
        spec_sleep_success_condition(now, seconds, nanoseconds),
    ensures
        spec_sleep_result(now, seconds, nanoseconds, SleepResultView::GenericError { error_code }) ==
            (SleepResultView::GenericError { error_code }),
{
    // Follows from spec_sleep_result: GenericError passes through.
}

/// Proof: the sleep result is always either success or error.
///
/// # Description
///
/// The result of spec_sleep_result is always one of the two categories:
/// success or generic error. There is no third possibility because
/// the Interrupted case from PM is folded into Success.
pub proof fn lemma_sleep_result_dichotomy(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
    pm_result: SleepResultView,
)
    ensures
        spec_is_success(spec_sleep_result(now, seconds, nanoseconds, pm_result))
        || spec_is_error(spec_sleep_result(now, seconds, nanoseconds, pm_result)),
{
    let timeout: DurationView = spec_duration_new(seconds, nanoseconds);
    if !spec_checked_add_succeeds(now, timeout) {
        // Overflow case: always GenericError.
        assert(spec_is_error(spec_sleep_result(now, seconds, nanoseconds, pm_result)));
    } else {
        // Non-overflow case: depends on pm_result.
        match pm_result {
            SleepResultView::Success => {
                assert(spec_is_success(spec_sleep_result(now, seconds, nanoseconds, pm_result)));
            },
            SleepResultView::Interrupted => {
                assert(spec_is_success(spec_sleep_result(now, seconds, nanoseconds, pm_result)));
            },
            SleepResultView::GenericError { .. } => {
                assert(spec_is_error(spec_sleep_result(now, seconds, nanoseconds, pm_result)));
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
    // checked_add: total_nanos = now.nanoseconds + 0 = now.nanoseconds < NANOS_PER_SEC().
    // carry = 0, new_seconds = now.seconds + 0 + 0 = now.seconds <= u64::MAX.
    assert(spec_checked_add_succeeds(now, timeout));
}

} // verus!
