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
//! - **Dispatch match structure**: The match dispatch in `do_kcall_dispatch`
//!   is verified to route each kcall number to the correct subsystem call.
//!   GetPid/GetTid return identity values; terminal calls always return error.
//! - **Sleep error routing**: Sleepable call errors are verified to route
//!   through `handle_sleep_error` (or diverge on Killed via
//!   `handle_sleep_error_killed`).
//! - **Remote dispatch routing**: The remote scoreboard path is split into
//!   verified routing logic (`remote_dispatch_verified`) over small
//!   external bodies (`scoreboard_get_mut`, `scoreboard_dispatch_call`),
//!   reusing `handle_sleep_error` for error conversion.
//! - **Success payload correctness**: ok()-returning calls (Recv, MutexLock,
//!   CondWait, Sleep, MutexUnlock, SchedulerYield) verified to return value 0
//!   on success. JoinThread success value >= 0 (u32 exit status).
//!   GetPid/GetTid success value >= 0 (non-negative identifiers).
//! - **Conditional pid/tid guarantee**: `do_kcall_context` surfaces the
//!   conditional property that GetPid/GetTid return non-negative values
//!   when ProcessManager access succeeds.
//! - **Local/Remote partition**: A kcall is either locally handled or dispatched
//!   to the scoreboard, never both.
//! - **Sleepable subset**: All sleepable calls are locally handled.
//! - **handle_sleep_error correctness**: Generic errors preserve the error code.
//!   TimedOut interruptions produce OperationTimedOut (error code 116).
//!   The Killed path is modeled as a divergent trust boundary.
//! - **Result well-formedness**: All result constructors produce well-formed
//!   results when given valid inputs.
//! - **Error code preservation**: Generic sleep errors carry the original error
//!   code through to the dispatch result.
//! - **Encoding injectivity**: Error-encoded i64 values always fit in i32 range,
//!   providing a necessary condition for distinguishing error from success.
//! - **Kcall constant consistency**: All 33 spec constants are documented with
//!   their expected values for manual cross-reference against the source enum.
//!
//! ## Verification Model
//!
//! The original `do_kcall` is an `extern "C"` function that uses `unsafe` global
//! state (`ProcessManager::get()`, `ScoreBoard::get_mut()`). For verification:
//! - KcallNumber is modeled as u32 constants matching the `#[repr(u32)]` values.
//! - The dispatch classification is modeled as a pure spec function.
//! - `handle_sleep_error` is modeled as a pure function with full verification.
//! - Individual subsystem calls (ProcessManager, ScoreBoard, ipc, event, pm)
//!   are modeled as small `external_body` boundary functions.
//! - The dispatch match structure is fully verified in `do_kcall_dispatch`.
//! - The pid/tid retrieval + dispatch flow is verified in `do_kcall_context`.
//! - The `do_kcall` entry point delegates to `do_kcall_context` and is fully
//!   verified. Only the ABI representation gap (C `u32`/`i64` ↔ Verus
//!   `DispatchArgs`/`DispatchResult`) remains as a trust boundary (T5).
//!
//! ## API Mapping
//!
//! | Original API                    | Verified Model               | Notes                     |
//! |---------------------------------|------------------------------|---------------------------|
//! | `KcallNumber` enum              | u32 spec constants           | Same `#[repr(u32)]` values|
//! | `KcallNumber::from(u32)`        | `classify_kcall_number()`    | Classification function   |
//! | `handle_sleep_error(SleepError)`| `handle_sleep_error()`       | Non-divergent paths     |
//! | *(Killed path diverges)*        | `handle_sleep_error_killed()`| Divergent trust boundary|
//! | `do_kcall()` match structure    | `do_kcall_dispatch()`        | Verified dispatch logic |
//! | `do_kcall()` pid/tid + dispatch | `do_kcall_context()`         | Verified entry wrapper  |
//! | `do_kcall()` C ABI             | `do_kcall()`                 | Verified (delegates)      |
//! | `KcallResult::ok()`             | `DispatchResult::ok()`       | Verified constructor      |
//! | `KcallResult::Success(v)`       | `DispatchResult::success(v)` | Verified constructor      |
//! | `KcallResult::Error(e)`         | `DispatchResult::error(e)`   | Verified constructor      |
//! | `ScoreBoard::get_mut()`         | `scoreboard_get_mut()`       | External body (T2)        |
//! | `scoreboard.dispatch()`         | `scoreboard_dispatch_call()` | External body (T2)        |
//! | Remote wildcard path            | `remote_dispatch_verified()` | Verified routing          |
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
//! - **T4: ProcessManager::exit divergence (decomposed).** In the
//!   `Interrupted(Killed)` path, `handle_sleep_error_killed()` is now verified
//!   (not external_body) and calls two external bodies in sequence:
//!   - **T4a: `pm_exit_interrupted()`** — models `ProcessManager::exit(ErrorCode::Interrupted)`.
//!     The exit side-effect is trusted but now explicitly part of the model.
//!   - **T4b: `diverge_after_exit()`** — models the `panic!()`. Ensures `false`.
//!   The verification proves that exit is performed before divergence.
//! - **T5: ABI representation gap (mostly closed).** The original `do_kcall`
//!   uses the C ABI `extern "C" fn(u32, u32, u32, u32, u32) -> i64`. The
//!   verified `do_kcall_abi` function now matches this signature, constructing
//!   `DispatchArgs` from raw u32 parameters and encoding the result as i64.
//!   The only remaining trust assumption is that the Rust calling convention
//!   delivers the u32/i64 values faithfully (a compiler concern).
//! - **T6: Kcall number constants.** The 33 spec constants (KCALL_DEBUG through
//!   KCALL_INVALID) are manually mirrored from the `KcallNumber` `#[repr(u32)]`
//!   enum in `src/libs/sys/src/sys/number.rs`. The `lemma_kcall_constants_consistency`
//!   proof asserts each value for regression, but the mirroring is not mechanically
//!   linked to the source. If the enum values change, the spec constants must be
//!   manually updated. A CI check diffing the enum values against the spec is
//!   recommended.
//!
//! ## Scope Limitations
//!
//! - **Liveness**: No liveness properties (eventual return, deadlock freedom) are
//!   specified or proved for sleepable or remote-dispatch calls. The external-body
//!   subsystem stubs assume termination. Liveness verification requires modeling
//!   the scheduler and scoreboard concurrency protocol, which is out of scope for
//!   the dispatcher module.

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

/// Model of a subsystem call that may block (sleep).
///
/// # Description
///
/// Models return types from subsystem calls that may result in a SleepError:
/// - `Ok(value)` → `succeeded: true, value: success payload`
/// - `Err(SleepError)` → `succeeded: false, sleep error details`
///
/// Used for JoinThread, Recv, MutexLock, CondWait, Sleep subsystem calls.
pub struct SleepableOutcome {
    /// Whether the subsystem call succeeded.
    pub succeeded: bool,
    /// The success value (meaningful only when `succeeded`).
    pub value: i64,
    /// The sleep error kind (meaningful only when `!succeeded`).
    pub sleep_error_kind: SleepErrorKind,
    /// The sleep error code (meaningful only when `!succeeded` and `Generic`).
    pub sleep_error_code: i64,
}

/// Model of a subsystem call that may fail but does not sleep.
///
/// # Description
///
/// Models return types from subsystem calls that return Ok or Err without sleeping:
/// - `Ok(value)` → `succeeded: true, value: success payload`
/// - `Err(e)` → `succeeded: false, error_code: e.code`
///
/// Used for pid/tid retrieval, MutexUnlock, CondSignal, SchedulerYield.
pub struct FallibleOutcome {
    /// Whether the subsystem call succeeded.
    pub succeeded: bool,
    /// The success value (meaningful only when `succeeded`).
    pub value: i64,
    /// The error code (meaningful only when `!succeeded`).
    pub error_code: i32,
}

/// Model of a scoreboard dispatch outcome.
///
/// # Description
///
/// Models `scoreboard.dispatch()` which returns `Result<KcallResult, SleepError>`:
/// - `Ok(KcallResult)` → `succeeded: true`, result fields carry the KcallResult.
/// - `Err(SleepError)` → `succeeded: false`, sleep error fields carry the error.
pub struct ScoreboardDispatchOutcome {
    /// Whether the scoreboard dispatch succeeded.
    pub succeeded: bool,
    /// The result success/error flag (meaningful only when `succeeded`).
    pub result_is_success: bool,
    /// The result value (meaningful only when `succeeded`).
    pub result_value: i64,
    /// The sleep error kind (meaningful only when `!succeeded`).
    pub sleep_error_kind: SleepErrorKind,
    /// The sleep error code (meaningful only when `!succeeded` and `Generic`).
    pub sleep_error_code: i64,
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
/// - `InterruptedTimedOut` → Error result with OperationTimedOut (116).
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
            DispatchResult::error(116i32)
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
/// `ProcessManager::exit(ErrorCode::Interrupted)` to be called, followed
/// by a `panic!`. The process is terminated and this function never returns.
///
/// This function is now verified (not `external_body`). It calls two
/// external bodies in sequence:
/// 1. `pm_exit_interrupted()` — models `ProcessManager::exit(ErrorCode::Interrupted)`.
/// 2. `diverge_after_exit()` — models the `panic!()` that follows.
///
/// This decomposition proves that the exit side-effect is performed
/// before divergence, narrowing the trust boundary from "the whole
/// function" to the two individual operations.
///
/// **Trust assumptions**:
/// - `pm_exit_interrupted()` performs forced process termination (T4a).
/// - `diverge_after_exit()` never returns (T4b).
pub fn handle_sleep_error_killed() -> (result: DispatchResult)
    ensures
        false, // This function diverges (never returns).
{
    pm_exit_interrupted();
    diverge_after_exit()
}

/// Models `ProcessManager::exit(ErrorCode::Interrupted)`.
///
/// # Description
///
/// Performs forced process termination on the Killed path. This is the
/// safety-critical side-effect of the Interrupted(Killed) sleep error.
/// The original code calls `ProcessManager::exit(ErrorCode::Interrupted)`.
///
/// Trust boundary T4a: the correctness of this call is assumed.
#[verifier::external_body]
fn pm_exit_interrupted()
    ensures true,
{ unimplemented!() }

/// Models the `panic!()` after `ProcessManager::exit()` on the Killed path.
///
/// # Description
///
/// This function never returns. It models the `panic!("do_kcall()")`
/// in the original code that follows the exit call.
///
/// Trust boundary T4b: this function is assumed to never return.
#[verifier::external_body]
fn diverge_after_exit() -> (result: DispatchResult)
    ensures
        false, // This function diverges (never returns).
{
    panic!("diverge_after_exit: killed path divergence")
}

//==================================================================================================
// Subsystem Boundary Functions (External Bodies)
//==================================================================================================
//
// Each function models one ProcessManager / subsystem operation.
// These are dependency boundary types — their correctness is assumed
// via trust boundaries T1–T3.
//
// NOTE: The original code passes `arg0 as usize` to many subsystem calls.
// On the target x86-32 platform, `usize` is 32 bits (same as `u32`),
// so the verified model accepts `u32` directly. If the code were ported
// to a 64-bit target, this equivalence would no longer hold and the
// external bodies would need to be updated.

/// Retrieves the current process identifier from the ProcessManager.
///
/// # Returns
///
/// FallibleOutcome: success with pid value (>= 0), or error code on failure.
#[verifier::external_body]
fn pm_get_pid() -> (result: FallibleOutcome)
    ensures result.wf(), result.succeeded ==> (result.value >= 0 && result.value <= i32::MAX as i64),
{ unimplemented!() }

/// Retrieves the current thread identifier from the ProcessManager.
///
/// # Returns
///
/// FallibleOutcome: success with tid value (>= 0, <= i32::MAX), or error code on failure.
#[verifier::external_body]
fn pm_get_tid() -> (result: FallibleOutcome)
    ensures result.wf(), result.succeeded ==> (result.value >= 0 && result.value <= i32::MAX as i64),
{ unimplemented!() }

/// Models ProcessManager::exit (Exit kcall).
///
/// # Description
///
/// Exit always returns Err in the original code (the process terminates,
/// `exit()` returns `unwrap_err()`).
#[verifier::external_body]
fn pm_exit(arg0: u32) -> (result: DispatchResult)
    ensures !result.is_success, result.wf(),
{ unimplemented!() }

/// Models ProcessManager::exit_thread (ExitThread kcall).
///
/// # Description
///
/// ExitThread always returns Err in the original code.
#[verifier::external_body]
fn pm_exit_thread(arg0: u32) -> (result: DispatchResult)
    ensures !result.is_success, result.wf(),
{ unimplemented!() }

/// Models pm::join_thread (JoinThread kcall).
///
/// # Description
///
/// On success, returns the exit status as a u32 converted to i64 (>= 0).
#[verifier::external_body]
fn pm_join_thread(pid: i64, arg0: u32, arg1: u32) -> (result: SleepableOutcome)
    ensures result.wf(), result.succeeded ==> result.value >= 0,
{ unimplemented!() }

/// Models ipc::recv (Recv kcall).
///
/// # Description
///
/// On success, returns KcallResult::ok() (value 0).
#[verifier::external_body]
fn ipc_recv(tid: i64, pid: i64, arg0: u32) -> (result: SleepableOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }

/// Models event::resume (Resume kcall).
///
/// # Description
///
/// Returns KcallResult directly from the event subsystem.
#[verifier::external_body]
fn event_resume(arg0: u32) -> (result: DispatchResult)
    ensures result.wf(),
{ unimplemented!() }

/// Models pm::lock_mutex (MutexLock kcall).
///
/// # Description
///
/// On success, returns KcallResult::ok() (value 0).
#[verifier::external_body]
fn pm_lock_mutex(pid: i64, tid: i64, arg0: u32, arg1: u32, arg2: u32) -> (result: SleepableOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }

/// Models pm::unlock_mutex (MutexUnlock kcall).
///
/// # Description
///
/// On success, returns KcallResult::ok() (value 0).
#[verifier::external_body]
fn pm_unlock_mutex(pid: i64, tid: i64, arg0: u32) -> (result: FallibleOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }

/// Models pm::wait_cond (CondWait kcall).
///
/// # Description
///
/// On success, returns KcallResult::ok() (value 0).
#[verifier::external_body]
fn pm_wait_cond(pid: i64, tid: i64, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> (result: SleepableOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }

/// Models pm::signal_cond (CondSignal kcall).
///
/// # Description
///
/// The original calls `pm::signal_cond(pid, tid, arg0 as usize, arg1 != 0)`
/// where the 4th argument is a `bool` broadcast flag. The external body
/// accepts a `bool` matching the original signature.
#[verifier::external_body]
fn pm_signal_cond(pid: i64, tid: i64, arg0: u32, broadcast: bool) -> (result: FallibleOutcome)
    ensures result.wf(), result.succeeded ==> result.value >= 0,
{ unimplemented!() }

/// Models ProcessManager::giveup (SchedulerYield kcall).
///
/// # Description
///
/// On success, returns KcallResult::ok() (value 0).
#[verifier::external_body]
fn pm_giveup() -> (result: FallibleOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }

/// Models pm::sleep (Sleep kcall).
///
/// # Description
///
/// On success, returns KcallResult::ok() (value 0).
#[verifier::external_body]
fn pm_sleep(arg0: u32, arg1: u32) -> (result: SleepableOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }

/// Models ScoreBoard::get_mut().
///
/// # Description
///
/// Acquires mutable access to the scoreboard singleton. Returns a
/// FallibleOutcome: success means access was granted; error means
/// the scoreboard is unavailable (value carries the error code).
/// The scoreboard protocol is separately verified in
/// `kernel::kcall::scoreboard`.
#[verifier::external_body]
fn scoreboard_get_mut() -> (result: FallibleOutcome)
    ensures result.wf(),
{ unimplemented!() }

/// Models scoreboard.dispatch().
///
/// # Description
///
/// Dispatches a kernel call to the scoreboard for remote execution.
/// Returns `Result<KcallResult, SleepError>` modeled as
/// ScoreboardDispatchOutcome.
#[verifier::external_body]
fn scoreboard_dispatch_call(number: u32, pid: i64, tid: i64, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> (result: ScoreboardDispatchOutcome)
    ensures result.wf(),
{ unimplemented!() }

//==================================================================================================
// Verified Dispatch Logic
//==================================================================================================

/// Verified remote dispatch path.
///
/// # Description
///
/// Implements the remote scoreboard dispatch path from the original
/// `do_kcall` wildcard match arm:
/// 1. `ScoreBoard::get_mut()` — acquire scoreboard access (may fail).
/// 2. `scoreboard.dispatch()` — dispatch to scoreboard (may return
///    `Ok(KcallResult)` or `Err(SleepError)`).
/// 3. On sleep error, routes through `handle_sleep_error` (or diverges
///    on Killed via `handle_sleep_error_killed`).
///
/// Each step is a small external body; the routing logic between them
/// is fully verified.
///
/// # Parameters
///
/// - `number`: Kernel call number.
/// - `pid`: Process identifier.
/// - `tid`: Thread identifier.
/// - `arg0`..`arg3`: Kernel call arguments.
///
/// # Returns
///
/// A well-formed DispatchResult (or diverges on Killed).
fn remote_dispatch_verified(number: u32, pid: i64, tid: i64, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> (result: DispatchResult)
    ensures
        result.wf(),
{
    let sb_outcome: FallibleOutcome = scoreboard_get_mut();
    if !sb_outcome.succeeded {
        DispatchResult::error(sb_outcome.error_code)
    } else {
        let dispatch_outcome: ScoreboardDispatchOutcome =
            scoreboard_dispatch_call(number, pid, tid, arg0, arg1, arg2, arg3);
        if dispatch_outcome.succeeded {
            if dispatch_outcome.result_is_success {
                DispatchResult::success(dispatch_outcome.result_value)
            } else {
                DispatchResult::error(dispatch_outcome.result_value as i32)
            }
        } else {
            match dispatch_outcome.sleep_error_kind {
                SleepErrorKind::InterruptedKilled => {
                    // SOUNDNESS NOTE: handle_sleep_error_killed calls
                    // pm_exit_interrupted() then diverge_after_exit().
                    // If the original Killed path changes, update T4a/T4b.
                    handle_sleep_error_killed()
                },
                SleepErrorKind::Generic => {
                    handle_sleep_error(SleepError {
                        kind: SleepErrorKind::Generic,
                        error_code: dispatch_outcome.sleep_error_code,
                    })
                },
                SleepErrorKind::InterruptedTimedOut => {
                    handle_sleep_error(SleepError {
                        kind: SleepErrorKind::InterruptedTimedOut,
                        error_code: dispatch_outcome.sleep_error_code,
                    })
                },
            }
        }
    }
}

/// Converts a sleepable subsystem outcome to a dispatch result.
///
/// # Description
///
/// Routes the outcome of a sleepable subsystem call:
/// - Success → `DispatchResult::success(value)`.
/// - `SleepError(Generic)` → `handle_sleep_error` → error with original code.
/// - `SleepError(TimedOut)` → `handle_sleep_error` → error 116.
/// - `SleepError(Killed)` → `handle_sleep_error_killed` → diverges.
///
/// # Parameters
///
/// - `outcome`: The sleepable subsystem call result.
///
/// # Returns
///
/// A well-formed DispatchResult (or diverges on Killed).
fn convert_sleepable(outcome: SleepableOutcome) -> (result: DispatchResult)
    requires
        outcome.wf(),
    ensures
        result.wf(),
        outcome.succeeded ==> (result.is_success && result.value == outcome.value),
        !outcome.succeeded ==> !result.is_success,
{
    if outcome.succeeded {
        DispatchResult::success(outcome.value)
    } else {
        match outcome.sleep_error_kind {
            SleepErrorKind::InterruptedKilled => {
                // SOUNDNESS NOTE: handle_sleep_error_killed calls
                // pm_exit_interrupted() then diverge_after_exit().
                // If the original Killed path changes, update T4a/T4b.
                handle_sleep_error_killed()
            },
            SleepErrorKind::Generic => {
                handle_sleep_error(SleepError {
                    kind: SleepErrorKind::Generic,
                    error_code: outcome.sleep_error_code,
                })
            },
            SleepErrorKind::InterruptedTimedOut => {
                handle_sleep_error(SleepError {
                    kind: SleepErrorKind::InterruptedTimedOut,
                    error_code: outcome.sleep_error_code,
                })
            },
        }
    }
}

/// Converts a fallible subsystem outcome to a dispatch result.
///
/// # Description
///
/// Routes the outcome of a non-sleeping subsystem call:
/// - Success → `DispatchResult::success(value)`.
/// - Error → `DispatchResult::error(error_code)`.
///
/// # Parameters
///
/// - `outcome`: The fallible subsystem call result.
///
/// # Returns
///
/// A well-formed DispatchResult.
fn convert_fallible(outcome: FallibleOutcome) -> (result: DispatchResult)
    requires
        outcome.wf(),
    ensures
        result.wf(),
        outcome.succeeded ==> (result.is_success && result.value == outcome.value),
        !outcome.succeeded ==> (!result.is_success && result.value == outcome.error_code as i64),
{
    if outcome.succeeded {
        DispatchResult::success(outcome.value)
    } else {
        DispatchResult::error(outcome.error_code)
    }
}

/// Verified dispatch logic after pid/tid retrieval.
///
/// # Description
///
/// Implements the core match statement from the original `do_kcall` function.
/// Each branch routes to the appropriate subsystem call (modeled as external
/// bodies) and converts the result.
///
/// The match structure is verified to:
/// - Return the correct pid/tid for GetPid/GetTid.
/// - Always return error for Exit/ExitThread (terminal calls).
/// - Route sleepable call errors through `handle_sleep_error`.
/// - Produce well-formed results for all paths.
///
/// # Parameters
///
/// - `pid`: The current process identifier (from ProcessManager).
/// - `tid`: The current thread identifier (from ProcessManager).
/// - `args`: The dispatch arguments.
///
/// # Returns
///
/// A well-formed DispatchResult.
fn do_kcall_dispatch(pid: i64, tid: i64, args: DispatchArgs) -> (result: DispatchResult)
    requires
        pid >= 0,
        tid >= 0,
    ensures
        result.wf(),
        // GetPid returns the pid value.
        args.number == 1u32 ==> (result.is_success && result.value == pid),
        // GetTid returns the tid value.
        args.number == 2u32 ==> (result.is_success && result.value == tid),
        // Terminal calls always return error.
        spec_classify_kcall(args.number) =~= DispatchCategory::LocalTerminal
            ==> !result.is_success,
        // ok()-returning sleepable calls: success value is 0.
        (args.number == 9u32 || args.number == 24u32
            || args.number == 27u32 || args.number == 29u32)
            && result.is_success ==> result.value == 0,
        // JoinThread: success value >= 0 (u32 exit status).
        args.number == 23u32 && result.is_success ==> result.value >= 0,
        // ok()-returning fallible calls: success value is 0.
        (args.number == 25u32 || args.number == 20u32)
            && result.is_success ==> result.value == 0,
        // CondSignal: success value >= 0 (count of woken threads).
        args.number == 26u32 && result.is_success ==> result.value >= 0,
        // Sleepable/fallible error paths produce well-formed error results.
        (args.number == 9u32 || args.number == 23u32 || args.number == 24u32
            || args.number == 27u32 || args.number == 29u32
            || args.number == 25u32 || args.number == 26u32
            || args.number == 20u32)
            && !result.is_success ==> (result.value >= i32::MIN as i64
                                        && result.value <= i32::MAX as i64),
{
    let number: u32 = args.number;
    if number == 1u32 {
        // GetPid: return pid directly.
        DispatchResult::success(pid)
    } else if number == 2u32 {
        // GetTid: return tid directly.
        DispatchResult::success(tid)
    } else if number == 3u32 {
        // Exit: always returns error (process terminates).
        pm_exit(args.arg0)
    } else if number == 22u32 {
        // ExitThread: always returns error (thread terminates).
        pm_exit_thread(args.arg0)
    } else if number == 23u32 {
        // JoinThread: sleepable.
        convert_sleepable(pm_join_thread(pid, args.arg0, args.arg1))
    } else if number == 9u32 {
        // Recv: sleepable.
        convert_sleepable(ipc_recv(tid, pid, args.arg0))
    } else if number == 5u32 {
        // Resume: direct result from event subsystem.
        event_resume(args.arg0)
    } else if number == 24u32 {
        // MutexLock: sleepable.
        convert_sleepable(pm_lock_mutex(pid, tid, args.arg0, args.arg1, args.arg2))
    } else if number == 25u32 {
        // MutexUnlock: fallible.
        convert_fallible(pm_unlock_mutex(pid, tid, args.arg0))
    } else if number == 27u32 {
        // CondWait: sleepable.
        convert_sleepable(pm_wait_cond(pid, tid, args.arg0, args.arg1, args.arg2, args.arg3))
    } else if number == 26u32 {
        // CondSignal: fallible. arg1 != 0 is the broadcast flag.
        convert_fallible(pm_signal_cond(pid, tid, args.arg0, args.arg1 != 0))
    } else if number == 20u32 {
        // SchedulerYield: fallible.
        convert_fallible(pm_giveup())
    } else if number == 29u32 {
        // Sleep: sleepable.
        convert_sleepable(pm_sleep(args.arg0, args.arg1))
    } else {
        // Remote: dispatched to scoreboard (verified routing).
        remote_dispatch_verified(args.number, pid, tid, args.arg0, args.arg1, args.arg2, args.arg3)
    }
}

/// Verified dispatch with pid/tid retrieval.
///
/// # Description
///
/// Models the full `do_kcall` flow including pid/tid retrieval from
/// ProcessManager. If pid/tid retrieval fails, returns an error
/// immediately regardless of the kcall number.
///
/// This verified function delegates to `do_kcall_dispatch` after
/// successful pid/tid retrieval.
///
/// ## Error-Code Propagation (Verification Note)
///
/// Error-code propagation is verified internally through the constructor
/// chain. Each failure path calls `DispatchResult::error(code)` whose
/// postcondition is `result.value == code as i64` (proven by
/// `lemma_error_constructor_preserves_code`). This guarantees that
/// subsystem error codes are faithfully propagated to the result.
///
/// For GetPid/GetTid specifically, `lemma_getpid_gettid_dispatch_infallible`
/// proves that the dispatch path always succeeds. Therefore, if
/// `do_kcall_context` returns an error for GetPid/GetTid, the error
/// necessarily came from pid/tid retrieval (trust boundary T1), not
/// from the dispatch logic.
///
/// The propagation property is not surfaced in the top-level postcondition
/// because Verus `ensures` clauses cannot reference local variables (e.g.,
/// `pid_outcome`). Ghost return values would add complexity without
/// verification value — the error codes originate from external-body
/// subsystem calls (trust boundaries T1–T3), so their specific values
/// are already assumed, not verified. Adding ghost outputs would prove
/// `result.value == ghost@`, which is trivially satisfiable by any
/// implementation setting `ghost = result.value`.
///
/// # Parameters
///
/// - `args`: The dispatch arguments.
///
/// # Returns
///
/// A well-formed DispatchResult.
pub fn do_kcall_context(args: DispatchArgs) -> (result: DispatchResult)
    ensures
        result.wf(),
        // Terminal calls always return error, even if pid/tid retrieval fails.
        spec_classify_kcall(args.number) =~= DispatchCategory::LocalTerminal
            ==> !result.is_success,
        // GetPid/GetTid: if success, value is non-negative (pids/tids >= 0).
        (args.number == 1u32 || args.number == 2u32)
            && result.is_success ==> result.value >= 0,
        // ok()-returning calls: success value is 0.
        (args.number == 9u32 || args.number == 24u32
            || args.number == 27u32 || args.number == 29u32
            || args.number == 25u32 || args.number == 20u32)
            && result.is_success ==> result.value == 0,
        // CondSignal: success value >= 0 (count of woken threads).
        args.number == 26u32 && result.is_success ==> result.value >= 0,
        // JoinThread: success value >= 0 (u32 exit status).
        args.number == 23u32 && result.is_success ==> result.value >= 0,
{
    let pid_outcome: FallibleOutcome = pm_get_pid();
    if !pid_outcome.succeeded {
        DispatchResult::error(pid_outcome.error_code)
    } else {
        let tid_outcome: FallibleOutcome = pm_get_tid();
        if !tid_outcome.succeeded {
            DispatchResult::error(tid_outcome.error_code)
        } else {
            do_kcall_dispatch(pid_outcome.value, tid_outcome.value, args)
        }
    }
}

//==================================================================================================
// Standalone Functions: Entry Point
//==================================================================================================

/// High-level kernel call dispatcher entry point.
///
/// # Description
///
/// Models the original `do_kcall` extern "C" function. Delegates directly to
/// the fully verified `do_kcall_context`, which handles pid/tid retrieval and
/// dispatch routing. All postconditions are mechanically verified.
///
/// # Parameters
///
/// - `args`: The dispatch arguments (number and four u32 args).
///
/// # Returns
///
/// The kernel call result as a DispatchResult.
///
/// # Note on ABI Representation (Trust Boundary T5)
///
/// The original `do_kcall` has the C ABI signature
/// `extern "C" fn(u32, u32, u32, u32, u32) -> i64`. This verified model uses
/// `DispatchArgs` and `DispatchResult` for richer postconditions. The ABI gap
/// is closed by `do_kcall_abi`, which takes raw u32 parameters, constructs
/// `DispatchArgs`, dispatches, and encodes the result as i64.
pub fn do_kcall(args: DispatchArgs) -> (result: DispatchResult)
    ensures
        result.wf(),
        // Terminal calls (Exit, ExitThread) always produce error results.
        spec_classify_kcall(args.number) =~= DispatchCategory::LocalTerminal ==> !result.is_success,
        // General category constraint.
        spec_dispatch_result_constrained(spec_classify_kcall(args.number), result@),
        // GetPid/GetTid: if success, value is non-negative.
        (args.number == 1u32 || args.number == 2u32)
            && result.is_success ==> result.value >= 0,
        // ok()-returning calls: success value is 0.
        (args.number == 9u32 || args.number == 24u32
            || args.number == 27u32 || args.number == 29u32
            || args.number == 25u32 || args.number == 20u32)
            && result.is_success ==> result.value == 0,
        // CondSignal: success value >= 0 (count of woken threads).
        args.number == 26u32 && result.is_success ==> result.value >= 0,
        // JoinThread: success value >= 0.
        args.number == 23u32 && result.is_success ==> result.value >= 0,
{
    do_kcall_context(args)
}

//==================================================================================================
// ABI Encoding
//==================================================================================================

/// Encodes a DispatchResult into an i64 matching the KcallResult::into() conversion.
///
/// # Description
///
/// Models the `Into<i64>` implementation for `KcallResult`:
/// - `Success(KcallSuccess(v))` → `v` (the i64 value directly).
/// - `Error(KcallError(e))` → `e as i64` (i32 sign-extended to i64).
///
/// Both cases encode to `result.value`, which is the identity function.
/// This verified function proves the encoding matches `spec_encode_result`.
///
/// # Parameters
///
/// - `result`: The dispatch result to encode.
///
/// # Returns
///
/// The encoded i64 value.
pub fn encode_result(result: &DispatchResult) -> (encoded: i64)
    ensures
        encoded as int == spec_encode_result(result@),
{
    result.value
}

/// Verified ABI-level dispatcher that returns the encoded i64.
///
/// # Description
///
/// Composes `do_kcall` with `encode_result` to produce the raw i64
/// matching the original `extern "C" fn do_kcall(...) -> i64` return value.
/// This bridges the gap between the typed `DispatchResult` verification
/// model and the C ABI representation (trust boundary T5).
///
/// The postcondition proves the returned i64 equals the spec-level
/// encoding of the verified dispatch result.
///
/// # Parameters
///
/// - `args`: The dispatch arguments.
///
/// # Returns
///
/// The encoded i64 return value.
pub fn do_kcall_encoded(args: DispatchArgs) -> (pair: (DispatchResult, i64))
    ensures ({
        let result: DispatchResult = pair.0;
        let encoded: i64 = pair.1;
        // The result is well-formed.
        &&& result.wf()
        // The encoded i64 equals spec_encode_result of this specific result.
        &&& encoded as int == spec_encode_result(result@)
        // The result satisfies all do_kcall postconditions.
        &&& (spec_classify_kcall(args.number) =~= DispatchCategory::LocalTerminal
                ==> !result.is_success)
        &&& spec_dispatch_result_constrained(spec_classify_kcall(args.number), result@)
        &&& ((args.number == 1u32 || args.number == 2u32)
                && result.is_success ==> result.value >= 0)
        &&& ((args.number == 9u32 || args.number == 24u32
                || args.number == 27u32 || args.number == 29u32
                || args.number == 25u32 || args.number == 20u32)
                && result.is_success ==> result.value == 0)
        &&& (args.number == 26u32 && result.is_success ==> result.value >= 0)
        &&& (args.number == 23u32 && result.is_success ==> result.value >= 0)
    }),
{
    let result: DispatchResult = do_kcall(args);
    proof {
        lemma_encode_result_is_value(result@);
    }
    let encoded: i64 = encode_result(&result);
    (result, encoded)
}

/// Verified C ABI entry point for the kernel call dispatcher.
///
/// # Description
///
/// Matches the original `extern "C" fn do_kcall(number: u32, arg0: u32,
/// arg1: u32, arg2: u32, arg3: u32) -> i64` signature. This function:
/// 1. Constructs `DispatchArgs` from the raw u32 parameters.
/// 2. Calls `do_kcall` to perform the verified dispatch.
/// 3. Encodes the result as an i64 via `encode_result`.
///
/// All three steps are verified, closing the ABI gap (trust boundary T5).
/// The only remaining trust assumption is that the Rust ABI calling
/// convention delivers the u32/i64 values faithfully (a compiler concern,
/// not a verification concern).
///
/// # Parameters
///
/// - `number`: Kernel call number.
/// - `arg0`..`arg3`: Kernel call arguments.
///
/// # Returns
///
/// The encoded i64 return value.
pub fn do_kcall_abi(number: u32, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> (encoded: i64)
    ensures ({
        let args: DispatchArgs = DispatchArgs { number, arg0, arg1, arg2, arg3 };
        exists|r: DispatchResultView| #![auto]
            spec_result_wf(r)
            && encoded as int == spec_encode_result(r)
            && (spec_classify_kcall(number) =~= DispatchCategory::LocalTerminal
                    ==> !r.is_success)
            && spec_dispatch_result_constrained(spec_classify_kcall(number), r)
    }),
{
    let args: DispatchArgs = DispatchArgs::new(number, arg0, arg1, arg2, arg3);
    let result: DispatchResult = do_kcall(args);
    proof {
        lemma_encode_result_is_value(result@);
    }
    encode_result(&result)
}

} // verus!
