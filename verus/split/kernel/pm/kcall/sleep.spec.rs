// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Sleep Kernel Call Specification.
// Defines View types, spec constants, and spec functions for the sleep
// kernel call verification model.
//
// ## Verification Model
//
// The sleep kcall converts user-provided (seconds, nanoseconds) into a
// Duration, adds it to the current time to compute an alarm time, and
// then calls ProcessManager::sleep(). This spec models:
// - SystemTime and Duration as abstract types at the verification boundary.
// - The timeout computation as a checked addition.
// - The sleep result classification (Ok, TimedOut, Error).

use vstd::prelude::*;

verus! {

//==================================================================================================
// Spec Constants
//==================================================================================================

/// Maximum value for nanoseconds in a Duration (999_999_999).
pub open spec fn NANOS_PER_SEC() -> nat {
    1_000_000_000
}

/// The ErrorCode value for InvalidArgument.
pub open spec fn ERROR_CODE_INVALID_ARGUMENT() -> int {
    22
}

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a SystemTime value.
///
/// # Description
///
/// Represents a point in time as (seconds, nanoseconds) since the epoch.
/// The nanoseconds component is always < 1_000_000_000.
#[verifier::ext_equal]
pub struct SystemTimeView {
    /// Seconds since epoch.
    pub seconds: nat,
    /// Nanoseconds since last second (< 1_000_000_000).
    pub nanoseconds: nat,
}

/// Abstract view of the Duration parameter.
///
/// # Description
///
/// Represents the sleep timeout duration as (seconds, nanoseconds).
#[verifier::ext_equal]
pub struct DurationView {
    /// Whole seconds component.
    pub seconds: nat,
    /// Fractional nanoseconds component (< 1_000_000_000).
    pub nanoseconds: nat,
}

/// Abstract view of the SleepError result.
///
/// # Description
///
/// Models the three possible outcomes of the sleep kcall.
#[verifier::ext_equal]
pub enum SleepResultView {
    /// Sleep completed successfully (includes TimedOut which is also success).
    Success,
    /// Sleep was interrupted for a non-timeout reason.
    Interrupted,
    /// A generic error occurred (e.g., invalid argument, borrow failure).
    GenericError { error_code: int },
}

//==================================================================================================
// Spec Functions
//==================================================================================================

/// Spec function: well-formedness of SystemTime.
///
/// # Description
///
/// A SystemTime is well-formed if its nanoseconds component is within range.
pub open spec fn spec_system_time_wf(st: SystemTimeView) -> bool {
    st.nanoseconds < NANOS_PER_SEC()
}

/// Spec function: well-formedness of Duration.
///
/// # Description
///
/// A Duration is well-formed if its nanoseconds component is within range.
pub open spec fn spec_duration_wf(d: DurationView) -> bool {
    d.nanoseconds < NANOS_PER_SEC()
}

/// Spec function: models Duration::new(seconds, nanoseconds).
///
/// # Description
///
/// Creates a DurationView from the user-provided seconds and nanoseconds.
/// In the original code, `Duration::new(secs as u64, nanos as u32)` handles
/// nanoseconds >= 1_000_000_000 by carrying into seconds. For Verus modeling,
/// we assume the standard library semantics.
pub open spec fn spec_duration_new(seconds: nat, nanoseconds: nat) -> DurationView {
    DurationView {
        seconds: seconds + nanoseconds / NANOS_PER_SEC(),
        nanoseconds: nanoseconds % NANOS_PER_SEC(),
    }
}

/// Spec function: whether checked_add_duration succeeds.
///
/// # Description
///
/// Models `SystemTime::checked_add_duration()`. Returns true if the addition
/// does not overflow. The exact overflow condition depends on the SystemTime
/// internal representation.
pub open spec fn spec_checked_add_succeeds(now: SystemTimeView, timeout: DurationView) -> bool {
    // The addition succeeds if the total nanoseconds and seconds don't overflow u64/u32 limits.
    let total_nanos: nat = now.nanoseconds + timeout.nanoseconds;
    let carry: nat = if total_nanos >= NANOS_PER_SEC() { 1 } else { 0 };
    let new_seconds: nat = now.seconds + timeout.seconds + carry;
    // Must fit in u64 for seconds and the resulting nanoseconds must be valid.
    new_seconds <= u64::MAX as nat
}

/// Spec function: computes the alarm time from now + timeout.
///
/// # Description
///
/// Models the result of `now.checked_add_duration(&timeout)` when it succeeds.
pub open spec fn spec_compute_alarm(now: SystemTimeView, timeout: DurationView) -> SystemTimeView
    recommends spec_checked_add_succeeds(now, timeout)
{
    let total_nanos: nat = now.nanoseconds + timeout.nanoseconds;
    let carry: nat = if total_nanos >= NANOS_PER_SEC() { 1 } else { 0 };
    SystemTimeView {
        seconds: now.seconds + timeout.seconds + carry,
        nanoseconds: if total_nanos >= NANOS_PER_SEC() { total_nanos - NANOS_PER_SEC() } else { total_nanos },
    }
}

/// Spec function: models the overall sleep kcall logic.
///
/// # Description
///
/// The sleep function:
/// 1. Gets current time (now).
/// 2. Computes timeout Duration from (seconds, nanoseconds).
/// 3. Checks if now + timeout overflows → InvalidArgument error.
/// 4. Calls ProcessManager::sleep(Some(alarm)).
/// 5. Returns Ok(()) on success or TimedOut, propagates other errors.
///
/// This spec models the high-level control flow.
pub open spec fn spec_sleep_result(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
    pm_result: SleepResultView,
) -> SleepResultView {
    let timeout: DurationView = spec_duration_new(seconds, nanoseconds);
    if !spec_checked_add_succeeds(now, timeout) {
        SleepResultView::GenericError { error_code: ERROR_CODE_INVALID_ARGUMENT() }
    } else {
        match pm_result {
            SleepResultView::Success => SleepResultView::Success,
            SleepResultView::Interrupted => SleepResultView::Success,  // TimedOut → Ok(())
            SleepResultView::GenericError { error_code } => SleepResultView::GenericError { error_code },
        }
    }
}

/// Spec function: the sleep function returns Ok for valid inputs when PM succeeds.
///
/// # Description
///
/// If the timeout addition succeeds and ProcessManager::sleep returns Ok or TimedOut,
/// the sleep kcall returns success.
pub open spec fn spec_sleep_success_condition(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
) -> bool {
    let timeout: DurationView = spec_duration_new(seconds, nanoseconds);
    spec_checked_add_succeeds(now, timeout)
}

/// Spec function: classifies the sleep result.
///
/// # Description
///
/// Returns true if the result represents a successful sleep (either direct
/// Ok or TimedOut interruption treated as success).
pub open spec fn spec_is_success(result: SleepResultView) -> bool {
    matches!(result, SleepResultView::Success)
}

/// Spec function: classifies the sleep result as error.
///
/// # Description
///
/// Returns true if the result represents an error condition.
pub open spec fn spec_is_error(result: SleepResultView) -> bool {
    matches!(result, SleepResultView::GenericError { .. })
}

/// Spec function: Duration::new normalizes nanoseconds.
///
/// # Description
///
/// The Duration::new function carries nanoseconds >= 1_000_000_000 into seconds.
/// The resulting nanoseconds are always < 1_000_000_000.
pub open spec fn spec_duration_new_wf(seconds: nat, nanoseconds: nat) -> bool {
    spec_duration_wf(spec_duration_new(seconds, nanoseconds))
}

/// Spec function: the alarm time is well-formed when checked_add succeeds.
pub open spec fn spec_alarm_wf(now: SystemTimeView, timeout: DurationView) -> bool
    recommends
        spec_system_time_wf(now),
        spec_duration_wf(timeout),
        spec_checked_add_succeeds(now, timeout),
{
    spec_system_time_wf(spec_compute_alarm(now, timeout))
}

} // verus!
