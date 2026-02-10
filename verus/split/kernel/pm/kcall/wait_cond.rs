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
//! 3. Computes a "stored result" from `get_cond` + `cond.wait(alarm)`. If get_cond
//!    fails, the error is stored and cond.wait is NOT called.
//! 4. Runs the continuation pipeline unconditionally: `put_cond`, `get_mutex`,
//!    `mutex.lock(None)`, `put_mutex_guard`. Each step uses `?` so its error
//!    overrides the stored result.
//! 5. Returns the stored result only if all continuation steps succeed.
//!
//! ## Verified Properties
//!
//! - **Timeout parsing correctness**: Both MAX → infinite; valid nanos → finite;
//!   invalid nanos → InvalidArgument error.
//! - **Stored result semantics**: get_cond failure stores the error (cond.wait is
//!   never called); get_cond success stores the cond.wait outcome.
//! - **Continuation pipeline**: put_cond, get_mutex, lock, put_guard run
//!   unconditionally after the stored result; their errors override it.
//! - **Pipeline short-circuit**: Errors at steps 1-2 abort before the stored result
//!   computation. Continuation errors override the stored result.
//! - **Error propagation**: Each step's error maps to the corresponding result variant.
//! - **Success requires all steps**: Success iff ALL pipeline steps succeed AND
//!   get_cond succeeds AND cond.wait returns Ok.
//! - **Result exhaustiveness**: Every input produces exactly one result category.
//! - **cond.wait result preservation**: When all subsequent steps succeed and get_cond
//!   succeeded, the cond.wait outcome is faithfully reflected in the final result.
//! - **Reacquisition uses infinite wait**: `mutex.lock(None)` cannot produce TimedOut.
//! - **get_cond failure ignores cond_wait**: When get_cond fails, the cond_wait_outcome
//!   parameter is irrelevant — changing it does not affect the result.
//! - **Continuation errors override stored result**: When get_cond fails but a
//!   continuation step also fails, the continuation error takes priority.
//! - **Error code linkage**: Spec constant matches `ErrorCode::InvalidArgument`.
//! - **Safety precondition composition**: Three safety requirements compose correctly.
//! - **Mutex protocol on success**: On success, `spec_mutex_released`,
//!   `spec_cond_ref_released`, and `spec_mutex_reacquired` all hold.
//! - **Resource release independence**: `spec_mutex_released` holds whenever
//!   `take_mutex_guard` succeeds (TmgOk), `spec_cond_ref_released` whenever
//!   `put_cond` succeeds (PcOk), and `spec_mutex_reacquired` whenever `lock`
//!   succeeds (LoOk) — independent of later continuation errors.
//! - **Architecture guard**: Explicit `USIZE_BITS() == 32` precondition.
//! - **Exec-spec equivalence**: `wait_cond_model` result matches
//!   `spec_wait_cond_result` applied to the ghost step outcomes.
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
//! ## Trust Boundaries
//!
//! - **T1: `ProcessManager::take_mutex_guard(pid, tid, addr)`**. Releases mutex guard.
//! - **T2: `ProcessManager::get_cond(addr)`**. Gets condition variable reference.
//! - **T3: `cond.wait(alarm)`**. Waits on the condition variable.
//! - **T4: `ProcessManager::put_cond(addr)`**. Releases condition variable reference.
//!   Called unconditionally in the original code regardless of get_cond outcome.
//! - **T5: `ProcessManager::get_mutex(addr)`**. Gets mutex for reacquisition.
//! - **T6: `Mutex::lock(None)`**. Reacquires the mutex with infinite wait.
//! - **T7: `ProcessManager::put_mutex_guard(addr, guard)`**. Stores new guard.
//!
//! ## Trust Assumptions
//!
//! The following uninterpreted predicates are assumed correct by the caller
//! and are not verified within this module:
//!
//! - `spec_caller_is_not_kernel_process(pid)`: The calling process is not
//!   the kernel process.
//! - `spec_caller_holds_no_resources(tid)`: The calling thread holds no
//!   resources that would deadlock.
//! - `spec_caller_no_pm_reference()`: The caller does not hold a
//!   ProcessManager reference.
//! - `spec_is_currently_running(pid, tid)`: The supplied identifiers match
//!   the thread the PM actually operates on.
//!
//! ## API Mapping
//!
//! | Original API                                  | Verified Model                     | Notes            |
//! |-----------------------------------------------|------------------------------------|------------------|
//! | `ConditionAddress::from(usize)`               | (not modeled)                      | Type wrapper.    |
//! | `MutexAddress::from(usize)`                   | (not modeled)                      | Type wrapper.    |
//! | `SystemTime::new(s, ns)`                      | `parse_timeout_model(s, ns)`       | Verified.        |
//! | `ProcessManager::take_mutex_guard(p,t,a)`     | `take_mutex_guard_model(a,p,t)`    | external_body.   |
//! | `ProcessManager::get_cond(a)`                 | `get_cond_model(a)`                | external_body.   |
//! | `cond.wait(alarm)`                            | `cond_wait_model(alarm)`           | external_body.   |
//! | get_cond + cond.wait combined                 | `get_cond_and_wait_model(a,alarm)` | Verified helper. |
//! | `ProcessManager::put_cond(a)`                 | `put_cond_model(a)`                | external_body.   |
//! | `ProcessManager::get_mutex(a)`                | `get_mutex_model(a)`               | external_body.   |
//! | `Mutex::lock(None)`                           | `mutex_lock_model(a)`              | external_body.   |
//! | `ProcessManager::put_mutex_guard(a, guard)`   | `put_mutex_guard_model(a)`         | external_body.   |
//! | `pub unsafe fn wait_cond(...)`                | `wait_cond_model(...)`             | Fully verified.  |

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
    /// get_cond failed (returned only when all continuation steps succeed).
    GetCondError { error_code: i32 },
    /// cond.wait returned TimedOut.
    CondWaitTimedOut,
    /// cond.wait returned Killed.
    CondWaitKilled,
    /// cond.wait returned GenericError.
    CondWaitGenericError { error_code: i32 },
    /// put_cond failed (overrides stored result).
    PutCondError { error_code: i32 },
    /// get_mutex failed (overrides stored result).
    GetMutexError { error_code: i32 },
    /// mutex.lock returned TimedOut (unreachable with None timeout; retained for
    /// exhaustive matching — `mutex_lock_model` postcondition proves this dead).
    LockTimedOut,
    /// mutex.lock returned Killed.
    LockKilled,
    /// mutex.lock returned GenericError.
    LockGenericError { error_code: i32 },
    /// put_mutex_guard failed (overrides stored result).
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
///
/// # Parameters
///
/// - `mutex_addr`: Mutex address (reordered first since it is the exec param;
///   original API order is `pid, tid, mutex_addr`).
/// - `pid`: Ghost process identifier (matches original's pid parameter).
/// - `tid`: Ghost thread identifier (matches original's tid parameter).
#[verifier::external_body]
pub fn take_mutex_guard_model(
    mutex_addr: u32,
    pid: Ghost<u32>,
    tid: Ghost<u32>,
) -> (result: TakeMutexGuardOutcomeModel)
    requires
        spec_is_currently_running(pid@ as nat, tid@ as nat),
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
/// Ghost parameters `timeout_s` and `timeout_ns` carry the parsed timeout
/// values for spec-level linkage, ensuring the correct alarm is used.
#[verifier::external_body]
pub fn cond_wait_model(
    has_alarm: bool,
    timeout_s: Ghost<u32>,
    timeout_ns: Ghost<u32>,
) -> (result: CondWaitOutcomeModel)
    requires
        has_alarm ==> spec_is_finite_timeout(timeout_s@ as nat, timeout_ns@ as nat),
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
/// # Returns
///
/// `(ok, has_alarm)` where ok indicates success and has_alarm
/// indicates whether a finite alarm was produced.
pub fn parse_timeout_model(timeout_s: u32, timeout_ns: u32) -> (result: (bool, bool))
    ensures
        result.0 == spec_timeout_parsed_ok(timeout_s as nat, timeout_ns as nat),
        result.0 ==> (result.1 == spec_is_finite_timeout(timeout_s as nat, timeout_ns as nat)),
        !result.0 ==> !result.1,
{
    if timeout_s == u32::MAX && timeout_ns == u32::MAX {
        (true, false)
    } else if timeout_ns < 1_000_000_000u32 {
        (true, true)
    } else {
        (false, false)
    }
}

/// Verified helper: models get_cond + optional cond.wait → stored result.
///
/// # Description
///
/// Encapsulates the "stored result" computation from the original code (lines
/// 109-122). If get_cond fails, the error is stored and cond.wait is NOT called.
/// If get_cond succeeds, cond.wait is called and its result is stored.
///
/// Returns the stored result model and ghost views of both step outcomes for
/// exec-spec linkage.
///
/// # Parameters
///
/// - `cond_addr`: Condition variable address.
/// - `has_alarm`: Whether a finite timeout alarm was set.
/// - `timeout_s`: Ghost timeout seconds (threaded through to cond_wait_model).
/// - `timeout_ns`: Ghost timeout nanoseconds (threaded through to cond_wait_model).
pub fn get_cond_and_wait_model(
    cond_addr: u32,
    has_alarm: bool,
    timeout_s: Ghost<u32>,
    timeout_ns: Ghost<u32>,
) -> (ret: (
    WaitCondResultModel,
    Ghost<GetCondOutcomeView>,
    Ghost<CondWaitOutcomeView>,
))
    requires
        has_alarm ==> spec_is_finite_timeout(timeout_s@ as nat, timeout_ns@ as nat),
    ensures
        ret.0.spec_view() == spec_stored_result(ret.1@, ret.2@),
        ret.1@ matches GetCondOutcomeView::GcError { error_code }
            ==> spec_is_valid_error_code(error_code),
        ret.2@ matches CondWaitOutcomeView::CwTimedOut ==> has_alarm,
{
    let gc_result: GetCondOutcomeModel = get_cond_model(cond_addr);
    let ghost gc_view: GetCondOutcomeView = gc_result.spec_view();

    match gc_result {
        GetCondOutcomeModel::Error { error_code } => {
            // get_cond failed: store error, cond.wait is never called.
            // Don't-care value: spec_stored_result ignores cw when gc fails,
            // proven by lemma_get_cond_fail_ignores_cond_wait.
            let ghost cw_view: CondWaitOutcomeView = CondWaitOutcomeView::CwOk;
            (
                WaitCondResultModel::GetCondError { error_code },
                Ghost(gc_view),
                Ghost(cw_view),
            )
        },
        GetCondOutcomeModel::Ok => {
            // get_cond succeeded: call cond.wait and store its result.
            let cw_result: CondWaitOutcomeModel =
                cond_wait_model(has_alarm, timeout_s, timeout_ns);
            let ghost cw_view: CondWaitOutcomeView = cw_result.spec_view();
            let stored: WaitCondResultModel = match cw_result {
                CondWaitOutcomeModel::Ok => WaitCondResultModel::Success,
                CondWaitOutcomeModel::TimedOut => WaitCondResultModel::CondWaitTimedOut,
                CondWaitOutcomeModel::Killed => WaitCondResultModel::CondWaitKilled,
                CondWaitOutcomeModel::GenericError { error_code } => {
                    WaitCondResultModel::CondWaitGenericError { error_code }
                },
            };
            (stored, Ghost(gc_view), Ghost(cw_view))
        },
    }
}

/// Verified exec model of the `wait_cond` kernel call.
///
/// # Description
///
/// This function mirrors the original `pub unsafe fn wait_cond(...)` control flow:
/// 1. Parse timeout.
/// 2. take_mutex_guard → release mutex (short-circuits on error).
/// 3. get_cond + cond.wait → compute "stored result". If get_cond fails, the
///    error is stored and cond.wait is NOT called. NO short-circuit.
/// 4. put_cond → continuation pipeline (overrides stored result on error).
/// 5. get_mutex → continuation pipeline.
/// 6. mutex.lock(None) → continuation pipeline.
/// 7. put_mutex_guard → continuation pipeline.
/// 8. Return stored result from step 3.
///
/// **Critical semantic detail**: Steps 4-7 execute regardless of step 3's outcome.
/// Errors in steps 4-7 override the stored result via the `?` operator. The stored
/// result is returned only when all continuation steps succeed.
///
/// # Parameters
///
/// - `cond_addr`: Condition variable address.
/// - `mutex_addr`: Mutex address.
/// - `timeout_s`: Timeout seconds (u32 on x86-32).
/// - `timeout_ns`: Timeout nanoseconds (u32 on x86-32).
/// - `pid`: Ghost process identifier (matches original's pid parameter).
/// - `tid`: Ghost thread identifier (matches original's tid parameter).
///
/// # Returns
///
/// A tuple of the exec result model and a ghost `WaitCondGhostState` capturing
/// all step outcomes for exec-spec equivalence.
pub fn wait_cond_model(
    cond_addr: u32,
    mutex_addr: u32,
    timeout_s: u32,
    timeout_ns: u32,
    pid: Ghost<u32>,
    tid: Ghost<u32>,
) -> (ret: (WaitCondResultModel, Ghost<WaitCondGhostState>))
    requires
        USIZE_BITS() == 32,
        spec_wait_cond_safety_preconditions(pid@ as nat, tid@ as nat),
        spec_is_currently_running(pid@ as nat, tid@ as nat),
    ensures
        // Exec-spec equivalence: the result matches the spec function.
        ret.0.spec_view() == spec_wait_cond_result(
            timeout_s as nat, timeout_ns as nat,
            ret.1@.tmg, ret.1@.gc, ret.1@.cw,
            ret.1@.pc, ret.1@.gm, ret.1@.lo, ret.1@.pg,
        ),
        // Mutex protocol on stored-result return: whenever the continuation
        // pipeline completes successfully (returning the stored result), the
        // mutex was released before the wait and reacquired afterward, and the
        // condvar reference was released.
        spec_is_stored_result_return(ret.0.spec_view()) ==> (
            spec_mutex_released(mutex_addr as nat)
            && spec_cond_ref_released(cond_addr as nat)
            && spec_mutex_reacquired(mutex_addr as nat)
        ),
        // Resource release: mutex was released whenever take_mutex_guard succeeded,
        // independent of later continuation errors.
        ret.1@.tmg matches TakeMutexGuardOutcomeView::TmgOk
            ==> spec_mutex_released(mutex_addr as nat),
        // Resource release: condvar reference was released whenever put_cond
        // succeeded, independent of later continuation errors.
        ret.1@.pc matches PutCondOutcomeView::PcOk
            ==> spec_cond_ref_released(cond_addr as nat),
        // Resource release: mutex was reacquired whenever lock succeeded,
        // independent of later put_guard errors.
        ret.1@.lo matches LockOutcomeView::LoOk
            ==> spec_mutex_reacquired(mutex_addr as nat),
{
    // Step 1: Parse timeout.
    let parse_result: (bool, bool) = parse_timeout_model(timeout_s, timeout_ns);
    let timeout_ok: bool = parse_result.0;
    let has_alarm: bool = parse_result.1;

    if !timeout_ok {
        proof {
            assert(22i32 as int == ERROR_CODE_INVALID_ARGUMENT());
        }
        let result: WaitCondResultModel =
            WaitCondResultModel::InvalidTimeoutError { error_code: 22i32 };
        let ghost gs: WaitCondGhostState = WaitCondGhostState {
            // Don't-care values: spec short-circuits on invalid timeout,
            // so these are never inspected.
            tmg: TakeMutexGuardOutcomeView::TmgOk,
            gc: GetCondOutcomeView::GcOk,
            cw: CondWaitOutcomeView::CwOk,
            pc: PutCondOutcomeView::PcOk,
            gm: GetMutexOutcomeView::GmOk,
            lo: LockOutcomeView::LoOk,
            pg: PutGuardOutcomeView::PgOk,
        };
        return (result, Ghost(gs));
    }

    // Step 2: take_mutex_guard → release mutex.
    let tmg_result: TakeMutexGuardOutcomeModel =
        take_mutex_guard_model(mutex_addr, pid, tid);
    let ghost tmg_view: TakeMutexGuardOutcomeView = tmg_result.spec_view();
    match tmg_result {
        TakeMutexGuardOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel =
                WaitCondResultModel::TakeMutexGuardError { error_code };
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                // Don't-care values: spec short-circuits on TmgError,
                // so subsequent step outcomes are never inspected.
                gc: GetCondOutcomeView::GcOk,
                cw: CondWaitOutcomeView::CwOk,
                pc: PutCondOutcomeView::PcOk,
                gm: GetMutexOutcomeView::GmOk,
                lo: LockOutcomeView::LoOk,
                pg: PutGuardOutcomeView::PgOk,
            };
            return (result, Ghost(gs));
        },
        TakeMutexGuardOutcomeModel::Ok => {},
    }

    // Steps 3+4: get_cond + optional cond.wait → stored result.
    // In the original code (lines 109-122), get_cond failure stores the error
    // and cond.wait is NOT called. The continuation pipeline runs regardless.
    let gcw_ret: (
        WaitCondResultModel,
        Ghost<GetCondOutcomeView>,
        Ghost<CondWaitOutcomeView>,
    ) = get_cond_and_wait_model(
        cond_addr,
        has_alarm,
        Ghost(timeout_s),
        Ghost(timeout_ns),
    );
    let stored: WaitCondResultModel = gcw_ret.0;
    let ghost gc_view: GetCondOutcomeView = gcw_ret.1@;
    let ghost cw_view: CondWaitOutcomeView = gcw_ret.2@;

    // Step 5: put_cond (runs unconditionally after get_cond+cond.wait block).
    let pc_result: PutCondOutcomeModel = put_cond_model(cond_addr);
    let ghost pc_view: PutCondOutcomeView = pc_result.spec_view();
    match pc_result {
        PutCondOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel =
                WaitCondResultModel::PutCondError { error_code };
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: GetMutexOutcomeView::GmOk,
                lo: LockOutcomeView::LoOk,
                pg: PutGuardOutcomeView::PgOk,
            };
            return (result, Ghost(gs));
        },
        PutCondOutcomeModel::Ok => {},
    }

    // Step 6: get_mutex.
    let gm_result: GetMutexOutcomeModel = get_mutex_model(mutex_addr);
    let ghost gm_view: GetMutexOutcomeView = gm_result.spec_view();
    match gm_result {
        GetMutexOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel =
                WaitCondResultModel::GetMutexError { error_code };
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: gm_view,
                lo: LockOutcomeView::LoOk,
                pg: PutGuardOutcomeView::PgOk,
            };
            return (result, Ghost(gs));
        },
        GetMutexOutcomeModel::Ok => {},
    }

    // Step 7: mutex.lock(None) — reacquire with infinite wait.
    let lock_result: LockOutcomeModel = mutex_lock_model(mutex_addr);
    let ghost lo_view: LockOutcomeView = lock_result.spec_view();
    match lock_result {
        LockOutcomeModel::TimedOut => {
            // Unreachable: mutex_lock_model postcondition guarantees
            // !matches!(result, LockOutcomeModel::TimedOut).
            // Branch retained for exhaustive matching; postcondition vacuously true.
            let result: WaitCondResultModel = WaitCondResultModel::LockTimedOut;
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: gm_view,
                lo: lo_view,
                pg: PutGuardOutcomeView::PgOk,
            };
            return (result, Ghost(gs));
        },
        LockOutcomeModel::Killed => {
            let result: WaitCondResultModel = WaitCondResultModel::LockKilled;
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: gm_view,
                lo: lo_view,
                pg: PutGuardOutcomeView::PgOk,
            };
            return (result, Ghost(gs));
        },
        LockOutcomeModel::GenericError { error_code } => {
            let result: WaitCondResultModel =
                WaitCondResultModel::LockGenericError { error_code };
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: gm_view,
                lo: lo_view,
                pg: PutGuardOutcomeView::PgOk,
            };
            return (result, Ghost(gs));
        },
        LockOutcomeModel::Ok => {},
    }

    // Step 8: put_mutex_guard.
    let pg_result: PutGuardOutcomeModel = put_mutex_guard_model(mutex_addr);
    let ghost pg_view: PutGuardOutcomeView = pg_result.spec_view();
    match pg_result {
        PutGuardOutcomeModel::Error { error_code } => {
            let result: WaitCondResultModel =
                WaitCondResultModel::PutGuardError { error_code };
            let ghost gs: WaitCondGhostState = WaitCondGhostState {
                tmg: tmg_view,
                gc: gc_view,
                cw: cw_view,
                pc: pc_view,
                gm: gm_view,
                lo: lo_view,
                pg: pg_view,
            };
            return (result, Ghost(gs));
        },
        PutGuardOutcomeModel::Ok => {},
    }

    // Step 9: All continuation steps succeeded. Return stored result.
    let ghost gs: WaitCondGhostState = WaitCondGhostState {
        tmg: tmg_view,
        gc: gc_view,
        cw: cw_view,
        pc: pc_view,
        gm: gm_view,
        lo: lo_view,
        pg: pg_view,
    };
    (stored, Ghost(gs))
}

} // verus!
