// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Lock Mutex Kernel Call Specification.
// Defines View types, spec constants, and spec functions for the lock_mutex
// kernel call verification model.
//
// ## Verification Model
//
// The lock_mutex kcall converts user-provided (timeout_s, timeout_ns) into an
// optional SystemTime timeout, then executes a three-step pipeline:
//   1. ProcessManager::get_mutex(mutex_addr) → Mutex
//   2. Mutex::lock(timeout) → MutexGuard
//   3. ProcessManager::put_mutex_guard(mutex_addr, guard) → ()
//
// This spec models:
// - Timeout parsing as a pure function with three outcomes (Infinite, Finite, Invalid).
// - The three-step pipeline as a sequential composition with short-circuit on error.
// - Error wrapping: get_mutex and put_guard errors wrapped in SleepError::Generic;
//   lock errors propagated unchanged.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Spec Constants
//==================================================================================================

/// Maximum value for usize on x86-32.
pub open spec fn USIZE_MAX_X86_32() -> nat {
    u32::MAX as nat
}

/// Nanoseconds per second boundary for SystemTime validity.
pub open spec fn NANOS_PER_SEC() -> nat {
    1_000_000_000
}

/// The ErrorCode value for InvalidArgument (repr(i32) = 22).
pub open spec fn ERROR_CODE_INVALID_ARGUMENT() -> int {
    22
}

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of the parsed timeout.
///
/// # Description
///
/// Represents the three possible outcomes of parsing the (timeout_s, timeout_ns)
/// parameters:
/// - `Infinite`: both parameters are usize::MAX, meaning wait indefinitely.
/// - `Finite`: a valid SystemTime was constructed from the parameters.
/// - `Invalid`: SystemTime::new returned None (nanoseconds >= 1_000_000_000).
#[verifier::ext_equal]
pub enum TimeoutView {
    /// Both timeout_s and timeout_ns are usize::MAX → no timeout (infinite wait).
    Infinite,
    /// A valid SystemTime was constructed.
    Finite { seconds: nat, nanoseconds: nat },
}

/// Abstract view of the outcome of ProcessManager::get_mutex.
#[verifier::ext_equal]
pub enum GetMutexOutcomeView {
    /// get_mutex succeeded, returning a Mutex.
    GmOk,
    /// get_mutex failed with an error code.
    GmError { error_code: int },
}

/// Abstract view of the outcome of Mutex::lock.
///
/// # Description
///
/// Models the four possible outcomes from Mutex::lock(timeout), which returns
/// Result<MutexGuard, SleepError>:
/// - Ok(guard) → LoOk
/// - Err(Interrupted(TimedOut)) → LoTimedOut
/// - Err(Interrupted(Killed)) → LoKilled
/// - Err(Generic(error)) → LoGenericError
///
/// Unlike the sleep kcall, lock_mutex does NOT re-classify these results.
/// It propagates the SleepError unchanged via the `?` operator.
#[verifier::ext_equal]
pub enum LockOutcomeView {
    /// Mutex::lock succeeded, returning a MutexGuard.
    LoOk,
    /// Mutex::lock returned Err(Interrupted(TimedOut)).
    LoTimedOut,
    /// Mutex::lock returned Err(Interrupted(Killed)).
    LoKilled,
    /// Mutex::lock returned Err(Generic(error)).
    LoGenericError { error_code: int },
}

/// Abstract view of the outcome of ProcessManager::put_mutex_guard.
#[verifier::ext_equal]
pub enum PutGuardOutcomeView {
    /// put_mutex_guard succeeded.
    PgOk,
    /// put_mutex_guard failed with an error code.
    PgError { error_code: int },
}

/// Abstract view of the lock_mutex kcall's final result.
///
/// # Description
///
/// The lock_mutex function returns Result<(), SleepError>. This view models
/// all possible outcomes:
/// - Success: all three pipeline steps completed without error.
/// - InvalidTimeoutError: timeout parsing failed (nanoseconds >= 1B).
/// - GetMutexError: ProcessManager::get_mutex failed.
/// - LockTimedOut/LockKilled/LockGenericError: Mutex::lock failed with SleepError.
/// - PutGuardError: ProcessManager::put_mutex_guard failed.
#[verifier::ext_equal]
pub enum LockMutexResultView {
    /// All steps succeeded: mutex was locked and guard was stored.
    Success,
    /// Timeout parsing failed: SystemTime::new returned None.
    InvalidTimeoutError { error_code: int },
    /// ProcessManager::get_mutex failed (wrapped in SleepError::Generic).
    GetMutexError { error_code: int },
    /// Mutex::lock returned Interrupted(TimedOut).
    LockTimedOut,
    /// Mutex::lock returned Interrupted(Killed).
    LockKilled,
    /// Mutex::lock returned Generic(error).
    LockGenericError { error_code: int },
    /// ProcessManager::put_mutex_guard failed (wrapped in SleepError::Generic).
    PutGuardError { error_code: int },
}

//==================================================================================================
// Spec Functions
//==================================================================================================

/// Spec function: whether timeout parameters indicate an infinite wait.
///
/// # Description
///
/// The original code checks `timeout_s == usize::MAX && timeout_ns == usize::MAX`.
/// On x86-32, usize::MAX == u32::MAX == 0xFFFF_FFFF.
pub open spec fn spec_is_infinite_timeout(timeout_s: nat, timeout_ns: nat) -> bool {
    timeout_s == USIZE_MAX_X86_32() && timeout_ns == USIZE_MAX_X86_32()
}

/// Spec function: whether finite timeout nanoseconds are valid.
///
/// # Description
///
/// SystemTime::new(seconds, nanoseconds) returns None when nanoseconds >= 1_000_000_000.
/// This spec checks the validity of the nanoseconds component.
pub open spec fn spec_timeout_ns_valid(timeout_ns: nat) -> bool {
    timeout_ns < NANOS_PER_SEC()
}

/// Spec function: parse the timeout from raw parameters.
///
/// # Description
///
/// Models the timeout parsing logic from the original code:
/// - Both MAX → None (Infinite timeout).
/// - Valid nanoseconds → Some(Finite { seconds, nanoseconds }).
/// - Invalid nanoseconds → None (error path).
///
/// Returns Some(TimeoutView) on success, None on invalid timeout.
pub open spec fn spec_parse_timeout(timeout_s: nat, timeout_ns: nat) -> Option<TimeoutView> {
    if spec_is_infinite_timeout(timeout_s, timeout_ns) {
        Some(TimeoutView::Infinite)
    } else if spec_timeout_ns_valid(timeout_ns) {
        Some(TimeoutView::Finite { seconds: timeout_s, nanoseconds: timeout_ns })
    } else {
        None
    }
}

/// Spec function: models the complete lock_mutex pipeline.
///
/// # Description
///
/// The lock_mutex function executes a sequential pipeline:
/// 1. Parse timeout → InvalidTimeoutError on failure.
/// 2. get_mutex → GetMutexError on failure.
/// 3. lock(timeout) → LockError variant on failure.
/// 4. put_mutex_guard → PutGuardError on failure.
/// 5. All succeeded → Success.
///
/// Each step only executes if all previous steps succeeded (short-circuit).
pub open spec fn spec_lock_mutex_result(
    timeout_s: nat,
    timeout_ns: nat,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
) -> LockMutexResultView {
    match spec_parse_timeout(timeout_s, timeout_ns) {
        None => LockMutexResultView::InvalidTimeoutError {
            error_code: ERROR_CODE_INVALID_ARGUMENT(),
        },
        Some(_timeout) => {
            match get_mutex_outcome {
                GetMutexOutcomeView::GmError { error_code } => {
                    LockMutexResultView::GetMutexError { error_code }
                },
                GetMutexOutcomeView::GmOk => {
                    match lock_outcome {
                        LockOutcomeView::LoTimedOut => LockMutexResultView::LockTimedOut,
                        LockOutcomeView::LoKilled => LockMutexResultView::LockKilled,
                        LockOutcomeView::LoGenericError { error_code } => {
                            LockMutexResultView::LockGenericError { error_code }
                        },
                        LockOutcomeView::LoOk => {
                            match put_guard_outcome {
                                PutGuardOutcomeView::PgError { error_code } => {
                                    LockMutexResultView::PutGuardError { error_code }
                                },
                                PutGuardOutcomeView::PgOk => LockMutexResultView::Success,
                            }
                        },
                    }
                },
            }
        },
    }
}

/// Spec function: whether the result is success.
pub open spec fn spec_is_success(result: LockMutexResultView) -> bool {
    matches!(result, LockMutexResultView::Success)
}

/// Spec function: whether the result is any kind of error.
pub open spec fn spec_is_error(result: LockMutexResultView) -> bool {
    !spec_is_success(result)
}

/// Spec function: whether the result is a timeout parsing error.
pub open spec fn spec_is_timeout_error(result: LockMutexResultView) -> bool {
    matches!(result, LockMutexResultView::InvalidTimeoutError { .. })
}

/// Spec function: whether the result is a get_mutex error.
pub open spec fn spec_is_get_mutex_error(result: LockMutexResultView) -> bool {
    matches!(result, LockMutexResultView::GetMutexError { .. })
}

/// Spec function: whether the result is a lock error (any variant).
pub open spec fn spec_is_lock_error(result: LockMutexResultView) -> bool {
    matches!(result, LockMutexResultView::LockTimedOut
        | LockMutexResultView::LockKilled
        | LockMutexResultView::LockGenericError { .. })
}

/// Spec function: whether the result is a put_guard error.
pub open spec fn spec_is_put_guard_error(result: LockMutexResultView) -> bool {
    matches!(result, LockMutexResultView::PutGuardError { .. })
}

/// Spec function: whether the timeout was successfully parsed.
pub open spec fn spec_timeout_parsed_ok(timeout_s: nat, timeout_ns: nat) -> bool {
    spec_parse_timeout(timeout_s, timeout_ns).is_some()
}

/// Spec function: well-formedness of a finite timeout.
pub open spec fn spec_finite_timeout_wf(seconds: nat, nanoseconds: nat) -> bool {
    nanoseconds < NANOS_PER_SEC()
}

} // verus!
