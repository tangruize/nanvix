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
// - The sleep result classification matching the original's 3-arm match:
//   Ok(()) | Interrupted(TimedOut) → Ok(()), Err(other) → Err(other).

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
/// Sourced from ErrorCode::InvalidArgument (repr(i32) = 22).
/// The linking proof `lemma_error_code_matches` in sleep.proof.rs verifies
/// that this constant equals `ErrorCode::InvalidArgument as int`.
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

/// Abstract view of the ProcessManager::sleep result.
///
/// # Description
///
/// Models the four possible outcomes from ProcessManager::sleep,
/// matching the original SleepError/InterruptReason enums:
/// - Ok(()) → PmOk
/// - Err(SleepError::Interrupted(InterruptReason::TimedOut)) → PmTimedOut
/// - Err(SleepError::Interrupted(InterruptReason::Killed)) → PmKilled
/// - Err(SleepError::Generic(error)) → PmGenericError
///
/// The sleep kcall's 3-arm match then classifies these into the final result.
#[verifier::ext_equal]
pub enum PmSleepResultView {
    /// ProcessManager::sleep returned Ok(()).
    PmOk,
    /// ProcessManager::sleep returned Err(Interrupted(TimedOut)).
    PmTimedOut,
    /// ProcessManager::sleep returned Err(Interrupted(Killed)).
    PmKilled,
    /// ProcessManager::sleep returned Err(Generic(error)).
    PmGenericError { error_code: int },
}

/// Abstract view of the sleep kcall's final result.
///
/// # Description
///
/// After the 3-arm match in the original sleep function, the result is either:
/// - Success: Ok(()) or Interrupted(TimedOut) mapped to Ok(())
/// - KilledError: Interrupted(Killed) propagated as Err(SleepError::Interrupted(Killed))
/// - GenericError: Generic error propagated unchanged
#[verifier::ext_equal]
pub enum SleepResultView {
    /// Sleep completed successfully (Ok(()) or TimedOut treated as success).
    Success,
    /// Sleep was interrupted because the process was killed.
    KilledError,
    /// A generic error occurred (e.g., invalid argument, borrow failure).
    ///
    /// ## Reason String Abstraction
    ///
    /// The original `Error::new(ErrorCode::InvalidArgument, "invalid sleep time")`
    /// carries a diagnostic reason string. This model intentionally abstracts it
    /// away because:
    /// 1. The reason string is not semantically observable: callers match on the
    ///    error code (`ErrorCode::InvalidArgument`), not the string.
    /// 2. The string is not returned to userspace — it is used only for kernel
    ///    logging via `error!()`.
    /// 3. The sleep kcall returns `SleepError::Generic(Error { code, reason })`,
    ///    and the caller's match arm `Err(error) => Err(error)` propagates the
    ///    entire `SleepError`, but the error code determines behavior.
    /// Only the numeric error code is semantically relevant for correctness.
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
/// nanoseconds >= 1_000_000_000 by carrying into seconds.
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
/// does not overflow.
pub open spec fn spec_checked_add_succeeds(now: SystemTimeView, timeout: DurationView) -> bool {
    let total_nanos: nat = now.nanoseconds + timeout.nanoseconds;
    let carry: nat = if total_nanos >= NANOS_PER_SEC() { 1 } else { 0 };
    let new_seconds: nat = now.seconds + timeout.seconds + carry;
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
        nanoseconds: if total_nanos >= NANOS_PER_SEC() { (total_nanos - NANOS_PER_SEC()) as nat } else { total_nanos },
    }
}

/// Spec function: models the sleep kcall's 3-arm match on PM result.
///
/// # Description
///
/// Maps the ProcessManager::sleep result to the sleep kcall final result,
/// matching the original code (sleep.rs:62-66):
/// ```ignore
/// match ProcessManager::sleep(Some(alarm)) {
///     Ok(()) => Ok(()),
///     Err(SleepError::Interrupted(InterruptReason::TimedOut)) => Ok(()),
///     Err(error) => Err(error),
/// }
/// ```
/// - PmOk → Success
/// - PmTimedOut → Success (TimedOut treated as normal completion)
/// - PmKilled → KilledError (propagated as error)
/// - PmGenericError → GenericError (propagated unchanged)
pub open spec fn spec_classify_pm_result(pm_result: PmSleepResultView) -> SleepResultView {
    match pm_result {
        PmSleepResultView::PmOk => SleepResultView::Success,
        PmSleepResultView::PmTimedOut => SleepResultView::Success,
        PmSleepResultView::PmKilled => SleepResultView::KilledError,
        PmSleepResultView::PmGenericError { error_code } => SleepResultView::GenericError { error_code },
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
/// 5. Classifies the PM result via the 3-arm match.
pub open spec fn spec_sleep_result(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
    pm_result: PmSleepResultView,
) -> SleepResultView {
    let timeout: DurationView = spec_duration_new(seconds, nanoseconds);
    if !spec_checked_add_succeeds(now, timeout) {
        SleepResultView::GenericError { error_code: ERROR_CODE_INVALID_ARGUMENT() }
    } else {
        spec_classify_pm_result(pm_result)
    }
}

/// Spec function: whether the checked_add for the alarm time succeeds.
pub open spec fn spec_sleep_success_condition(
    now: SystemTimeView,
    seconds: nat,
    nanoseconds: nat,
) -> bool {
    let timeout: DurationView = spec_duration_new(seconds, nanoseconds);
    spec_checked_add_succeeds(now, timeout)
}

/// Spec function: whether a sleep result is success.
pub open spec fn spec_is_success(result: SleepResultView) -> bool {
    matches!(result, SleepResultView::Success)
}

/// Spec function: whether a sleep result is a killed error.
pub open spec fn spec_is_killed(result: SleepResultView) -> bool {
    matches!(result, SleepResultView::KilledError)
}

/// Spec function: whether a sleep result is a generic error.
pub open spec fn spec_is_generic_error(result: SleepResultView) -> bool {
    matches!(result, SleepResultView::GenericError { .. })
}

/// Spec function: whether a sleep result is any kind of error (killed or generic).
pub open spec fn spec_is_error(result: SleepResultView) -> bool {
    spec_is_killed(result) || spec_is_generic_error(result)
}

/// Spec function: Duration::new normalizes nanoseconds.
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
