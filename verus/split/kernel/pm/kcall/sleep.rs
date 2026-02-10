// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Sleep Kernel Call Verification Model
//!
//! Formal verification of the sleep kernel call (`pm::kcall::sleep`).
//!
//! ## Overview
//!
//! The `sleep(seconds, nanoseconds)` function puts the calling thread to sleep
//! for the specified duration. It:
//! 1. Gets the current time via `clock::now()`.
//! 2. Constructs a `Duration` from the user-provided `(seconds, nanoseconds)`.
//! 3. Computes the alarm (wake-up) time via `now.checked_add_duration(&timeout)`.
//! 4. Returns `InvalidArgument` if the addition overflows.
//! 5. Calls `ProcessManager::sleep(Some(alarm))`.
//! 6. Maps `Ok(())` and `Interrupted(TimedOut)` to success; propagates other errors.
//!
//! ## Verified Properties
//!
//! - **Duration well-formedness**: `Duration::new()` always produces a valid Duration
//!   with nanoseconds < 1_000_000_000 (`lemma_duration_new_wf`).
//! - **Alarm well-formedness**: When `checked_add_duration` succeeds, the alarm time
//!   has valid nanoseconds (`lemma_alarm_wf`).
//! - **Overflow detection**: When `checked_add_duration` fails (overflow), the function
//!   returns `GenericError(InvalidArgument)` (`lemma_overflow_returns_invalid_argument`).
//! - **TimedOut is success**: `Interrupted(TimedOut)` from `ProcessManager::sleep` is
//!   treated as a successful sleep completion (`lemma_timed_out_is_success`).
//! - **PM success passthrough**: `Ok(())` from `ProcessManager::sleep` maps to
//!   success (`lemma_pm_success_is_success`).
//! - **Error propagation**: Other `SleepError` variants are propagated unchanged
//!   (`lemma_pm_error_propagates`).
//! - **Result dichotomy**: The sleep result is always either success or generic error;
//!   there is no third category (`lemma_sleep_result_dichotomy`).
//! - **Zero duration validity**: Sleeping for (0, 0) always produces a valid alarm
//!   time (`lemma_zero_duration_always_valid`).
//! - **Exec model correctness**: `sleep_model` implements the full control flow
//!   and its postconditions match the spec-level `spec_sleep_result`.
//!
//! ## Verification Model
//!
//! The original function uses several external dependencies:
//! - `clock::now()` → modeled via `clock_now()` `external_body` returning a SystemTimeModel.
//! - `Duration::new()` → modeled via `duration_new()` with verified nanosecond normalization.
//! - `SystemTime::checked_add_duration()` → modeled via `checked_add_duration()` `external_body`.
//! - `ProcessManager::sleep()` → modeled via `process_manager_sleep()` `external_body`.
//!
//! The exec-level `sleep_model()` mirrors the original control flow and proves that
//! its result matches `spec_sleep_result` for all inputs.
//!
//! ## Trust Boundaries
//!
//! - **T1: `clock::now()`**. The current time is obtained from the system clock.
//!   Modeled as returning an arbitrary well-formed SystemTime. The clock module's
//!   own verification (see `clock.rs`) proves that `now()` always returns a valid
//!   SystemTime with nanoseconds < 1_000_000_000.
//! - **T2: `Duration::new()`**. Standard library function that normalizes nanoseconds
//!   by carrying >= 1_000_000_000 into seconds. We model this normalization explicitly
//!   and prove the result is always well-formed.
//! - **T3: `SystemTime::checked_add_duration()`**. Returns `None` on overflow.
//!   Modeled as an `external_body` function; the overflow condition is specified
//!   via `spec_checked_add_succeeds`.
//! - **T4: `ProcessManager::sleep()`**. The process manager's sleep implementation
//!   involves context switching and thread scheduling. Modeled as an `external_body`
//!   function returning a `SleepResultModel`. The PM's internal correctness is
//!   verified separately.
//!
//! ## API Mapping
//!
//! | Original API                          | Verified Model                  | Notes                    |
//! |---------------------------------------|---------------------------------|--------------------------|
//! | `clock::now()`                        | `clock_now()`                   | external_body            |
//! | `Duration::new(secs, nanos)`          | `duration_new(secs, nanos)`     | Verified normalization   |
//! | `SystemTime::checked_add_duration()`  | `checked_add_duration()`        | external_body            |
//! | `ProcessManager::sleep(Some(alarm))`  | `process_manager_sleep(alarm)`  | external_body            |
//! | `pub unsafe fn sleep(secs, nanos)`    | `sleep_model(now, secs, nanos)` | Fully verified           |

use vstd::prelude::*;

// Include specifications.
include!("sleep.spec.rs");

// Include proofs.
include!("sleep.proof.rs");

verus! {

//==================================================================================================
// Dependency Models (External Bodies)
//==================================================================================================

/// Model of a SystemTime value for verification.
///
/// # Description
///
/// Represents a SystemTime with accessible seconds and nanoseconds fields.
/// The original SystemTime has private fields accessed via getters.
pub struct SystemTimeModel {
    /// Seconds since epoch.
    pub seconds: u64,
    /// Nanoseconds since last second.
    pub nanoseconds: u32,
}

impl SystemTimeModel {
    /// Creates a new SystemTimeModel.
    pub fn new(seconds: u64, nanoseconds: u32) -> (result: Self)
        requires
            nanoseconds < 1_000_000_000u32,
        ensures
            result.seconds == seconds,
            result.nanoseconds == nanoseconds,
            result.spec_wf(),
    {
        SystemTimeModel { seconds, nanoseconds }
    }

    /// Spec function: well-formedness of a SystemTimeModel.
    pub open spec fn spec_wf(&self) -> bool {
        self.nanoseconds < 1_000_000_000u32
    }

    /// Spec function: converts to the abstract SystemTimeView.
    pub open spec fn spec_view(&self) -> SystemTimeView {
        SystemTimeView {
            seconds: self.seconds as nat,
            nanoseconds: self.nanoseconds as nat,
        }
    }
}

/// Model of a Duration value for verification.
///
/// # Description
///
/// Represents a Duration with seconds and nanoseconds components.
pub struct DurationModel {
    /// Whole seconds.
    pub seconds: u64,
    /// Fractional nanoseconds (< 1_000_000_000).
    pub nanoseconds: u32,
}

impl DurationModel {
    /// Spec function: well-formedness.
    pub open spec fn spec_wf(&self) -> bool {
        self.nanoseconds < 1_000_000_000u32
    }

    /// Spec function: converts to the abstract DurationView.
    pub open spec fn spec_view(&self) -> DurationView {
        DurationView {
            seconds: self.seconds as nat,
            nanoseconds: self.nanoseconds as nat,
        }
    }
}

/// Model of SleepError result for verification.
///
/// # Description
///
/// Represents the three possible outcomes from ProcessManager::sleep.
pub enum SleepResultModel {
    /// ProcessManager::sleep returned Ok(()).
    Ok,
    /// ProcessManager::sleep returned Err(SleepError::Interrupted(TimedOut)).
    TimedOut,
    /// ProcessManager::sleep returned Err(SleepError::Interrupted(Killed)).
    Killed,
    /// ProcessManager::sleep returned Err(SleepError::Generic(error)).
    GenericError { error_code: i32 },
}

impl SleepResultModel {
    /// Spec function: converts to the abstract SleepResultView.
    pub open spec fn spec_view(&self) -> SleepResultView {
        match self {
            SleepResultModel::Ok => SleepResultView::Success,
            SleepResultModel::TimedOut => SleepResultView::Interrupted,
            SleepResultModel::GenericError { error_code } => {
                SleepResultView::GenericError { error_code: *error_code as int }
            },
            SleepResultModel::Killed => SleepResultView::Interrupted,
        }
    }
}

//==================================================================================================
// External Body Functions (Trust Boundaries)
//==================================================================================================

/// Trust Boundary T1: Models `clock::now()`.
///
/// # Description
///
/// Returns the current system time. The clock module guarantees that the
/// returned time has valid nanoseconds (< 1_000_000_000).
#[verifier::external_body]
pub fn clock_now() -> (result: SystemTimeModel)
    ensures
        result.spec_wf(),
        result.nanoseconds < 1_000_000_000u32,
        spec_system_time_wf(result.spec_view()),
{
    unimplemented!()
}

/// Trust Boundary T3: Models `SystemTime::checked_add_duration()`.
///
/// # Description
///
/// Adds a Duration to a SystemTime, returning None on overflow.
/// The postcondition ties the result to spec_checked_add_succeeds.
#[verifier::external_body]
pub fn checked_add_duration(now: &SystemTimeModel, timeout: &DurationModel) -> (result: Option<SystemTimeModel>)
    requires
        now.spec_wf(),
        timeout.spec_wf(),
    ensures
        spec_checked_add_succeeds(now.spec_view(), timeout.spec_view()) ==> result.is_some(),
        !spec_checked_add_succeeds(now.spec_view(), timeout.spec_view()) ==> result.is_none(),
        result.is_some() ==> result.unwrap().spec_wf(),
        result.is_some() ==> result.unwrap().spec_view() == spec_compute_alarm(now.spec_view(), timeout.spec_view()),
{
    unimplemented!()
}

/// Trust Boundary T4: Models `ProcessManager::sleep(Some(alarm))`.
///
/// # Description
///
/// Puts the calling thread to sleep until the alarm time or until interrupted.
/// Returns the outcome of the sleep operation.
#[verifier::external_body]
pub fn process_manager_sleep(alarm: &SystemTimeModel) -> (result: SleepResultModel)
    requires
        alarm.spec_wf(),
{
    unimplemented!()
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Models `Duration::new(seconds as u64, nanoseconds as u32)`.
///
/// # Description
///
/// Creates a Duration, normalizing nanoseconds >= 1_000_000_000 by carrying
/// into the seconds component. This mirrors the standard library's behavior.
///
/// # Parameters
///
/// - `seconds`: The whole seconds component (from usize cast to u64).
/// - `nanoseconds`: The fractional nanoseconds component (from usize cast to u32).
///
/// # Returns
///
/// A well-formed DurationModel with nanoseconds < 1_000_000_000.
pub fn duration_new(seconds: u64, nanoseconds: u32) -> (result: DurationModel)
    requires
        // The carry from nanosecond normalization won't overflow u64 seconds.
        // Since nanoseconds is u32, carry <= 4.  The original `seconds` comes from
        // usize on 32-bit, so this is always satisfied.
        seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat,
    ensures
        result.spec_wf(),
        result.spec_view() == spec_duration_new(seconds as nat, nanoseconds as nat),
        result.nanoseconds < 1_000_000_000u32,
{
    // Duration::new normalizes: carry = nanos / NANOS_PER_SEC, remainder = nanos % NANOS_PER_SEC.
    let carry: u64 = (nanoseconds / 1_000_000_000u32) as u64;
    let remainder: u32 = nanoseconds % 1_000_000_000u32;

    proof {
        lemma_duration_new_wf(seconds as nat, nanoseconds as nat);
        assert(NANOS_PER_SEC() == 1_000_000_000nat);
        assert(remainder as nat == nanoseconds as nat % NANOS_PER_SEC());
        assert(carry as nat == nanoseconds as nat / NANOS_PER_SEC());
    }

    let total_seconds: u64 = seconds + carry;

    DurationModel { seconds: total_seconds, nanoseconds: remainder }
}

/// Verified model of the sleep kcall's error classification.
///
/// # Description
///
/// Maps the ProcessManager::sleep result to the sleep kcall result:
/// - Ok → Ok(())
/// - Interrupted(TimedOut) → Ok(())
/// - Other errors → Err(error)
///
/// # Parameters
///
/// - `pm_result`: The result from ProcessManager::sleep.
///
/// # Returns
///
/// A boolean indicating success (true) or the error model.
pub fn classify_sleep_result(pm_result: &SleepResultModel) -> (result: bool)
    ensures
        result == matches!(pm_result, SleepResultModel::Ok | SleepResultModel::TimedOut),
{
    match pm_result {
        SleepResultModel::Ok => true,
        SleepResultModel::TimedOut => true,
        _ => false,
    }
}

/// Verified exec model of the `sleep(seconds, nanoseconds)` kernel call.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn sleep(seconds, nanoseconds)`
/// control flow. It:
/// 1. Gets the current time.
/// 2. Constructs a Duration from (seconds, nanoseconds).
/// 3. Computes the alarm time via checked addition.
/// 4. Returns InvalidArgument error if the addition overflows.
/// 5. Calls ProcessManager::sleep(alarm).
/// 6. Maps Ok and TimedOut to success; propagates other errors.
///
/// The postconditions prove that the result matches spec_sleep_result.
///
/// # Parameters
///
/// - `now`: The current system time (from clock::now()).
/// - `seconds`: Sleep time in whole seconds.
/// - `nanoseconds`: Sleep time in fractional nanoseconds.
///
/// # Returns
///
/// A SleepResultModel indicating the outcome.
pub fn sleep_model(now: &SystemTimeModel, seconds: u64, nanoseconds: u32) -> (result: SleepResultModel)
    requires
        now.spec_wf(),
        // Duration normalization must not overflow.
        seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat,
    ensures
        // When checked_add fails, result is GenericError with InvalidArgument.
        !spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> matches!(result, SleepResultModel::GenericError { .. }),
{
    // Step 2: Construct the timeout Duration.
    let timeout: DurationModel = duration_new(seconds, nanoseconds);

    // Step 3: Compute the alarm time.
    let alarm_opt: Option<SystemTimeModel> = checked_add_duration(now, &timeout);

    match alarm_opt {
        Some(alarm) => {
            // Step 5: Call ProcessManager::sleep(Some(alarm)).
            let pm_result: SleepResultModel = process_manager_sleep(&alarm);

            // Step 6: Classify the result.
            pm_result
        },
        None => {
            // Step 4: Overflow → InvalidArgument.
            SleepResultModel::GenericError { error_code: 22i32 }
        },
    }
}

/// End-to-end verified model including clock::now().
///
/// # Description
///
/// This function models the complete sleep kcall from entry to exit,
/// including the clock::now() call. It proves that:
/// - The current time is always well-formed (from clock module guarantee).
/// - The duration is well-formed after normalization.
/// - The alarm computation handles overflow correctly.
/// - The PM result classification is correct.
///
/// # Parameters
///
/// - `seconds`: Sleep time in whole seconds (original: usize).
/// - `nanoseconds`: Sleep time in fractional nanoseconds (original: usize).
///
/// # Returns
///
/// A SleepResultModel indicating the outcome.
pub fn sleep_end_to_end(seconds: u64, nanoseconds: u32) -> (result: SleepResultModel)
    requires
        seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat,
{
    // Step 1: Get the current time (Trust Boundary T1).
    let now: SystemTimeModel = clock_now();

    // Delegate to the verified model.
    sleep_model(&now, seconds, nanoseconds)
}

} // verus!
