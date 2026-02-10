// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Wait Condition Kernel Call Specification.
// Defines View types, spec constants, and spec functions for the wait_cond
// kernel call verification model.
//
// ## Verification Model
//
// The wait_cond kcall converts user-provided (timeout_s, timeout_ns) into an
// optional SystemTime alarm, then executes a multi-step pipeline:
//   1. Parse timeout → Optional alarm (Infinite, Finite, or Invalid).
//   2. ProcessManager::take_mutex_guard(pid, tid, mutex_addr) → release mutex.
//   3. get_cond + cond.wait → combined into a "stored result".
//   4. ProcessManager::put_cond(cond_addr) → release condition variable ref.
//   5. ProcessManager::get_mutex(mutex_addr) → reacquire mutex reference.
//   6. mutex.lock(None) → lock mutex (infinite wait on reacquisition).
//   7. ProcessManager::put_mutex_guard(mutex_addr, guard) → store guard.
//
// Steps 4-7 execute regardless of the stored result from step 3. The stored
// result is returned only if all of steps 4-7 succeed. If any step 4-7 fails,
// its error overrides the stored result (via the `?` operator).

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

/// Number of bits in usize on the target architecture.
pub open spec fn USIZE_BITS() -> nat {
    32
}

/// Spec predicate: whether an error code is a valid ErrorCode discriminant.
pub open spec fn spec_is_valid_error_code(code: int) -> bool {
    code > 0
}

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of the parsed timeout.
#[verifier::ext_equal]
pub enum TimeoutView {
    /// Both timeout_s and timeout_ns are usize::MAX → no timeout (infinite wait).
    Infinite,
    /// A valid SystemTime was constructed.
    Finite { seconds: nat, nanoseconds: nat },
}

/// Abstract view of the outcome of ProcessManager::take_mutex_guard.
#[verifier::ext_equal]
pub enum TakeMutexGuardOutcomeView {
    /// take_mutex_guard succeeded, guard was dropped (mutex released).
    TmgOk,
    /// take_mutex_guard failed with an error code.
    TmgError { error_code: int },
}

/// Abstract view of the outcome of ProcessManager::get_cond.
#[verifier::ext_equal]
pub enum GetCondOutcomeView {
    /// get_cond succeeded.
    GcOk,
    /// get_cond failed with an error code.
    GcError { error_code: int },
}

/// Abstract view of the outcome of cond.wait(alarm).
///
/// # Description
///
/// Models the four possible outcomes from Condvar::wait(alarm), which returns
/// Result<(), SleepError>:
/// - Ok(()) → CwOk
/// - Err(Interrupted(TimedOut)) → CwTimedOut
/// - Err(Interrupted(Killed)) → CwKilled
/// - Err(Generic(error)) → CwGenericError
#[verifier::ext_equal]
pub enum CondWaitOutcomeView {
    /// cond.wait succeeded.
    CwOk,
    /// cond.wait returned Err(Interrupted(TimedOut)).
    CwTimedOut,
    /// cond.wait returned Err(Interrupted(Killed)).
    CwKilled,
    /// cond.wait returned Err(Generic(error)).
    CwGenericError { error_code: int },
}

/// Abstract view of the outcome of ProcessManager::put_cond.
#[verifier::ext_equal]
pub enum PutCondOutcomeView {
    /// put_cond succeeded.
    PcOk,
    /// put_cond failed with an error code.
    PcError { error_code: int },
}

/// Abstract view of the outcome of ProcessManager::get_mutex.
#[verifier::ext_equal]
pub enum GetMutexOutcomeView {
    /// get_mutex succeeded.
    GmOk,
    /// get_mutex failed with an error code.
    GmError { error_code: int },
}

/// Abstract view of the outcome of mutex.lock(None).
///
/// # Description
///
/// On reacquisition, lock is called with None (infinite wait). The possible
/// outcomes are the same SleepError variants, but TimedOut cannot occur
/// with an infinite timeout.
#[verifier::ext_equal]
pub enum LockOutcomeView {
    /// lock succeeded.
    LoOk,
    /// lock returned Err(Interrupted(TimedOut)) — should not occur with None timeout.
    LoTimedOut,
    /// lock returned Err(Interrupted(Killed)).
    LoKilled,
    /// lock returned Err(Generic(error)).
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

/// Abstract view of the wait_cond kcall's final result.
///
/// # Description
///
/// The wait_cond function returns Result<(), SleepError>. This view models
/// all possible outcomes from the multi-step pipeline.
#[verifier::ext_equal]
pub enum WaitCondResultView {
    /// cond.wait succeeded and mutex was reacquired.
    Success,
    /// Timeout parsing failed.
    InvalidTimeoutError { error_code: int },
    /// ProcessManager::take_mutex_guard failed.
    TakeMutexGuardError { error_code: int },
    /// ProcessManager::get_cond failed (returned only if all continuation steps succeed).
    GetCondError { error_code: int },
    /// cond.wait returned Interrupted(TimedOut).
    CondWaitTimedOut,
    /// cond.wait returned Interrupted(Killed).
    CondWaitKilled,
    /// cond.wait returned Generic(error).
    CondWaitGenericError { error_code: int },
    /// ProcessManager::put_cond failed.
    PutCondError { error_code: int },
    /// ProcessManager::get_mutex failed (during mutex reacquisition).
    GetMutexError { error_code: int },
    /// mutex.lock returned Interrupted(TimedOut). Dead variant: unreachable with
    /// None timeout during reacquisition. Retained for exhaustive spec matching.
    LockTimedOut,
    /// mutex.lock returned Interrupted(Killed).
    LockKilled,
    /// mutex.lock returned Generic(error).
    LockGenericError { error_code: int },
    /// ProcessManager::put_mutex_guard failed.
    PutGuardError { error_code: int },
}

/// Ghost state bundle capturing all step outcomes for exec-spec linkage.
///
/// # Description
///
/// Returned as a ghost value from `wait_cond_model` to prove the exec result
/// matches `spec_wait_cond_result` applied to the actual step outcomes.
/// When a step is not reached (due to prior error), its field is set to an
/// arbitrary don't-care value — the spec ignores it due to short-circuit.
#[verifier::ext_equal]
pub struct WaitCondGhostState {
    /// Outcome of ProcessManager::take_mutex_guard.
    pub tmg: TakeMutexGuardOutcomeView,
    /// Outcome of ProcessManager::get_cond.
    pub gc: GetCondOutcomeView,
    /// Outcome of cond.wait (don't-care when get_cond fails).
    pub cw: CondWaitOutcomeView,
    /// Outcome of ProcessManager::put_cond.
    pub pc: PutCondOutcomeView,
    /// Outcome of ProcessManager::get_mutex.
    pub gm: GetMutexOutcomeView,
    /// Outcome of mutex.lock.
    pub lo: LockOutcomeView,
    /// Outcome of ProcessManager::put_mutex_guard.
    pub pg: PutGuardOutcomeView,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

/// Spec function: whether timeout parameters indicate an infinite wait.
pub open spec fn spec_is_infinite_timeout(timeout_s: nat, timeout_ns: nat) -> bool {
    timeout_s == USIZE_MAX_X86_32() && timeout_ns == USIZE_MAX_X86_32()
}

/// Spec function: whether finite timeout nanoseconds are valid.
pub open spec fn spec_timeout_ns_valid(timeout_ns: nat) -> bool {
    timeout_ns < NANOS_PER_SEC()
}

/// Spec function: parse the timeout from raw parameters.
///
/// # Description
///
/// Models the timeout parsing logic from wait_cond:
/// - Both MAX → None (Infinite timeout).
/// - Valid nanoseconds → Some(Finite { seconds, nanoseconds }).
/// - Invalid nanoseconds → None (error path).
pub open spec fn spec_parse_timeout(timeout_s: nat, timeout_ns: nat) -> Option<TimeoutView> {
    if spec_is_infinite_timeout(timeout_s, timeout_ns) {
        Some(TimeoutView::Infinite)
    } else if spec_timeout_ns_valid(timeout_ns) {
        Some(TimeoutView::Finite { seconds: timeout_s, nanoseconds: timeout_ns })
    } else {
        None
    }
}

/// Spec function: whether the timeout was successfully parsed.
pub open spec fn spec_timeout_parsed_ok(timeout_s: nat, timeout_ns: nat) -> bool {
    spec_parse_timeout(timeout_s, timeout_ns).is_some()
}

/// Spec function: whether a parsed timeout is finite.
pub open spec fn spec_is_finite_timeout(timeout_s: nat, timeout_ns: nat) -> bool {
    spec_parse_timeout(timeout_s, timeout_ns) matches Some(TimeoutView::Finite { .. })
}

/// Spec function: computes the "stored result" from get_cond + cond.wait.
///
/// # Description
///
/// In the original code (lines 109-122):
/// ```ignore
/// let result = {
///     match ProcessManager::get_cond(cond_addr) {
///         Ok(cond) => cond.wait(alarm),
///         Err(error) => Err(SleepError::Generic(error)),
///     }
/// };
/// ```
/// The stored result combines get_cond and cond.wait:
/// - If get_cond fails, the error is stored (cond.wait is never called).
/// - If get_cond succeeds, cond.wait's result is stored.
///
/// The continuation pipeline (put_cond, get_mutex, lock, put_guard) runs
/// regardless, and the stored result is returned only if all continue OK.
pub open spec fn spec_stored_result(
    get_cond_outcome: GetCondOutcomeView,
    cond_wait_outcome: CondWaitOutcomeView,
) -> WaitCondResultView {
    match get_cond_outcome {
        GetCondOutcomeView::GcError { error_code } => {
            WaitCondResultView::GetCondError { error_code }
        },
        GetCondOutcomeView::GcOk => {
            match cond_wait_outcome {
                CondWaitOutcomeView::CwOk => WaitCondResultView::Success,
                CondWaitOutcomeView::CwTimedOut => WaitCondResultView::CondWaitTimedOut,
                CondWaitOutcomeView::CwKilled => WaitCondResultView::CondWaitKilled,
                CondWaitOutcomeView::CwGenericError { error_code } => {
                    WaitCondResultView::CondWaitGenericError { error_code }
                },
            }
        },
    }
}

/// Spec function: models the complete wait_cond pipeline.
///
/// # Description
///
/// The wait_cond function executes the following pipeline:
/// 1. Parse timeout → InvalidTimeoutError on failure.
/// 2. take_mutex_guard → TakeMutexGuardError on failure (short-circuit).
/// 3. get_cond + cond.wait → combined into a "stored result" (NO short-circuit).
/// 4. put_cond → PutCondError on failure (overrides stored result).
/// 5. get_mutex → GetMutexError on failure (overrides stored result).
/// 6. lock → Lock error on failure (overrides stored result).
/// 7. put_guard → PutGuardError on failure (overrides stored result).
/// 8. Return stored result from step 3.
///
/// **Critical semantic detail**: In the original code, the `result` variable
/// (line 109) stores the get_cond/cond.wait outcome, and the continuation
/// pipeline (put_cond through put_guard, lines 123-128) executes
/// UNCONDITIONALLY via `?` operators. Continuation errors override the
/// stored result. The stored result is returned only at line 130 after all
/// continuation steps succeed.
pub open spec fn spec_wait_cond_result(
    timeout_s: nat,
    timeout_ns: nat,
    take_guard_outcome: TakeMutexGuardOutcomeView,
    get_cond_outcome: GetCondOutcomeView,
    cond_wait_outcome: CondWaitOutcomeView,
    put_cond_outcome: PutCondOutcomeView,
    get_mutex_outcome: GetMutexOutcomeView,
    lock_outcome: LockOutcomeView,
    put_guard_outcome: PutGuardOutcomeView,
) -> WaitCondResultView {
    match spec_parse_timeout(timeout_s, timeout_ns) {
        None => WaitCondResultView::InvalidTimeoutError {
            error_code: ERROR_CODE_INVALID_ARGUMENT(),
        },
        Some(_timeout) => {
            match take_guard_outcome {
                TakeMutexGuardOutcomeView::TmgError { error_code } => {
                    WaitCondResultView::TakeMutexGuardError { error_code }
                },
                TakeMutexGuardOutcomeView::TmgOk => {
                    // Compute stored result (get_cond + cond.wait).
                    let stored: WaitCondResultView =
                        spec_stored_result(get_cond_outcome, cond_wait_outcome);
                    // Continuation pipeline runs regardless of stored result.
                    match put_cond_outcome {
                        PutCondOutcomeView::PcError { error_code } => {
                            WaitCondResultView::PutCondError { error_code }
                        },
                        PutCondOutcomeView::PcOk => {
                            match get_mutex_outcome {
                                GetMutexOutcomeView::GmError { error_code } => {
                                    WaitCondResultView::GetMutexError { error_code }
                                },
                                GetMutexOutcomeView::GmOk => {
                                    match lock_outcome {
                                        LockOutcomeView::LoTimedOut => {
                                            WaitCondResultView::LockTimedOut
                                        },
                                        LockOutcomeView::LoKilled => {
                                            WaitCondResultView::LockKilled
                                        },
                                        LockOutcomeView::LoGenericError { error_code } => {
                                            WaitCondResultView::LockGenericError { error_code }
                                        },
                                        LockOutcomeView::LoOk => {
                                            match put_guard_outcome {
                                                PutGuardOutcomeView::PgError { error_code } => {
                                                    WaitCondResultView::PutGuardError { error_code }
                                                },
                                                PutGuardOutcomeView::PgOk => {
                                                    // All continuation succeeded.
                                                    // Return stored result.
                                                    stored
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
        },
    }
}

/// Spec function: whether the result is success.
pub open spec fn spec_is_success(result: WaitCondResultView) -> bool {
    matches!(result, WaitCondResultView::Success)
}

/// Spec function: whether the result is any kind of error.
pub open spec fn spec_is_error(result: WaitCondResultView) -> bool {
    !spec_is_success(result)
}

/// Spec function: whether the result is a timeout parsing error.
pub open spec fn spec_is_timeout_error(result: WaitCondResultView) -> bool {
    matches!(result, WaitCondResultView::InvalidTimeoutError { .. })
}

/// Spec function: whether the result is a take_mutex_guard error.
pub open spec fn spec_is_take_guard_error(result: WaitCondResultView) -> bool {
    matches!(result, WaitCondResultView::TakeMutexGuardError { .. })
}

/// Spec function: whether the result is a get_cond error.
pub open spec fn spec_is_get_cond_error(result: WaitCondResultView) -> bool {
    matches!(result, WaitCondResultView::GetCondError { .. })
}

/// Spec function: whether the result is a cond_wait error.
pub open spec fn spec_is_cond_wait_error(result: WaitCondResultView) -> bool {
    matches!(result, WaitCondResultView::CondWaitTimedOut
        | WaitCondResultView::CondWaitKilled
        | WaitCondResultView::CondWaitGenericError { .. })
}

/// Spec function: whether the result is a put_cond error.
pub open spec fn spec_is_put_cond_error(result: WaitCondResultView) -> bool {
    matches!(result, WaitCondResultView::PutCondError { .. })
}

/// Spec function: whether the result is a get_mutex error.
pub open spec fn spec_is_get_mutex_error(result: WaitCondResultView) -> bool {
    matches!(result, WaitCondResultView::GetMutexError { .. })
}

/// Spec function: whether the result is a lock error.
pub open spec fn spec_is_lock_error(result: WaitCondResultView) -> bool {
    matches!(result, WaitCondResultView::LockTimedOut
        | WaitCondResultView::LockKilled
        | WaitCondResultView::LockGenericError { .. })
}

/// Spec function: whether the result is a put_guard error.
pub open spec fn spec_is_put_guard_error(result: WaitCondResultView) -> bool {
    matches!(result, WaitCondResultView::PutGuardError { .. })
}

/// Spec function: whether the result is a "stored result" return.
///
/// # Description
///
/// A stored-result return occurs when all continuation pipeline steps
/// (put_cond, get_mutex, lock, put_guard) succeeded, so the stored result
/// from get_cond+cond.wait is returned. This includes Success, GetCondError,
/// and all CondWait error variants. In all these cases, the mutex was released
/// before the wait and reacquired afterward.
pub open spec fn spec_is_stored_result_return(result: WaitCondResultView) -> bool {
    matches!(result,
        WaitCondResultView::Success
        | WaitCondResultView::GetCondError { .. }
        | WaitCondResultView::CondWaitTimedOut
        | WaitCondResultView::CondWaitKilled
        | WaitCondResultView::CondWaitGenericError { .. })
}

/// Spec function: whether the result indicates a continuation pipeline error.
///
/// # Description
///
/// Continuation errors override the stored result and indicate that the
/// mutex may NOT have been reacquired.
pub open spec fn spec_is_continuation_error(result: WaitCondResultView) -> bool {
    spec_is_put_cond_error(result)
    || spec_is_get_mutex_error(result)
    || spec_is_lock_error(result)
    || spec_is_put_guard_error(result)
}

//==================================================================================================
// Caller Safety Contract Spec Predicates
//==================================================================================================

/// Spec predicate: the calling process is not the kernel process.
pub uninterp spec fn spec_caller_is_not_kernel_process(pid: nat) -> bool;

/// Spec predicate: the calling thread holds no resources.
pub uninterp spec fn spec_caller_holds_no_resources(tid: nat) -> bool;

/// Spec predicate: the caller does not hold a ProcessManager reference.
pub uninterp spec fn spec_caller_no_pm_reference() -> bool;

/// Spec predicate: all three caller safety requirements are satisfied.
pub open spec fn spec_wait_cond_safety_preconditions(pid: nat, tid: nat) -> bool {
    spec_caller_is_not_kernel_process(pid)
    && spec_caller_holds_no_resources(tid)
    && spec_caller_no_pm_reference()
}

/// Spec predicate: the supplied pid/tid identify the currently-running thread.
///
/// # Description
///
/// The PM operates on the currently-running thread internally. This predicate
/// ensures the supplied identifiers match the thread the PM actually operates
/// on, so that ownership postconditions are semantically correct.
pub uninterp spec fn spec_is_currently_running(pid: nat, tid: nat) -> bool;

/// Spec predicate: the mutex was released before the condition wait.
///
/// # Description
///
/// In the wait_cond protocol, the mutex must be released (step 2) before
/// waiting on the condition variable (step 3). This is the standard
/// condition variable usage pattern to avoid deadlocks.
pub uninterp spec fn spec_mutex_released(mutex_addr: nat) -> bool;

/// Spec predicate: the mutex was reacquired after the condition wait.
pub uninterp spec fn spec_mutex_reacquired(mutex_addr: nat) -> bool;

/// Spec predicate: the condition variable reference count was decremented.
pub uninterp spec fn spec_cond_ref_released(cond_addr: nat) -> bool;

} // verus!
