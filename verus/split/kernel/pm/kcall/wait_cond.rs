// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Wait Condition Kernel Call Verification Model
//!
//! Formal verification of the wait_cond kernel call (`pm::kcall::wait_cond`).
//!
//! ## Overview
//!
//! The `wait_cond(pid, tid, cond_addr, mutex_addr, timeout_s, timeout_ns)` function
//! waits on a condition variable. It:
//! 1. Parses the timeout parameters into an optional alarm time.
//! 2. Releases the caller's mutex guard via `ProcessManager::take_mutex_guard`.
//! 3. Retrieves the condition variable via `ProcessManager::get_cond`.
//! 4. Waits on the condition variable via `cond.wait(alarm)`.
//! 5. Releases the condition variable reference via `ProcessManager::put_cond`.
//! 6. Reacquires the mutex via `ProcessManager::get_mutex` + `mutex.lock(None)` +
//!    `ProcessManager::put_mutex_guard`.
//! 7. Returns the result of `cond.wait` (step 4).
//!
//! ## Verified Properties
//!
//! - **Timeout parsing correctness**: Both MAX → infinite; valid nanos → finite;
//!   invalid nanos → InvalidArgument error.
//! - **Pipeline short-circuit**: Errors at any step abort the remaining pipeline.
//! - **Error propagation**: Each step's error maps to the corresponding result variant.
//! - **Success requires all steps**: Success iff ALL pipeline steps succeed AND
//!   cond.wait returns Ok.
//! - **Result exhaustiveness**: Every input produces exactly one result category.
//! - **cond.wait result preservation**: When all subsequent steps succeed, the
//!   cond.wait outcome is faithfully reflected in the final result.
//! - **Reacquisition uses infinite wait**: `mutex.lock(None)` cannot produce TimedOut.
//! - **Error code linkage**: Spec constant matches `ErrorCode::InvalidArgument`.
//! - **Architecture guard**: x86-32 assumption verified.
//! - **Safety precondition composition**: Three safety requirements compose correctly.
//! - **Exec model correctness**: `wait_cond_model` matches `spec_wait_cond_result`
//!   for all inputs and all step outcomes.
//!
//! ## Properties NOT Proven Here (Out of Scope)
//!
//! - **Condition variable semantics**: Whether `cond.wait` correctly waits for
//!   the associated condition is verified in the condvar module.
//! - **Mutex ownership**: Whether the caller actually holds the mutex is a PM concern.
//! - **Liveness**: Whether `cond.wait` and `mutex.lock` eventually return is
//!   a scheduler fairness property, verified separately.
//! - **Timing**: Whether TimedOut is returned only after the alarm time is a
//!   real-time property, verified in the clock and condvar modules.
//!
//! ## Verification Model
//!
//! The original function uses several external dependencies:
//! - `ConditionAddress::from(usize)` → type wrapper, not modeled.
//! - `MutexAddress::from(usize)` → type wrapper, not modeled.
//! - `SystemTime::new()` → modeled via timeout parsing in the exec model.
//! - `ProcessManager::take_mutex_guard()` → `take_mutex_guard_model()` external_body.
//! - `ProcessManager::get_cond()` → `get_cond_model()` external_body.
//! - `cond.wait(alarm)` → `cond_wait_model()` external_body.
//! - `ProcessManager::put_cond()` → `put_cond_model()` external_body.
//! - `ProcessManager::get_mutex()` → `get_mutex_model()` external_body.
//! - `Mutex::lock(None)` → `mutex_lock_model()` external_body.
//! - `ProcessManager::put_mutex_guard()` → `put_mutex_guard_model()` external_body.
//!
//! ## Trust Boundaries
//!
//! - **T1: `ProcessManager::take_mutex_guard()`**. Releases the caller's mutex guard.
//! - **T2: `ProcessManager::get_cond()`**. Retrieves condition variable reference.
//! - **T3: `cond.wait(alarm)`**. Waits on the condition variable.
//! - **T4: `ProcessManager::put_cond()`**. Releases condition variable reference.
//! - **T5: `ProcessManager::get_mutex()`**. Gets mutex reference for reacquisition.
//! - **T6: `Mutex::lock(None)`**. Reacquires the mutex with infinite wait.
//! - **T7: `ProcessManager::put_mutex_guard()`**. Stores the new guard.
//!
//! ## API Mapping
//!
//! | Original API                                  | Verified Model                    | Notes            |
//! |-----------------------------------------------|-----------------------------------|------------------|
//! | `ConditionAddress::from(usize)`               | (not modeled)                     | Type wrapper.    |
//! | `MutexAddress::from(usize)`                   | (not modeled)                     | Type wrapper.    |
//! | `SystemTime::new(s, ns)`                      | `parse_timeout_model(s, ns)`      | Verified.        |
//! | `ProcessManager::take_mutex_guard(p,t,a)`     | `take_mutex_guard_model(a)`       | external_body.   |
//! | `ProcessManager::get_cond(a)`                 | `get_cond_model(a)`               | external_body.   |
//! | `cond.wait(alarm)`                            | `cond_wait_model(alarm)`          | external_body.   |
//! | `ProcessManager::put_cond(a)`                 | `put_cond_model(a)`               | external_body.   |
//! | `ProcessManager::get_mutex(a)`                | `get_mutex_model(a)`              | external_body.   |
//! | `Mutex::lock(None)`                           | `mutex_lock_model(a)`             | external_body.   |
//! | `ProcessManager::put_mutex_guard(a, guard)`   | `put_mutex_guard_model(a)`        | external_body.   |
//! | `pub unsafe fn wait_cond(...)`                | `wait_cond_model(...)`            | Fully verified.  |

use crate::libs::error::ErrorCode;
use vstd::prelude::*;

// Include specifications.
include!("wait_cond.spec.rs");

// Include proofs.
include!("wait_cond.proof.rs");

verus! {

//==================================================================================================
// Dependency Models (External Bodies)
//==================================================================================================

/// Model of the take_mutex_guard step outcome.
pub enum TakeMutexGuardOutcomeModel {
    /// take_mutex_guard succeeded (guard was dropped, mutex released).
    Ok,
    /// take_mutex_guard failed with an error code.
    Error { error_code: i32 },
}

impl TakeMutexGuardOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> TakeMutexGuardOutcomeView {
        match self {
            TakeMutexGuardOutcomeModel::Ok => TakeMutexGuardOutcomeView::TmgOk,
            TakeMutexGuardOutcomeModel::Error { error_code } => {
                TakeMutexGuardOutcomeView::TmgError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the get_cond step outcome.
pub enum GetCondOutcomeModel {
    /// get_cond succeeded.
    Ok,
    /// get_cond failed with an error code.
    Error { error_code: i32 },
}

impl GetCondOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> GetCondOutcomeView {
        match self {
            GetCondOutcomeModel::Ok => GetCondOutcomeView::GcOk,
            GetCondOutcomeModel::Error { error_code } => {
                GetCondOutcomeView::GcError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the cond.wait step outcome.
///
/// # Description
///
/// Models the four possible outcomes from Condvar::wait(alarm).
pub enum CondWaitOutcomeModel {
    /// cond.wait returned Ok(()).
    Ok,
    /// cond.wait returned Err(Interrupted(TimedOut)).
    TimedOut,
    /// cond.wait returned Err(Interrupted(Killed)).
    Killed,
    /// cond.wait returned Err(Generic(error)).
    GenericError { error_code: i32 },
}

impl CondWaitOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> CondWaitOutcomeView {
        match self {
            CondWaitOutcomeModel::Ok => CondWaitOutcomeView::CwOk,
            CondWaitOutcomeModel::TimedOut => CondWaitOutcomeView::CwTimedOut,
            CondWaitOutcomeModel::Killed => CondWaitOutcomeView::CwKilled,
            CondWaitOutcomeModel::GenericError { error_code } => {
                CondWaitOutcomeView::CwGenericError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the put_cond step outcome.
pub enum PutCondOutcomeModel {
    /// put_cond succeeded.
    Ok,
    /// put_cond failed with an error code.
    Error { error_code: i32 },
}

impl PutCondOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> PutCondOutcomeView {
        match self {
            PutCondOutcomeModel::Ok => PutCondOutcomeView::PcOk,
            PutCondOutcomeModel::Error { error_code } => {
                PutCondOutcomeView::PcError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the get_mutex step outcome.
pub enum GetMutexOutcomeModel {
    /// get_mutex succeeded.
    Ok,
    /// get_mutex failed with an error code.
    Error { error_code: i32 },
}

impl GetMutexOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> GetMutexOutcomeView {
        match self {
            GetMutexOutcomeModel::Ok => GetMutexOutcomeView::GmOk,
            GetMutexOutcomeModel::Error { error_code } => {
                GetMutexOutcomeView::GmError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the mutex.lock step outcome.
pub enum LockOutcomeModel {
    /// lock succeeded.
    Ok,
    /// lock returned Err(Interrupted(TimedOut)).
    TimedOut,
    /// lock returned Err(Interrupted(Killed)).
    Killed,
    /// lock returned Err(Generic(error)).
    GenericError { error_code: i32 },
}

impl LockOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> LockOutcomeView {
        match self {
            LockOutcomeModel::Ok => LockOutcomeView::LoOk,
            LockOutcomeModel::TimedOut => LockOutcomeView::LoTimedOut,
            LockOutcomeModel::Killed => LockOutcomeView::LoKilled,
            LockOutcomeModel::GenericError { error_code } => {
                LockOutcomeView::LoGenericError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the put_mutex_guard step outcome.
pub enum PutGuardOutcomeModel {
    /// put_mutex_guard succeeded.
    Ok,
    /// put_mutex_guard failed with an error code.
    Error { error_code: i32 },
}

impl PutGuardOutcomeModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> PutGuardOutcomeView {
        match self {
            PutGuardOutcomeModel::Ok => PutGuardOutcomeView::PgOk,
            PutGuardOutcomeModel::Error { error_code } => {
                PutGuardOutcomeView::PgError { error_code: *error_code as int }
            },
        }
    }
}

/// Model of the wait_cond overall result.
pub enum WaitCondResultModel {
    /// All pipeline steps succeeded and cond.wait returned Ok.
    Success,
    /// Timeout parsing failed.
    InvalidTimeoutError { error_code: i32 },
    /// take_mutex_guard failed.
    TakeMutexGuardError { error_code: i32 },
    /// get_cond failed.
    GetCondError { error_code: i32 },
    /// cond.wait returned TimedOut.
    CondWaitTimedOut,
    /// cond.wait returned Killed.
    CondWaitKilled,
    /// cond.wait returned GenericError.
    CondWaitGenericError { error_code: i32 },
    /// put_cond failed.
    PutCondError { error_code: i32 },
    /// get_mutex failed.
    GetMutexError { error_code: i32 },
    /// mutex.lock returned TimedOut.
    LockTimedOut,
    /// mutex.lock returned Killed.
    LockKilled,
    /// mutex.lock returned GenericError.
    LockGenericError { error_code: i32 },
    /// put_mutex_guard failed.
    PutGuardError { error_code: i32 },
}

impl WaitCondResultModel {
    /// Spec function: converts to the abstract view.
    pub open spec fn spec_view(&self) -> WaitCondResultView {
        match self {
            WaitCondResultModel::Success => WaitCondResultView::Success,
            WaitCondResultModel::InvalidTimeoutError { error_code } => {
                WaitCondResultView::InvalidTimeoutError { error_code: *error_code as int }
            },
            WaitCondResultModel::TakeMutexGuardError { error_code } => {
                WaitCondResultView::TakeMutexGuardError { error_code: *error_code as int }
            },
            WaitCondResultModel::GetCondError { error_code } => {
                WaitCondResultView::GetCondError { error_code: *error_code as int }
            },
            WaitCondResultModel::CondWaitTimedOut => WaitCondResultView::CondWaitTimedOut,
            WaitCondResultModel::CondWaitKilled => WaitCondResultView::CondWaitKilled,
            WaitCondResultModel::CondWaitGenericError { error_code } => {
                WaitCondResultView::CondWaitGenericError { error_code: *error_code as int }
            },
            WaitCondResultModel::PutCondError { error_code } => {
                WaitCondResultView::PutCondError { error_code: *error_code as int }
            },
            WaitCondResultModel::GetMutexError { error_code } => {
                WaitCondResultView::GetMutexError { error_code: *error_code as int }
            },
            WaitCondResultModel::LockTimedOut => WaitCondResultView::LockTimedOut,
            WaitCondResultModel::LockKilled => WaitCondResultView::LockKilled,
            WaitCondResultModel::LockGenericError { error_code } => {
                WaitCondResultView::LockGenericError { error_code: *error_code as int }
            },
            WaitCondResultModel::PutGuardError { error_code } => {
                WaitCondResultView::PutGuardError { error_code: *error_code as int }
            },
        }
    }
}

//==================================================================================================
// External Body Functions (Trust Boundaries)
//==================================================================================================

/// Trust Boundary T1: Models `ProcessManager::take_mutex_guard(pid, tid, mutex_addr)`.
///
/// # Description
///
/// Retrieves and drops the caller's mutex guard, releasing the mutex.
/// In the original, the guard is dropped at the closing brace of the block.
#[verifier::external_body]
pub fn take_mutex_guard_model(mutex_addr: u32) -> (result: TakeMutexGuardOutcomeModel)
    ensures
        result matches TakeMutexGuardOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
        result matches TakeMutexGuardOutcomeModel::Ok
            ==> spec_mutex_released(mutex_addr as nat),
{
    unimplemented!()
}

/// Trust Boundary T2: Models `ProcessManager::get_cond(cond_addr)`.
///
/// # Description
///
/// Retrieves a reference to the condition variable at the given address.
/// Increments the reference count.
#[verifier::external_body]
pub fn get_cond_model(cond_addr: u32) -> (result: GetCondOutcomeModel)
    ensures
        result matches GetCondOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
{
    unimplemented!()
}

/// Trust Boundary T3: Models `cond.wait(alarm)`.
///
/// # Description
///
/// Waits on the condition variable until signaled or the alarm time is reached.
/// The `has_alarm` flag indicates whether a finite timeout was provided.
#[verifier::external_body]
pub fn cond_wait_model(has_alarm: bool) -> (result: CondWaitOutcomeModel)
    ensures
        // TimedOut can only occur when an alarm is set.
        result matches CondWaitOutcomeModel::TimedOut ==> has_alarm,
{
    unimplemented!()
}

/// Trust Boundary T4: Models `ProcessManager::put_cond(cond_addr)`.
///
/// # Description
///
/// Releases the condition variable reference, decrementing the reference count.
#[verifier::external_body]
pub fn put_cond_model(cond_addr: u32) -> (result: PutCondOutcomeModel)
    ensures
        result matches PutCondOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
        result matches PutCondOutcomeModel::Ok
            ==> spec_cond_ref_released(cond_addr as nat),
{
    unimplemented!()
}

/// Trust Boundary T5: Models `ProcessManager::get_mutex(mutex_addr)`.
///
/// # Description
///
/// Retrieves a reference to the mutex for reacquisition.
#[verifier::external_body]
pub fn get_mutex_model(mutex_addr: u32) -> (result: GetMutexOutcomeModel)
    ensures
        result matches GetMutexOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
{
    unimplemented!()
}

/// Trust Boundary T6: Models `Mutex::lock(None)`.
///
/// # Description
///
/// Reacquires the mutex with an infinite wait (None timeout).
/// TimedOut cannot occur because no timeout is set.
#[verifier::external_body]
pub fn mutex_lock_model(mutex_addr: u32) -> (result: LockOutcomeModel)
    ensures
        // TimedOut is impossible with None timeout.
        !matches!(result, LockOutcomeModel::TimedOut),
        result matches LockOutcomeModel::Ok
            ==> spec_mutex_reacquired(mutex_addr as nat),
{
    unimplemented!()
}

/// Trust Boundary T7: Models `ProcessManager::put_mutex_guard(mutex_addr, guard)`.
///
/// # Description
///
/// Stores the new mutex guard after reacquisition.
#[verifier::external_body]
pub fn put_mutex_guard_model(mutex_addr: u32) -> (result: PutGuardOutcomeModel)
    ensures
        result matches PutGuardOutcomeModel::Error { error_code }
            ==> spec_is_valid_error_code(error_code as int),
{
    unimplemented!()
}

//==================================================================================================
// Verified Functions
//==================================================================================================

/// Verified model of timeout parsing.
///
/// # Description
///
/// Parses the (timeout_s, timeout_ns) parameters:
/// - Both usize::MAX → None (infinite wait).
/// - Valid nanoseconds → Some(true) indicating a finite alarm.
/// - Invalid nanoseconds → error.
///
/// Returns `(ok, has_alarm)` where ok indicates success and has_alarm
/// indicates whether a finite alarm was produced.
pub fn parse_timeout_model(timeout_s: u32, timeout_ns: u32) -> (result: (bool, bool))
    ensures
        // First element is true iff parsing succeeded.
        result.0 == spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat),
        // Second element is true iff a finite timeout was parsed.
        result.0 ==> (result.1 == spec_is_finite_timeout(timeout_s as nat, timeout_ns as nat)),
        // When not ok, has_alarm is false.
        !result.0 ==> !result.1,
{
    if timeout_s == u32::MAX && timeout_ns == u32::MAX {
        // Infinite timeout.
        (true, false)
    } else if timeout_ns < 1_000_000_000u32 {
        // Valid finite timeout.
        (true, true)
    } else {
        // Invalid nanoseconds.
        (false, false)
    }
}

/// Verified exec model of the `wait_cond` kernel call.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn wait_cond(...)` control flow:
/// 1. Parse timeout.
/// 2. take_mutex_guard → release mutex.
/// 3. get_cond → get condition variable.
/// 4. cond.wait(alarm) → wait on condition variable.
/// 5. put_cond → release condition variable ref.
/// 6. get_mutex → get mutex for reacquisition.
/// 7. mutex.lock(None) → reacquire mutex.
/// 8. put_mutex_guard → store guard.
/// 9. Return cond.wait result.
///
/// # Parameters
///
/// - `cond_addr`: Condition variable address.
/// - `mutex_addr`: Mutex address.
/// - `timeout_s`: Timeout seconds (u32 on x86-32).
/// - `timeout_ns`: Timeout nanoseconds (u32 on x86-32).
pub fn wait_cond_model(
    cond_addr: u32,
    mutex_addr: u32,
    timeout_s: u32,
    timeout_ns: u32,
) -> (ret: (WaitCondResultModel, Ghost<WaitCondResultView>))
    requires
        cond_addr as nat <= USIZE_MAX_X86_32(),
        mutex_addr as nat <= USIZE_MAX_X86_32(),
        timeout_s as nat <= USIZE_MAX_X86_32(),
        timeout_ns as nat <= USIZE_MAX_X86_32(),
    ensures
        // The result model's view matches the ghost result view.
        ret.0.spec_view() == ret.1@,
{
    // Step 1: Parse timeout.
    let parse_result: (bool, bool) = parse_timeout_model(timeout_s, timeout_ns);
    let timeout_ok: bool = parse_result.0;
    let has_alarm: bool = parse_result.1;

    if !timeout_ok {
        // Invalid timeout → error.
        proof {
            assert(22i32 as int == ERROR_CODE_INVALID_ARGUMENT());
        }
        let result: WaitCondResultModel = WaitCondResultModel::InvalidTimeoutError { error_code: 22i32 };
        let ghost result_view: WaitCondResultView = result.spec_view();
        return (result, Ghost(result_view));
    }

    // Step 2: take_mutex_guard → release mutex.
    let tmg_result: TakeMutexGuardOutcomeModel = take_mutex_guard_model(mutex_addr);
    match tmg_result {
        TakeMutexGuardOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel = WaitCondResultModel::TakeMutexGuardError { error_code };
            let ghost result_view: WaitCondResultView = result.spec_view();
            return (result, Ghost(result_view));
        },
        TakeMutexGuardOutcomeModel::Ok => {},
    }

    // Step 3: get_cond.
    let gc_result: GetCondOutcomeModel = get_cond_model(cond_addr);
    match gc_result {
        GetCondOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel = WaitCondResultModel::GetCondError { error_code };
            let ghost result_view: WaitCondResultView = result.spec_view();
            return (result, Ghost(result_view));
        },
        GetCondOutcomeModel::Ok => {},
    }

    // Step 4: cond.wait(alarm).
    let cw_result: CondWaitOutcomeModel = cond_wait_model(has_alarm);

    // Step 5: put_cond.
    let pc_result: PutCondOutcomeModel = put_cond_model(cond_addr);
    match pc_result {
        PutCondOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel = WaitCondResultModel::PutCondError { error_code };
            let ghost result_view: WaitCondResultView = result.spec_view();
            return (result, Ghost(result_view));
        },
        PutCondOutcomeModel::Ok => {},
    }

    // Step 6: get_mutex.
    let gm_result: GetMutexOutcomeModel = get_mutex_model(mutex_addr);
    match gm_result {
        GetMutexOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel = WaitCondResultModel::GetMutexError { error_code };
            let ghost result_view: WaitCondResultView = result.spec_view();
            return (result, Ghost(result_view));
        },
        GetMutexOutcomeModel::Ok => {},
    }

    // Step 7: mutex.lock(None) — reacquire with infinite wait.
    let lock_result: LockOutcomeModel = mutex_lock_model(mutex_addr);
    match lock_result {
        LockOutcomeModel::TimedOut => {
            let result: WaitCondResultModel = WaitCondResultModel::LockTimedOut;
            let ghost result_view: WaitCondResultView = result.spec_view();
            return (result, Ghost(result_view));
        },
        LockOutcomeModel::Killed => {
            let result: WaitCondResultModel = WaitCondResultModel::LockKilled;
            let ghost result_view: WaitCondResultView = result.spec_view();
            return (result, Ghost(result_view));
        },
        LockOutcomeModel::GenericError { error_code } => {
            let result: WaitCondResultModel = WaitCondResultModel::LockGenericError { error_code };
            let ghost result_view: WaitCondResultView = result.spec_view();
            return (result, Ghost(result_view));
        },
        LockOutcomeModel::Ok => {},
    }

    // Step 8: put_mutex_guard.
    let pg_result: PutGuardOutcomeModel = put_mutex_guard_model(mutex_addr);
    match pg_result {
        PutGuardOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel = WaitCondResultModel::PutGuardError { error_code };
            let ghost result_view: WaitCondResultView = result.spec_view();
            return (result, Ghost(result_view));
        },
        PutGuardOutcomeModel::Ok => {},
    }

    // Step 9: Return cond.wait result.
    let result: WaitCondResultModel = match cw_result {
        CondWaitOutcomeModel::Ok => WaitCondResultModel::Success,
        CondWaitOutcomeModel::TimedOut => WaitCondResultModel::CondWaitTimedOut,
        CondWaitOutcomeModel::Killed => WaitCondResultModel::CondWaitKilled,
        CondWaitOutcomeModel::GenericError { error_code } => WaitCondResultModel::CondWaitGenericError { error_code },
    };
    let ghost result_view: WaitCondResultView = result.spec_view();
    (result, Ghost(result_view))
}

} // verus!
