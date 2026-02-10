// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Dispatcher Verification Model
//!
//! Formal verification of the kernel call dispatcher routing logic.
//!
//! ## Overview
//!
//! The `do_kcall` function is the high-level kernel call entry point. It:
//! 1. Retrieves the current process and thread identifiers from the ProcessManager.
//! 2. Classifies the kernel call number into a dispatch category.
//! 3. Routes the call to the appropriate local handler or the remote scoreboard.
//! 4. Handles sleep errors uniformly via `handle_sleep_error`.
//!
//! ## Verified Properties
//!
//! - **Dispatch classification totality**: Every u32 maps to exactly one
//!   DispatchCategory. No kcall number is unclassified.
//! - **Dispatch classification correctness**: Each of the 32 defined kcall
//!   numbers maps to the correct handler category matching the original match
//!   statement.
//! - **Local/Remote partition**: A kcall is either locally handled or dispatched
//!   to the scoreboard, never both.
//! - **Sleepable subset**: All sleepable calls are locally handled.
//! - **handle_sleep_error correctness**: Generic errors preserve the error code.
//!   TimedOut interruptions produce OperationTimedOut (error code 110).
//!   The Killed path is modeled as a divergent trust boundary.
//! - **Result well-formedness**: All result constructors produce well-formed
//!   results when given valid inputs.
//! - **Error code preservation**: Generic sleep errors carry the original error
//!   code through to the dispatch result.
//! - **Encoding injectivity**: Error-encoded i64 values always fit in i32 range,
//!   providing a necessary condition for distinguishing error from success.
//!
//! ## Verification Model
//!
//! The original `do_kcall` is an `extern "C"` function that uses `unsafe` global
//! state (`ProcessManager::get()`, `ScoreBoard::get_mut()`). For verification:
//! - KcallNumber is modeled as u32 constants matching the `#[repr(u32)]` values.
//! - The dispatch classification is modeled as a pure spec function.
//! - `handle_sleep_error` is modeled as a pure function with full verification.
//! - External subsystem calls (ProcessManager, ScoreBoard, ipc, event, pm)
//!   are modeled via `external_body` boundary functions.
//! - The `do_kcall` entry point is `external_body` because it requires unsafe
//!   global state access that cannot be modeled in Verus.
//!
//! ## API Mapping
//!
//! | Original API                    | Verified Model               | Notes                     |
//! |---------------------------------|------------------------------|---------------------------|
//! | `KcallNumber` enum              | u32 spec constants           | Same `#[repr(u32)]` values|
//! | `KcallNumber::from(u32)`        | `classify_kcall_number()`    | Classification function   |
//! | `handle_sleep_error(SleepError)`| `handle_sleep_error()`       | Non-divergent paths     |
//! | *(Killed path diverges)*        | `handle_sleep_error_killed()`| Divergent trust boundary|
//! | `do_kcall()`                    | `do_kcall()`                 | External body w/ specs  |
//! | `KcallResult::ok()`             | `DispatchResult::ok()`       | Verified constructor      |
//! | `KcallResult::Success(v)`       | `DispatchResult::success(v)` | Verified constructor      |
//! | `KcallResult::Error(e)`         | `DispatchResult::error(e)`   | Verified constructor      |
//!
//! ## Trust Boundaries
//!
//! - **T1: ProcessManager access.** `ProcessManager::get()` accesses a `static mut`
//!   global. The safety of this access (single-threaded context, initialized) is
//!   assumed. The ProcessManager is a dependency boundary type.
//! - **T2: ScoreBoard access.** `ScoreBoard::get_mut()` accesses a `static mut`
//!   global. The scoreboard protocol is separately verified in `kernel::kcall::scoreboard`.
//! - **T3: PM subsystem calls.** `pm::join_thread`, `pm::lock_mutex`, etc. are
//!   dependency boundary operations. Their correctness is assumed.
//! - **T4: ProcessManager::exit divergence.** In the `Interrupted(Killed)` path,
//!   `ProcessManager::exit()` is called and the thread terminates. The original
//!   code panics if exit fails. This divergent path is modeled but not verified
//!   (it should never return).

use vstd::prelude::*;

// Include specifications.
include!("dispatcher.spec.rs");

// Include proofs.
include!("dispatcher.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// Model of a kernel call dispatch result.
///
/// # Description
///
/// Models the `KcallResult` enum from the original code:
/// - `Success(KcallSuccess(i64))` → `is_success: true, value: i64`
/// - `Error(KcallError(i32))` → `is_success: false, value: i32 as i64`
///
/// Fields are `pub` because Verus requires public fields for spec-level access.
pub struct DispatchResult {
    /// Whether this result is a success variant.
    pub is_success: bool,
    /// The payload value.
    pub value: i64,
}

/// Model of the kernel call dispatch arguments.
///
/// # Description
///
/// Models the five u32 arguments passed to `do_kcall`.
pub struct DispatchArgs {
    /// Kernel call number.
    pub number: u32,
    /// First argument.
    pub arg0: u32,
    /// Second argument.
    pub arg1: u32,
    /// Third argument.
    pub arg2: u32,
    /// Fourth argument.
    pub arg3: u32,
}

/// Model of a sleep error.
///
/// # Description
///
/// Models the `SleepError` enum from the process manager:
/// - `Generic(Error)` → `kind: Generic, error_code: error.code as i64`
/// - `Interrupted(Killed)` → `kind: InterruptedKilled`
/// - `Interrupted(TimedOut)` → `kind: InterruptedTimedOut`
///
/// The error_code is stored as i64 for uniformity with DispatchResult.
pub struct SleepError {
    /// The kind of sleep error.
    pub kind: SleepErrorKind,
    /// The error code (meaningful only for Generic kind).
    pub error_code: i64,
}

//==================================================================================================
// Implementation: DispatchResult
//==================================================================================================

impl DispatchResult {
    /// Creates a success result with value 0 (ok).
    ///
    /// # Returns
    ///
    /// A well-formed success DispatchResult with value 0.
    pub fn ok() -> (result: DispatchResult)
        ensures
            result.is_success,
            result.value == 0,
            result.wf(),
            result@ == spec_ok_result(),
    {
        DispatchResult { is_success: true, value: 0 }
    }

    /// Creates a success result with the given value.
    ///
    /// # Parameters
    ///
    /// - `value`: The success payload.
    ///
    /// # Returns
    ///
    /// A well-formed success DispatchResult.
    pub fn success(value: i64) -> (result: DispatchResult)
        ensures
            result.is_success,
            result.value == value,
            result.wf(),
            result@ == spec_success_result(value as int),
    {
        DispatchResult { is_success: true, value }
    }

    /// Creates an error result with the given error code.
    ///
    /// # Parameters
    ///
    /// - `code`: The error code (must fit in i32).
    ///
    /// # Returns
    ///
    /// A well-formed error DispatchResult.
    pub fn error(code: i32) -> (result: DispatchResult)
        ensures
            !result.is_success,
            result.value == code as i64,
            result.wf(),
            result@ == spec_error_result(code as int),
    {
        DispatchResult { is_success: false, value: code as i64 }
    }
}

//==================================================================================================
// Implementation: DispatchArgs
//==================================================================================================

impl DispatchArgs {
    /// Creates new dispatch arguments.
    ///
    /// # Parameters
    ///
    /// - `number`: Kernel call number.
    /// - `arg0`..`arg3`: Kernel call arguments.
    ///
    /// # Returns
    ///
    /// A well-formed DispatchArgs.
    pub fn new(number: u32, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> (result: DispatchArgs)
        ensures
            result.number == number,
            result.arg0 == arg0,
            result.arg1 == arg1,
            result.arg2 == arg2,
            result.arg3 == arg3,
            result.wf(),
    {
        DispatchArgs { number, arg0, arg1, arg2, arg3 }
    }
}

//==================================================================================================
// Implementation: SleepError
//==================================================================================================

impl SleepError {
    /// Creates a Generic sleep error.
    ///
    /// # Parameters
    ///
    /// - `error_code`: The error code from the original Error.
    ///
    /// # Returns
    ///
    /// A well-formed SleepError of Generic kind.
    pub fn generic(error_code: i32) -> (result: SleepError)
        ensures
            result.kind =~= SleepErrorKind::Generic,
            result.error_code == error_code as i64,
            result.wf(),
    {
        SleepError { kind: SleepErrorKind::Generic, error_code: error_code as i64 }
    }

    /// Creates an InterruptedKilled sleep error.
    ///
    /// # Returns
    ///
    /// A SleepError representing a killed interruption.
    pub fn interrupted_killed() -> (result: SleepError)
        ensures
            result.kind =~= SleepErrorKind::InterruptedKilled,
            result.wf(),
    {
        SleepError { kind: SleepErrorKind::InterruptedKilled, error_code: 0 }
    }

    /// Creates an InterruptedTimedOut sleep error.
    ///
    /// # Returns
    ///
    /// A SleepError representing a timed-out interruption.
    pub fn interrupted_timed_out() -> (result: SleepError)
        ensures
            result.kind =~= SleepErrorKind::InterruptedTimedOut,
            result.wf(),
    {
        SleepError { kind: SleepErrorKind::InterruptedTimedOut, error_code: 0 }
    }
}

//==================================================================================================
// Standalone Functions: Classification
//==================================================================================================

/// Classifies a kernel call number into its dispatch category.
///
/// # Description
///
/// Maps a u32 kcall number to the handler path taken by `do_kcall`.
/// This function mirrors the match statement in the original dispatcher.
///
/// # Parameters
///
/// - `number`: The kernel call number (u32).
///
/// # Returns
///
/// The DispatchCategory for this kcall number.
pub fn classify_kcall_number(number: u32) -> (result: DispatchCategory)
    ensures
        result =~= spec_classify_kcall(number),
{
    if number == 1 || number == 2 {
        // GetPid (1), GetTid (2).
        DispatchCategory::LocalImmediate
    } else if number == 3 || number == 22 {
        // Exit (3), ExitThread (22).
        DispatchCategory::LocalTerminal
    } else if number == 23 || number == 9 || number == 24 || number == 27 || number == 29 {
        // JoinThread (23), Recv (9), MutexLock (24), CondWait (27), Sleep (29).
        DispatchCategory::LocalSleepable
    } else if number == 25 || number == 26 || number == 20 {
        // MutexUnlock (25), CondSignal (26), SchedulerYield (20).
        DispatchCategory::LocalFallible
    } else if number == 5 {
        // Resume (5).
        DispatchCategory::LocalDirect
    } else {
        // Everything else: dispatched to scoreboard.
        DispatchCategory::Remote
    }
}

/// Checks if a kernel call is handled locally.
///
/// # Description
///
/// Returns true if the kcall is handled directly by `do_kcall` without
/// dispatching to the scoreboard.
///
/// # Parameters
///
/// - `number`: The kernel call number.
///
/// # Returns
///
/// True if the kcall is locally handled.
pub fn is_locally_handled(number: u32) -> (result: bool)
    ensures
        result == spec_is_locally_handled(number),
{
    let category: DispatchCategory = classify_kcall_number(number);
    !matches!(category, DispatchCategory::Remote)
}

/// Checks if a kernel call may block (sleep).
///
/// # Description
///
/// Returns true if the kcall may cause the calling thread to sleep.
///
/// # Parameters
///
/// - `number`: The kernel call number.
///
/// # Returns
///
/// True if the kcall is sleepable.
pub fn is_sleepable(number: u32) -> (result: bool)
    ensures
        result == spec_is_sleepable(number),
{
    let category: DispatchCategory = classify_kcall_number(number);
    matches!(category, DispatchCategory::LocalSleepable)
}

//==================================================================================================
// Standalone Functions: Error Handling
//==================================================================================================

/// Handles a sleep error by converting it to a dispatch result.
///
/// # Description
///
/// Models the original `handle_sleep_error` function for non-divergent cases:
/// - `Generic(error)` → Error result with the error code.
/// - `InterruptedTimedOut` → Error result with OperationTimedOut (110).
///
/// The `InterruptedKilled` case is excluded by precondition because the
/// original code calls `ProcessManager::exit()` and panics — it never
/// returns. That divergent path is modeled separately by
/// `handle_sleep_error_killed()`.
///
/// # Parameters
///
/// - `sleep_error`: The sleep error to handle (consumed by value, matching original).
///
/// # Returns
///
/// A DispatchResult representing the error.
pub fn handle_sleep_error(sleep_error: SleepError) -> (result: DispatchResult)
    requires
        sleep_error.wf(),
        spec_sleep_error_returns(sleep_error.kind),
    ensures
        result@ == spec_handle_sleep_error(sleep_error.kind, sleep_error.error_code as int),
        !result.is_success,
        result.wf(),
{
    match sleep_error.kind {
        SleepErrorKind::Generic => {
            DispatchResult::error(sleep_error.error_code as i32)
        },
        SleepErrorKind::InterruptedTimedOut => {
            DispatchResult::error(110i32)
        },
        SleepErrorKind::InterruptedKilled => {
            // Unreachable: excluded by precondition spec_sleep_error_returns.
            DispatchResult::error(-1i32)
        },
    }
}

/// Models the divergent `InterruptedKilled` path of `handle_sleep_error`.
///
/// # Description
///
/// In the original code, `SleepError::Interrupted(Killed)` causes
/// `ProcessManager::exit()` to be called, followed by a `panic!`. The
/// process is terminated and this function never returns.
///
/// This is an `external_body` trust boundary because:
/// 1. The divergence cannot be modeled in Verus (no `!` return type support).
/// 2. The function calls unsafe global state (`ProcessManager::exit()`).
///
/// The postcondition `ensures false` documents that this function diverges.
/// As an `external_body`, this is a trusted assertion.
#[verifier::external_body]
pub fn handle_sleep_error_killed() -> (result: DispatchResult)
    ensures
        false,
{
    // Trust boundary: original calls ProcessManager::exit() then panic!().
    // This function never returns.
    panic!("handle_sleep_error_killed: divergent path")
}

//==================================================================================================
// Standalone Functions: Entry Point (External Body)
//==================================================================================================

/// High-level kernel call dispatcher entry point.
///
/// # Description
///
/// Models the original `do_kcall` extern "C" function. This is an external body
/// because it accesses unsafe global state (ProcessManager, ScoreBoard) that
/// cannot be modeled in Verus.
///
/// # Parameters
///
/// - `args`: The dispatch arguments (number and four u32 args).
///
/// # Returns
///
/// The kernel call result as a DispatchResult.
///
/// # Trust Boundary
///
/// This function is the trust boundary between user-space kernel calls and the
/// verified dispatch logic. The routing classification and error handling are
/// verified; the actual subsystem operations are dependency boundary calls.
///
/// The postconditions document the intended contract:
/// - LocalImmediate calls (GetPid, GetTid) always produce a success result.
/// - The result is always well-formed.
/// - The classification of the kcall number determines the dispatch path.
#[verifier::external_body]
pub fn do_kcall(args: DispatchArgs) -> (result: DispatchResult)
    ensures
        result.wf(),
        // LocalImmediate calls (GetPid, GetTid) always succeed.
        spec_classify_kcall(args.number) =~= DispatchCategory::LocalImmediate ==> result.is_success,
        // The dispatch category is determined by the kcall number.
        spec_do_kcall_result_category(args@) =~= spec_classify_kcall(args.number),
{
    // Boundary: actual implementation accesses ProcessManager and ScoreBoard
    // via unsafe global state. See trust boundaries T1-T4.
    unimplemented!()
}

} // verus!
