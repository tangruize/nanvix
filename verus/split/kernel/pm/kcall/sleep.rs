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
//! 6. Classifies the result via a 3-arm match:
//!    - `Ok(())` → `Ok(())`
//!    - `Err(Interrupted(TimedOut))` → `Ok(())`
//!    - `Err(error)` → `Err(error)` (propagates Killed and Generic errors)
//!
//! ## Verified Properties
//!
//! - **Duration well-formedness**: `Duration::new()` always produces a valid Duration
//!   with nanoseconds < 1_000_000_000 (`lemma_duration_new_wf`).
//! - **Alarm well-formedness**: When `checked_add_duration` succeeds, the alarm time
//!   has valid nanoseconds (`lemma_alarm_wf`).
//! - **Overflow detection**: When `checked_add_duration` fails (overflow), the function
//!   returns `GenericError(InvalidArgument)` (`lemma_overflow_returns_invalid_argument`).
//! - **TimedOut is success**: Only `PmTimedOut` from `ProcessManager::sleep` is
//!   treated as success (`lemma_timed_out_is_success`).
//! - **Killed is error**: `PmKilled` is propagated as `KilledError`, NOT as success
//!   (`lemma_killed_is_error`).
//! - **PM success passthrough**: `PmOk` maps to success (`lemma_pm_success_is_success`).
//! - **Error propagation**: `PmGenericError` is propagated unchanged
//!   (`lemma_pm_error_propagates`).
//! - **Result trichotomy**: The sleep result is always exactly one of Success,
//!   KilledError, or GenericError (`lemma_sleep_result_trichotomy`).
//! - **Success only from Ok or TimedOut**: When checked_add succeeds, the result is
//!   Success if and only if the PM result was PmOk or PmTimedOut
//!   (`lemma_success_only_from_ok_or_timed_out`).
//! - **Zero duration validity**: Sleeping for (0, 0) always produces a valid alarm
//!   time (`lemma_zero_duration_always_valid`).
//! - **Exec model full correctness**: `sleep_model` implements the 3-arm match
//!   and its postconditions tie the result to `spec_sleep_result` for ALL paths
//!   (overflow, success, and error).
//!
//! ## Verification Model
//!
//! The original function uses several external dependencies:
//! - `clock::now()` → modeled via `clock_now()` `external_body` returning a SystemTimeModel.
//! - `Duration::new()` → modeled via `duration_new()` with verified nanosecond normalization.
//! - `SystemTime::checked_add_duration()` → modeled via `checked_add_duration()` `external_body`.
//! - `ProcessManager::sleep()` → modeled via `process_manager_sleep()` `external_body`.
//!
//! The exec-level `sleep_model()` mirrors the original control flow including the
//! 3-arm match on the PM result, and proves that the result matches `spec_sleep_result`
//! for all inputs and all PM outcomes.
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
//! - **T5: `usize` to `u64`/`u32` cast**. The original takes `(usize, usize)` and
//!   casts to `(u64, u32)`. On Nanvix's x86-32 target, usize is 32 bits, so the
//!   cast always fits. The model takes `(u64, u32)` with a precondition
//!   `seconds <= u32::MAX as u64` to match the 32-bit origin. Cast safety is
//!   at the ABI boundary, not verified here.
//!
//! ## Error Reason Abstraction
//!
//! The original code constructs `Error::new(ErrorCode::InvalidArgument, "invalid sleep time")`
//! on overflow. This model abstracts away the reason string, retaining only the numeric
//! error code (22). The reason string is diagnostic and does not affect control flow or
//! semantic correctness. Callers match on the error code, not the string.
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

/// Model of ProcessManager::sleep result for verification.
///
/// # Description
///
/// Represents the four possible outcomes from ProcessManager::sleep,
/// matching the original SleepError/InterruptReason enums:
/// - Ok(()) → Ok
/// - Err(Interrupted(TimedOut)) → TimedOut
/// - Err(Interrupted(Killed)) → Killed
/// - Err(Generic(error)) → GenericError
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
    /// Spec function: converts to the abstract PmSleepResultView.
    ///
    /// # Description
    ///
    /// Maps each exec-level variant to the corresponding spec-level PM result.
    /// This is a 1:1 mapping that preserves the distinction between all four
    /// PM outcomes (Ok, TimedOut, Killed, GenericError).
    pub open spec fn spec_pm_view(&self) -> PmSleepResultView {
        match self {
            SleepResultModel::Ok => PmSleepResultView::PmOk,
            SleepResultModel::TimedOut => PmSleepResultView::PmTimedOut,
            SleepResultModel::Killed => PmSleepResultView::PmKilled,
            SleepResultModel::GenericError { error_code } => {
                PmSleepResultView::PmGenericError { error_code: *error_code as int }
            },
        }
    }

    /// Spec function: converts to the final SleepResultView after classification.
    ///
    /// # Description
    ///
    /// Applies the 3-arm match classification: Ok and TimedOut become Success,
    /// Killed becomes KilledError, GenericError passes through.
    pub open spec fn spec_classified_view(&self) -> SleepResultView {
        spec_classify_pm_result(self.spec_pm_view())
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
/// Returns one of the four possible outcomes: Ok, TimedOut, Killed, or GenericError.
#[verifier::external_body]
pub fn process_manager_sleep(alarm: &SystemTimeModel) -> (result: SleepResultModel)
    requires
        alarm.spec_wf(),
    ensures
        // Postcondition documents that the result is always one of the defined variants.
        // This is trivially true for the enum but useful for documentation and refinement.
        matches!(result, SleepResultModel::Ok | SleepResultModel::TimedOut
            | SleepResultModel::Killed | SleepResultModel::GenericError { .. }),
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
        // Since nanoseconds is u32, carry <= 4. On Nanvix's x86-32, seconds
        // comes from usize (32-bit), so this is always satisfied.
        seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat,
    ensures
        result.spec_wf(),
        result.spec_view() == spec_duration_new(seconds as nat, nanoseconds as nat),
        result.nanoseconds < 1_000_000_000u32,
{
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

/// Verified model of the sleep kcall's 3-arm result classification.
///
/// # Description
///
/// Implements the original's match statement (sleep.rs:62-66):
/// ```ignore
/// match ProcessManager::sleep(Some(alarm)) {
///     Ok(()) => Ok(()),
///     Err(SleepError::Interrupted(InterruptReason::TimedOut)) => Ok(()),
///     Err(error) => Err(error),
/// }
/// ```
///
/// Returns a classified SleepResultModel:
/// - Ok/TimedOut → SleepResultModel::Ok (success)
/// - Killed → SleepResultModel::Killed (propagated as error)
/// - GenericError → SleepResultModel::GenericError (propagated unchanged)
///
/// # Parameters
///
/// - `pm_result`: The result from ProcessManager::sleep.
///
/// # Returns
///
/// The classified result matching the original's 3-arm match.
pub fn classify_pm_result(pm_result: SleepResultModel) -> (result: SleepResultModel)
    ensures
        result.spec_classified_view() == spec_classify_pm_result(pm_result.spec_pm_view()),
        // Ok and TimedOut map to success.
        matches!(pm_result, SleepResultModel::Ok | SleepResultModel::TimedOut)
            ==> matches!(result, SleepResultModel::Ok),
        // Killed propagates as error.
        matches!(pm_result, SleepResultModel::Killed)
            ==> matches!(result, SleepResultModel::Killed),
        // GenericError propagates unchanged.
        pm_result.spec_pm_view() matches PmSleepResultView::PmGenericError { error_code }
            ==> result.spec_pm_view() matches PmSleepResultView::PmGenericError { error_code: ec }
                && ec == error_code,
{
    match pm_result {
        SleepResultModel::Ok => SleepResultModel::Ok,
        SleepResultModel::TimedOut => SleepResultModel::Ok,
        SleepResultModel::Killed => SleepResultModel::Killed,
        SleepResultModel::GenericError { error_code } => SleepResultModel::GenericError { error_code },
    }
}

/// Verified exec model of the `sleep(seconds, nanoseconds)` kernel call.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn sleep(seconds, nanoseconds)`
/// control flow. It:
/// 1. Constructs a Duration from (seconds, nanoseconds).
/// 2. Computes the alarm time via checked addition.
/// 3. Returns InvalidArgument error if the addition overflows.
/// 4. Calls ProcessManager::sleep(alarm).
/// 5. Classifies the result via the 3-arm match: Ok/TimedOut → success,
///    Killed/GenericError → propagated as errors.
///
/// # Parameters
///
/// - `now`: The current system time (from clock::now()).
/// - `seconds`: Sleep time in whole seconds. On Nanvix's x86-32 target,
///   this originates from a 32-bit usize, so values > u32::MAX cannot occur
///   at runtime. The model accepts u64 to verify the post-cast domain.
/// - `nanoseconds`: Sleep time in fractional nanoseconds (from usize cast to u32).
///
/// # Returns
///
/// A SleepResultModel indicating the outcome.
pub fn sleep_model(now: &SystemTimeModel, seconds: u64, nanoseconds: u32) -> (ret: (SleepResultModel, Ghost<PmSleepResultView>))
    requires
        now.spec_wf(),
        // On Nanvix's x86-32, seconds originates from a 32-bit usize.
        seconds <= u32::MAX as u64,
        // Duration normalization must not overflow.
        seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat,
    ensures
        // Overflow path: checked_add fails → GenericError(InvalidArgument).
        !spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> ret.0.spec_classified_view() == (SleepResultView::GenericError {
                    error_code: ERROR_CODE_INVALID_ARGUMENT()
                }),
        // Overflow path: result is always GenericError.
        !spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> matches!(ret.0, SleepResultModel::GenericError { .. }),
        // Non-tautological: the ghost captures the original PM result, and the
        // classified view equals spec_sleep_result applied to that PM result.
        // This ties the exec result to the actual PM outcome, not just to the
        // post-classification spec_pm_view (which would be tautological).
        spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> ret.0.spec_classified_view() == spec_sleep_result(
                    now.spec_view(), seconds as nat, nanoseconds as nat, ret.1@),
        // Success path: result is always a classified PM result (Ok, Killed, or GenericError).
        // TimedOut has been folded into Ok by the 3-arm match.
        spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> matches!(ret.0, SleepResultModel::Ok | SleepResultModel::Killed
                    | SleepResultModel::GenericError { .. }),
        // Success path: TimedOut never appears in the output (folded into Ok).
        spec_sleep_success_condition(now.spec_view(), seconds as nat, nanoseconds as nat)
            ==> !matches!(ret.0, SleepResultModel::TimedOut),
{
    // Step 1: Construct the timeout Duration.
    let timeout: DurationModel = duration_new(seconds, nanoseconds);

    // Step 2: Compute the alarm time.
    let alarm_opt: Option<SystemTimeModel> = checked_add_duration(now, &timeout);

    match alarm_opt {
        Some(alarm) => {
            // Step 3: Call ProcessManager::sleep(Some(alarm)).
            let pm_result: SleepResultModel = process_manager_sleep(&alarm);

            // Capture the original PM result as a ghost before classification.
            let ghost orig_pm_view: PmSleepResultView = pm_result.spec_pm_view();

            // Step 4: Classify the result via the 3-arm match.
            // Original: Ok(()) => Ok(()), Interrupted(TimedOut) => Ok(()),
            //           Err(error) => Err(error)
            let classified: SleepResultModel = classify_pm_result(pm_result);
            (classified, Ghost(orig_pm_view))
        },
        None => {
            // Overflow → InvalidArgument.
            proof {
                assert(22i32 as int == ERROR_CODE_INVALID_ARGUMENT());
            }
            // Ghost PM value is arbitrary on the overflow path (ensures are vacuously true).
            (SleepResultModel::GenericError { error_code: 22i32 }, Ghost(PmSleepResultView::PmOk))
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
/// - The PM result classification matches the original's 3-arm match.
///
/// # Parameters
///
/// - `seconds`: Sleep time in whole seconds (original: usize, 32-bit on x86).
/// - `nanoseconds`: Sleep time in fractional nanoseconds (original: usize cast to u32).
///
/// # Returns
///
/// A SleepResultModel indicating the outcome.
pub fn sleep_end_to_end(seconds: u64, nanoseconds: u32) -> (ret: (SleepResultModel, Ghost<PmSleepResultView>))
    requires
        // On Nanvix's x86-32, seconds originates from a 32-bit usize.
        seconds <= u32::MAX as u64,
        seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat,
    ensures
        // TimedOut never appears in the output (core invariant from the 3-arm match).
        !matches!(ret.0, SleepResultModel::TimedOut),
{
    // Step 1: Get the current time (Trust Boundary T1).
    let now: SystemTimeModel = clock_now();

    // Delegate to the verified model.
    sleep_model(&now, seconds, nanoseconds)
}

} // verus!
